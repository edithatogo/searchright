use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    ContractError, DATA_HANDLING_DECISION_SCHEMA_VERSION, DATA_HANDLING_REQUEST_SCHEMA_VERSION,
    DATA_LIFECYCLE_DECISION_SCHEMA_VERSION, DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION,
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

/// Lifecycle operation proposed for persisted review data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataLifecycleAction {
    /// Retain the targets for an explicitly bounded period.
    Retain,
    /// Export an explicitly identified set of targets.
    Export,
    /// Remove mutable content while preserving immutable audit evidence.
    Delete,
}

/// Whether a lifecycle request only previews effects or may authorise them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleExecutionMode {
    /// Calculate a decision without authorising any effect.
    Preview,
    /// Authorise the declared effects when every gate is satisfied.
    Apply,
}

/// Accountable approval scoped to one lifecycle action and review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LifecycleApproval {
    /// Stable approval identifier retained in the audit receipt.
    pub approval_id: String,
    /// Accountable approving principal.
    pub approved_by: String,
    /// Review to which the approval is restricted.
    pub review_id: String,
    /// Lifecycle action that was approved.
    pub action: DataLifecycleAction,
    /// RFC 3339 timestamp validated by the wire schema and treated opaquely by this evaluator.
    pub approved_at: String,
}

/// Preview/apply request for retention, export or deletion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataLifecycleRequest {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable request identifier.
    pub request_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Highest classification among the targets.
    pub classification: DataClassification,
    /// Requested lifecycle action.
    pub action: DataLifecycleAction,
    /// Preview is non-authorising; apply requires scoped approval.
    pub execution_mode: LifecycleExecutionMode,
    /// Stable identifiers for every affected object.
    pub target_ids: Vec<String>,
    /// Requested retention period; valid only for retain requests.
    pub retention_days: Option<u32>,
    /// Explicit export destination; valid only for export requests.
    pub export_destination: Option<String>,
    /// Whether any target is part of the immutable audit ledger.
    pub includes_audit_log: bool,
    /// Whether a legal or preservation hold currently covers any target.
    pub legal_hold: bool,
    /// Scoped accountable approval. Preview requests must not carry one.
    pub approval: Option<LifecycleApproval>,
}

/// Deterministic lifecycle policy decision; it does not perform the effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataLifecycleDecision {
    /// Contract identifier.
    pub schema_version: String,
    /// Request identifier.
    pub request_id: String,
    /// Policy identifier.
    pub policy_id: String,
    /// Whether the declared request satisfies policy.
    pub permitted: bool,
    /// Whether a caller may perform effects after persisting this decision.
    pub effects_authorized: bool,
    /// Stable blocking reason codes.
    pub blockers: Vec<String>,
    /// Stable non-blocking caution codes.
    pub warnings: Vec<String>,
    /// Audit events must remain immutable under every lifecycle action.
    pub immutable_audit_preserved: bool,
    /// Targets that must receive durable tombstones if deletion is applied.
    pub tombstone_target_ids: Vec<String>,
    /// Every apply path must persist a content-addressed receipt.
    pub receipt_required: bool,
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

impl Validate for DataLifecycleRequest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION,
            "data_lifecycle_request.schema_version",
        )?;
        require_text(&self.request_id, "data_lifecycle_request.request_id")?;
        require_text(&self.review_id, "data_lifecycle_request.review_id")?;
        if self.target_ids.is_empty() {
            return Err(ContractError::EmptyCollection(
                "data_lifecycle_request.target_ids",
            ));
        }
        if self
            .target_ids
            .iter()
            .any(|target| target.trim().is_empty())
        {
            return Err(ContractError::Invariant(
                "lifecycle target identifiers must be non-empty".to_owned(),
            ));
        }
        if self.target_ids.iter().collect::<BTreeSet<_>>().len() != self.target_ids.len() {
            return Err(ContractError::Invariant(
                "lifecycle target identifiers must be unique".to_owned(),
            ));
        }
        match self.action {
            DataLifecycleAction::Retain => {
                if self.retention_days == Some(0) || self.retention_days.is_none() {
                    return Err(ContractError::Invariant(
                        "retain requests require a positive retention_days value".to_owned(),
                    ));
                }
                if self.export_destination.is_some() {
                    return Err(ContractError::Invariant(
                        "retain requests cannot declare an export destination".to_owned(),
                    ));
                }
            }
            DataLifecycleAction::Export => {
                let destination = self.export_destination.as_deref().ok_or_else(|| {
                    ContractError::Invariant(
                        "export requests require an explicit destination".to_owned(),
                    )
                })?;
                require_text(destination, "data_lifecycle_request.export_destination")?;
                if self.retention_days.is_some() {
                    return Err(ContractError::Invariant(
                        "export requests cannot change retention".to_owned(),
                    ));
                }
            }
            DataLifecycleAction::Delete => {
                if self.retention_days.is_some() || self.export_destination.is_some() {
                    return Err(ContractError::Invariant(
                        "delete requests cannot declare retention or export fields".to_owned(),
                    ));
                }
            }
        }
        if matches!(self.execution_mode, LifecycleExecutionMode::Preview) && self.approval.is_some()
        {
            return Err(ContractError::Invariant(
                "preview requests must not consume an approval".to_owned(),
            ));
        }
        if let Some(approval) = &self.approval {
            require_text(
                &approval.approval_id,
                "data_lifecycle_request.approval.approval_id",
            )?;
            require_text(
                &approval.approved_by,
                "data_lifecycle_request.approval.approved_by",
            )?;
            require_text(
                &approval.review_id,
                "data_lifecycle_request.approval.review_id",
            )?;
            require_text(
                &approval.approved_at,
                "data_lifecycle_request.approval.approved_at",
            )?;
        }
        Ok(())
    }
}

impl Validate for DataLifecycleDecision {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DATA_LIFECYCLE_DECISION_SCHEMA_VERSION,
            "data_lifecycle_decision.schema_version",
        )?;
        require_text(&self.request_id, "data_lifecycle_decision.request_id")?;
        require_text(&self.policy_id, "data_lifecycle_decision.policy_id")?;
        if self.permitted != self.blockers.is_empty() {
            return Err(ContractError::Invariant(
                "lifecycle permission and blockers disagree".to_owned(),
            ));
        }
        if self.effects_authorized && (!self.permitted || !self.receipt_required) {
            return Err(ContractError::Invariant(
                "authorised lifecycle effects require permission and a receipt".to_owned(),
            ));
        }
        if !self.immutable_audit_preserved {
            return Err(ContractError::Invariant(
                "a lifecycle decision cannot waive immutable audit preservation".to_owned(),
            ));
        }
        Ok(())
    }
}
