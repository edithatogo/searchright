//! Compatibility and parity helpers for a controlled Sourceright shared-core migration.

#![forbid(unsafe_code)]

use evidence_search_core::ProviderMode;
use schemars::JsonSchema;
use searchright_contracts::{
    ExecutionPolicy, ParityDimensionResult, SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION,
    SourcerightParityReport, Validate,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Product-neutral subset of Sourceright's legacy runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LegacyRuntimeConfig {
    /// Whether live calls are enabled.
    pub enabled: bool,
    /// Whether a live smoke operation was explicitly selected.
    pub smoke_enabled: bool,
    /// Timeout in seconds.
    pub timeout_secs: u64,
    /// Minimum call interval.
    pub min_interval_ms: u64,
    /// Retry count.
    pub max_retries: u8,
    /// Whether a cache/replay directory was configured.
    pub cache_enabled: bool,
}

impl LegacyRuntimeConfig {
    /// Translate legacy controls to a bounded shared-core execution policy.
    #[must_use]
    pub fn execution_policy(&self, max_records: u64, max_pages: u32) -> ExecutionPolicy {
        ExecutionPolicy {
            live_enabled: self.enabled && self.smoke_enabled,
            max_records,
            max_pages,
            timeout_seconds: self.timeout_secs,
            total_timeout_seconds: Some(
                self.timeout_secs
                    .saturating_mul(u64::from(max_pages.max(1))),
            ),
            max_retries: self.max_retries,
            min_interval_ms: self.min_interval_ms,
            retry_base_delay_ms: Some(self.min_interval_ms.max(100)),
            retry_max_delay_ms: Some(self.min_interval_ms.max(100).saturating_mul(16)),
            max_response_bytes: Some(16 * 1024 * 1024),
            replay_enabled: self.cache_enabled,
            cache_write_enabled: self.cache_enabled,
        }
    }
}

/// Legacy execution classification retained for parity comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LegacyExecution {
    /// Fixture-backed operation.
    Fixture,
    /// Live operation.
    Live,
    /// Operation did not run.
    Skipped,
}

/// Map a legacy execution class to a shared-core mode where one exists.
#[must_use]
pub const fn provider_mode(value: LegacyExecution) -> Option<ProviderMode> {
    match value {
        LegacyExecution::Fixture => Some(ProviderMode::Fixture),
        LegacyExecution::Live => Some(ProviderMode::Live),
        LegacyExecution::Skipped => None,
    }
}

/// One old/new parity observation supplied by a dual-run harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParityObservation {
    /// Stable dimension.
    pub dimension: String,
    /// Legacy normalised value.
    pub legacy: Value,
    /// Shared-core normalised value.
    pub shared: Value,
    /// Optional approved difference identifier.
    pub approved_difference_id: Option<String>,
    /// Review note.
    pub note: String,
}

/// Build a deterministic advisory summary from normalised observations.
///
/// The v1 report cannot verify provider/fixture/case-level execution coverage.
/// Callers with incomplete execution evidence must supply explicit blockers.
/// An operational cutover requires a separately validated complete matrix and
/// owner decisions; `cutover_ready` alone must never enable it.
pub fn build_report(
    legacy_revision: &str,
    shared_revision: &str,
    case_ids: Vec<String>,
    observations: &[ParityObservation],
    blockers: Vec<String>,
    generated_at: &str,
) -> Result<SourcerightParityReport, CompatibilityError> {
    let mut dimensions = Vec::with_capacity(observations.len());
    for observation in observations {
        let legacy_digest = digest(&observation.legacy)?;
        let shared_digest = digest(&observation.shared)?;
        dimensions.push(ParityDimensionResult {
            dimension: observation.dimension.clone(),
            equivalent: legacy_digest == shared_digest,
            legacy_digest,
            shared_digest,
            approved_difference_id: observation.approved_difference_id.clone(),
            note: observation.note.clone(),
        });
    }
    let cutover_ready = blockers.is_empty()
        && dimensions
            .iter()
            .all(|item| item.equivalent || item.approved_difference_id.is_some());
    let report = SourcerightParityReport {
        schema_version: SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION.to_owned(),
        legacy_revision: legacy_revision.to_owned(),
        shared_revision: shared_revision.to_owned(),
        case_ids,
        dimensions,
        cutover_ready,
        blockers,
        generated_at: generated_at.to_owned(),
    };
    report.validate()?;
    Ok(report)
}

fn digest(value: &Value) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(&canonicalise(value))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn canonicalise(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalise).collect()),
        Value::Object(fields) => {
            let mut entries = fields
                .iter()
                .map(|(name, value)| (name.clone(), canonicalise(value)))
                .collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            Value::Object(entries.into_iter().collect())
        }
        scalar => scalar.clone(),
    }
}

/// Compatibility-harness failure.
#[derive(Debug, thiserror::Error)]
pub enum CompatibilityError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Observation serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{SOURCERIGHT_PARITY_CASE_IDS, SOURCERIGHT_PARITY_DIMENSIONS};
    use serde_json::json;

    use super::*;

    fn complete_case_ids() -> Vec<String> {
        SOURCERIGHT_PARITY_CASE_IDS
            .iter()
            .map(|value| (*value).to_owned())
            .collect()
    }

    fn complete_observations() -> Vec<ParityObservation> {
        SOURCERIGHT_PARITY_DIMENSIONS
            .iter()
            .map(|dimension| ParityObservation {
                dimension: (*dimension).to_owned(),
                legacy: json!({"value": dimension}),
                shared: json!({"value": dimension}),
                approved_difference_id: None,
                note: "declared parity dimension matches".to_owned(),
            })
            .collect()
    }

    #[test]
    fn identical_observations_make_the_advisory_summary_ready_without_blockers() {
        let report = build_report(
            "legacy",
            "shared",
            complete_case_ids(),
            &complete_observations(),
            Vec::new(),
            "source-epoch:1785984000",
        );
        assert!(report.is_ok());
        if let Ok(report) = report {
            assert!(report.cutover_ready);
        }
    }

    #[test]
    fn complete_summary_retains_missing_execution_matrix_blocker()
    -> Result<(), Box<dyn std::error::Error>> {
        let blocker = "provider/fixture/case execution matrix not yet evaluated";
        let report = build_report(
            "legacy",
            "shared",
            complete_case_ids(),
            &complete_observations(),
            vec![blocker.to_owned()],
            "source-epoch:1785984000",
        )?;
        assert!(!report.cutover_ready);
        assert_eq!(report.blockers, vec![blocker.to_owned()]);
        Ok(())
    }

    #[test]
    fn object_field_order_does_not_create_a_false_difference()
    -> Result<(), Box<dyn std::error::Error>> {
        let legacy: Value =
            serde_json::from_str(r#"{"provider":"pubmed","ids":{"pmid":"1","doi":"10/test"}}"#)?;
        let shared: Value =
            serde_json::from_str(r#"{"ids":{"doi":"10/test","pmid":"1"},"provider":"pubmed"}"#)?;
        let report = build_report(
            "legacy",
            "shared",
            complete_case_ids(),
            &complete_observations()
                .into_iter()
                .map(|mut observation| {
                    if observation.dimension == "identifiers" {
                        observation.legacy = legacy.clone();
                        observation.shared = shared.clone();
                    }
                    observation
                })
                .collect::<Vec<_>>(),
            Vec::new(),
            "source-epoch:1785984000",
        )?;
        assert!(report.cutover_ready);
        assert!(
            report
                .dimensions
                .first()
                .is_some_and(|item| item.equivalent)
        );
        Ok(())
    }

    #[test]
    fn an_unapproved_difference_is_reportable_when_it_has_a_blocker()
    -> Result<(), Box<dyn std::error::Error>> {
        let report = build_report(
            "legacy",
            "shared",
            vec!["bounded-retry".to_owned()],
            &[ParityObservation {
                dimension: "retry count".to_owned(),
                legacy: json!(3),
                shared: json!(2),
                approved_difference_id: None,
                note: "requires downstream review".to_owned(),
            }],
            vec!["downstream parity review pending".to_owned()],
            "source-epoch:1785984000",
        )?;
        assert!(!report.cutover_ready);
        assert!(
            report
                .dimensions
                .first()
                .is_some_and(|item| !item.equivalent)
        );
        Ok(())
    }

    #[test]
    fn an_unapproved_difference_cannot_be_marked_unblocked() {
        let result = build_report(
            "legacy",
            "shared",
            vec!["bounded-retry".to_owned()],
            &[ParityObservation {
                dimension: "retry count".to_owned(),
                legacy: json!(3),
                shared: json!(2),
                approved_difference_id: None,
                note: "requires downstream review".to_owned(),
            }],
            Vec::new(),
            "source-epoch:1785984000",
        );
        assert!(result.is_err());
    }

    #[test]
    fn a_blank_difference_approval_cannot_make_a_valid_summary() {
        let result = build_report(
            "legacy",
            "shared",
            vec!["bounded-retry".to_owned()],
            &[ParityObservation {
                dimension: "retry count".to_owned(),
                legacy: json!(3),
                shared: json!(2),
                approved_difference_id: Some("  ".to_owned()),
                note: "requires a real approval identifier".to_owned(),
            }],
            Vec::new(),
            "source-epoch:1785984000",
        );
        assert!(result.is_err());
    }

    #[test]
    fn incomplete_or_invented_catalogues_cannot_make_a_ready_summary() {
        let incomplete = build_report(
            "legacy",
            "shared",
            vec!["invented-case".to_owned()],
            &[ParityObservation {
                dimension: "invented dimension".to_owned(),
                legacy: json!(1),
                shared: json!(1),
                approved_difference_id: None,
                note: "not declared by the parity catalogue".to_owned(),
            }],
            Vec::new(),
            "source-epoch:1785984000",
        );
        assert!(incomplete.is_err());
    }

    #[test]
    fn blank_and_duplicate_coverage_is_rejected() {
        let observations = complete_observations();
        let blank = build_report(
            "legacy",
            "shared",
            vec!["   ".to_owned()],
            &observations,
            vec!["coverage pending".to_owned()],
            "source-epoch:1785984000",
        );
        assert!(blank.is_err());

        let duplicate_cases = build_report(
            "legacy",
            "shared",
            vec!["disabled-live".to_owned(), "disabled-live".to_owned()],
            &observations,
            vec!["coverage pending".to_owned()],
            "source-epoch:1785984000",
        );
        assert!(duplicate_cases.is_err());

        let mut duplicate_dimensions = observations;
        let first_dimension = duplicate_dimensions.first().cloned();
        assert!(first_dimension.is_some());
        if let Some(first_dimension) = first_dimension {
            duplicate_dimensions.push(first_dimension);
        }
        let duplicate_dimensions = build_report(
            "legacy",
            "shared",
            complete_case_ids(),
            &duplicate_dimensions,
            vec!["coverage pending".to_owned()],
            "source-epoch:1785984000",
        );
        assert!(duplicate_dimensions.is_err());
    }
}
