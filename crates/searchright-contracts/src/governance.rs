use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, DATA_HANDLING_DECISION_SCHEMA_VERSION, DATA_HANDLING_REQUEST_SCHEMA_VERSION,
    INSTITUTIONAL_POLICY_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Classification assigned to data handled by a Searchright operation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    /// Public bibliographic metadata.
    PublicMetadata,
    /// Internal review state without sensitive full text.
    InternalReviewData,
    /// Confidential review material or unpublished protocol content.
    Confidential,
    /// Copyright-restricted or contract-restricted full text.
    RestrictedFullText,
    /// Sensitive personal or health information that should normally be excluded.
    SensitivePersonalData,
}

/// Deployment boundary approved by an institution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    /// Local execution on an approved device.
    LocalOnly,
    /// Institution-managed self-hosted deployment.
    InstitutionSelfHosted,
    /// Dedicated hosted tenancy.
    HostedSingleTenant,
    /// Shared hosted service with logical tenancy controls.
    HostedMultiTenant,
}

/// Kind of data operation to evaluate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataOperationKind {
    /// Read or persist bibliographic metadata.
    Metadata,
    /// Analyse full text without retaining it.
    FullTextAnalysis,
    /// Persist full text or excerpts.
    FullTextPersistence,
    /// Export review artefacts.
    Export,
    /// Emit operational telemetry.
    Telemetry,
    /// Use an external model or hosted processing service.
    ExternalModelProcessing,
}

/// Institution-specific policy applied before data processing or deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InstitutionalPolicy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable policy identifier.
    pub policy_id: String,
    /// Institution or project governance owner.
    pub institution: String,
    /// Approved deployment modes.
    pub deployment_modes: Vec<DeploymentMode>,
    /// Highest data classifications allowed.
    pub allowed_classifications: Vec<DataClassification>,
    /// ISO 3166-1 alpha-2 regions in which data may be processed or stored.
    #[serde(default)]
    pub permitted_regions: Vec<String>,
    /// Maximum retention period for derived review data.
    pub maximum_retention_days: u32,
    /// Whether telemetry is permitted.
    pub telemetry_allowed: bool,
    /// Whether restricted full text may be persisted.
    pub full_text_persistence_allowed: bool,
    /// Whether external model processing is permitted.
    pub external_model_processing_allowed: bool,
    /// Whether cross-border transfer outside permitted regions is allowed.
    pub cross_border_transfer_allowed: bool,
    /// Human governance approver.
    pub approved_by: String,
    /// Effective date or timestamp.
    pub effective_from: String,
    /// Review or expiry date, when one has been set.
    pub review_by: Option<String>,
}

/// Proposed operation evaluated against an institutional policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataHandlingRequest {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable request identifier.
    pub request_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Data classification.
    pub classification: DataClassification,
    /// Operation kind.
    pub operation: DataOperationKind,
    /// Deployment mode.
    pub deployment_mode: DeploymentMode,
    /// Processing/storage region.
    pub region: Option<String>,
    /// Requested retention period.
    pub retention_days: u32,
    /// Whether data crosses a regional boundary.
    pub cross_border_transfer: bool,
    /// Whether the operation is dry-run only.
    pub dry_run: bool,
}

/// Deterministic policy decision. Human approval remains authoritative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataHandlingDecision {
    /// Contract identifier.
    pub schema_version: String,
    /// Request identifier.
    pub request_id: String,
    /// Policy identifier.
    pub policy_id: String,
    /// Whether the request is permitted by the machine-readable policy.
    pub permitted: bool,
    /// Stable blocking reason codes.
    pub blockers: Vec<String>,
    /// Non-blocking cautions.
    pub warnings: Vec<String>,
    /// Whether an additional human decision is required.
    pub human_approval_required: bool,
}

impl Validate for InstitutionalPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            INSTITUTIONAL_POLICY_SCHEMA_VERSION,
            "institutional_policy.schema_version",
        )?;
        require_text(&self.policy_id, "institutional_policy.policy_id")?;
        require_text(&self.institution, "institutional_policy.institution")?;
        require_text(&self.approved_by, "institutional_policy.approved_by")?;
        require_text(&self.effective_from, "institutional_policy.effective_from")?;
        if self.deployment_modes.is_empty() {
            return Err(ContractError::EmptyCollection(
                "institutional_policy.deployment_modes",
            ));
        }
        if self.allowed_classifications.is_empty() {
            return Err(ContractError::EmptyCollection(
                "institutional_policy.allowed_classifications",
            ));
        }
        if self.maximum_retention_days == 0 {
            return Err(ContractError::Invariant(
                "institutional retention must be at least one day".to_owned(),
            ));
        }
        if self.permitted_regions.iter().any(|region| {
            region.len() != 2
                || !region
                    .chars()
                    .all(|character| character.is_ascii_uppercase())
        }) {
            return Err(ContractError::Invariant(
                "permitted regions must use upper-case ISO 3166-1 alpha-2 codes".to_owned(),
            ));
        }
        if self.full_text_persistence_allowed
            && !self
                .allowed_classifications
                .contains(&DataClassification::RestrictedFullText)
        {
            return Err(ContractError::Invariant(
                "full-text persistence requires restricted-full-text classification approval"
                    .to_owned(),
            ));
        }
        if let Some(review_by) = self.review_by.as_deref() {
            require_text(review_by, "institutional_policy.review_by")?;
        }
        Ok(())
    }
}

impl Validate for DataHandlingRequest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DATA_HANDLING_REQUEST_SCHEMA_VERSION,
            "data_handling.schema_version",
        )?;
        require_text(&self.request_id, "data_handling.request_id")?;
        require_text(&self.review_id, "data_handling.review_id")?;
        if self.retention_days == 0 {
            return Err(ContractError::Invariant(
                "requested retention must be at least one day".to_owned(),
            ));
        }
        if let Some(region) = self.region.as_deref()
            && (region.len() != 2
                || !region
                    .chars()
                    .all(|character| character.is_ascii_uppercase()))
        {
            return Err(ContractError::Invariant(
                "data-handling region must use an upper-case ISO alpha-2 code".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for DataHandlingDecision {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DATA_HANDLING_DECISION_SCHEMA_VERSION,
            "data_handling_decision.schema_version",
        )?;
        require_text(&self.request_id, "data_handling_decision.request_id")?;
        require_text(&self.policy_id, "data_handling_decision.policy_id")?;
        if self.permitted && !self.blockers.is_empty() {
            return Err(ContractError::Invariant(
                "a permitted data-handling decision cannot contain blockers".to_owned(),
            ));
        }
        if !self.permitted && self.blockers.is_empty() {
            return Err(ContractError::Invariant(
                "a denied data-handling decision requires at least one blocker".to_owned(),
            ));
        }
        if self
            .blockers
            .iter()
            .chain(&self.warnings)
            .any(|value| value.trim().is_empty())
        {
            return Err(ContractError::Invariant(
                "data-handling blocker and warning codes must be non-empty".to_owned(),
            ));
        }
        Ok(())
    }
}
