use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    ContractError, SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// Complete fixture catalogue required before a Sourceright cutover can be ready.
pub const SOURCERIGHT_PARITY_CASE_IDS: &[&str] = &[
    "bounded-retry",
    "cache-write-replay",
    "disabled-live",
    "fixture-identifiers",
    "malformed-payload",
    "secret-redaction",
    "undeclared-host",
];

/// Complete comparison surface required before a Sourceright cutover can be ready.
pub const SOURCERIGHT_PARITY_DIMENSIONS: &[&str] = &[
    "cache key redaction",
    "disabled-live negative behaviour",
    "endpoint and secret redaction",
    "error classification",
    "execution mode",
    "fixture determinism",
    "host policy",
    "identifiers",
    "malformed and adversarial response handling",
    "normalised fields",
    "provider identity",
    "receipt counts",
    "replay and cache behaviour",
    "retry and rate behaviour",
    "timeout behaviour",
];

/// Result for one Sourceright/shared-core migration parity dimension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParityDimensionResult {
    /// Stable dimension name.
    pub dimension: String,
    /// Digest of the legacy observation.
    pub legacy_digest: String,
    /// Digest of the shared-core observation.
    pub shared_digest: String,
    /// Whether the observations are equivalent under the declared comparator.
    pub equivalent: bool,
    /// Approved difference identifier, when behaviour intentionally differs.
    pub approved_difference_id: Option<String>,
    /// Human-readable evidence note.
    pub note: String,
}

/// Advisory dimension summary for a Sourceright shared-core migration.
///
/// Version 1 does not bind observations to provider/fixture/case cells. Its
/// readiness flag cannot prove execution coverage or authorize a cutover;
/// operational consumers must separately require a complete validated matrix
/// and the accountable owner's recorded decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourcerightParityReport {
    /// Contract identifier.
    pub schema_version: String,
    /// Pinned Sourceright source commit or blob.
    pub legacy_revision: String,
    /// Searchright/shared-core revision.
    pub shared_revision: String,
    /// Fixture or scenario identifiers evaluated.
    pub case_ids: Vec<String>,
    /// Results across required parity dimensions.
    pub dimensions: Vec<ParityDimensionResult>,
    /// Whether the supplied dimension summaries are accepted without blockers.
    /// This is not verified case-level coverage or operational authorization.
    pub cutover_ready: bool,
    /// Outstanding blockers.
    pub blockers: Vec<String>,
    /// Generation timestamp or reproducible source epoch label.
    pub generated_at: String,
}

impl Validate for SourcerightParityReport {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION,
            "sourceright_parity.schema_version",
        )?;
        require_text(&self.legacy_revision, "sourceright_parity.legacy_revision")?;
        require_text(&self.shared_revision, "sourceright_parity.shared_revision")?;
        require_text(&self.generated_at, "sourceright_parity.generated_at")?;
        if self.case_ids.is_empty() {
            return Err(ContractError::EmptyCollection(
                "sourceright_parity.case_ids",
            ));
        }
        let case_ids = unique_nonblank(&self.case_ids, "sourceright_parity.case_ids")?;
        if self.dimensions.is_empty() {
            return Err(ContractError::EmptyCollection(
                "sourceright_parity.dimensions",
            ));
        }
        for dimension in &self.dimensions {
            require_text(&dimension.dimension, "sourceright_parity.dimension")?;
            require_text(&dimension.legacy_digest, "sourceright_parity.legacy_digest")?;
            require_text(&dimension.shared_digest, "sourceright_parity.shared_digest")?;
            require_text(&dimension.note, "sourceright_parity.note")?;
            if let Some(approval) = &dimension.approved_difference_id {
                require_text(approval, "sourceright_parity.approved_difference_id")?;
            }
        }
        let dimension_names = unique_nonblank(
            &self
                .dimensions
                .iter()
                .map(|item| item.dimension.clone())
                .collect::<Vec<_>>(),
            "sourceright_parity.dimensions",
        )?;
        let has_unapproved_difference = self
            .dimensions
            .iter()
            .any(|item| !item.equivalent && item.approved_difference_id.is_none());
        if has_unapproved_difference && self.blockers.is_empty() {
            return Err(ContractError::Invariant(
                "unapproved parity differences require an explicit blocker".to_owned(),
            ));
        }
        let all_accepted = self
            .dimensions
            .iter()
            .all(|item| item.equivalent || item.approved_difference_id.is_some());
        if self.cutover_ready != (all_accepted && self.blockers.is_empty()) {
            return Err(ContractError::Invariant(
                "cutover readiness must agree with parity dimensions and blockers".to_owned(),
            ));
        }
        if self.cutover_ready
            && (case_ids != expected_set(SOURCERIGHT_PARITY_CASE_IDS)
                || dimension_names != expected_set(SOURCERIGHT_PARITY_DIMENSIONS))
        {
            return Err(ContractError::Invariant(
                "cutover readiness requires exact parity case and dimension coverage".to_owned(),
            ));
        }
        if self.blockers.iter().any(|value| value.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "parity blockers must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

fn unique_nonblank(
    values: &[String],
    field: &'static str,
) -> Result<BTreeSet<String>, ContractError> {
    let mut unique = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !unique.insert(value.clone()) {
            return Err(ContractError::Invariant(format!(
                "{field} must contain unique values"
            )));
        }
    }
    Ok(unique)
}

fn expected_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
