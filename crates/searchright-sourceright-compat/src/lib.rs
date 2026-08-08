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
    pub const fn execution_policy(&self, max_records: u64, max_pages: u32) -> ExecutionPolicy {
        ExecutionPolicy {
            live_enabled: self.enabled && self.smoke_enabled,
            max_records,
            max_pages,
            timeout_seconds: self.timeout_secs,
            max_retries: self.max_retries,
            min_interval_ms: self.min_interval_ms,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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

/// Build a deterministic report from normalised observations.
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
    let bytes = serde_json::to_vec(value)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
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
    use serde_json::json;

    use super::*;

    #[test]
    fn identical_observations_are_cutover_ready_without_blockers() {
        let report = build_report(
            "legacy",
            "shared",
            vec!["fixture-1".to_owned()],
            &[ParityObservation {
                dimension: "identifiers".to_owned(),
                legacy: json!({"doi": "10.1000/test"}),
                shared: json!({"doi": "10.1000/test"}),
                approved_difference_id: None,
                note: "canonical identifiers match".to_owned(),
            }],
            Vec::new(),
            "source-epoch:1785984000",
        );
        assert!(report.is_ok());
        if let Ok(report) = report {
            assert!(report.cutover_ready);
        }
    }
}
