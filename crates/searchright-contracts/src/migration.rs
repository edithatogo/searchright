use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

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

/// Evidence-bearing compatibility report for a Sourceright shared-core cutover.
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
    /// Whether all dimensions are equivalent or explicitly approved.
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
            if !dimension.equivalent && dimension.approved_difference_id.is_none() {
                return Err(ContractError::Invariant(format!(
                    "non-equivalent dimension `{}` requires an approved difference identifier",
                    dimension.dimension
                )));
            }
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
        if self.blockers.iter().any(|value| value.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "parity blockers must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}
