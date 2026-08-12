use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    /// SHA-256 of the complete requested effect set.
    pub request_digest: String,
    /// Policy identifier against which the request was approved.
    pub policy_id: String,
    /// Single-use value checked by the external approval verifier.
    pub nonce: String,
    /// RFC 3339 timestamp validated by the wire schema and treated opaquely by this evaluator.
    pub approved_at: String,
    /// RFC 3339 expiry checked by the external approval verifier.
    pub expires_at: String,
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
            require_sha256(
                &approval.request_digest,
                "data_lifecycle_request.approval.request_digest",
            )?;
            require_text(
                &approval.policy_id,
                "data_lifecycle_request.approval.policy_id",
            )?;
            require_text(&approval.nonce, "data_lifecycle_request.approval.nonce")?;
            require_text(
                &approval.expires_at,
                "data_lifecycle_request.approval.expires_at",
            )?;
        }
        Ok(())
    }
}

impl DataLifecycleRequest {
    /// Canonical SHA-256 binding every effect-bearing request field except approval evidence.
    #[must_use]
    pub fn effects_digest(&self) -> String {
        let mut bytes = Vec::new();
        push_part(&mut bytes, &self.schema_version);
        push_part(&mut bytes, &self.request_id);
        push_part(&mut bytes, &self.review_id);
        push_part(&mut bytes, &format!("{:?}", self.classification));
        push_part(&mut bytes, &format!("{:?}", self.action));
        push_part(&mut bytes, &format!("{:?}", self.execution_mode));
        for target in &self.target_ids {
            push_part(&mut bytes, target);
        }
        push_part(
            &mut bytes,
            &self
                .retention_days
                .map_or_else(|| "null".to_owned(), |value| value.to_string()),
        );
        push_part(
            &mut bytes,
            self.export_destination.as_deref().unwrap_or("null"),
        );
        push_part(&mut bytes, if self.includes_audit_log { "1" } else { "0" });
        push_part(&mut bytes, if self.legal_hold { "1" } else { "0" });
        sha256_hex(&bytes)
    }
}

fn push_part(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(value.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(value.as_bytes());
}

fn require_sha256(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ContractError::Invariant(format!(
            "{field} must be lowercase SHA-256"
        )))
    }
}

fn sha256_hex(input: &[u8]) -> String {
    Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lifecycle_request(action: DataLifecycleAction) -> DataLifecycleRequest {
        DataLifecycleRequest {
            schema_version: DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "request-1".to_owned(),
            review_id: "review-1".to_owned(),
            classification: DataClassification::PublicMetadata,
            action,
            execution_mode: LifecycleExecutionMode::Preview,
            target_ids: vec!["record-1".to_owned()],
            retention_days: matches!(action, DataLifecycleAction::Retain).then_some(7),
            export_destination: matches!(action, DataLifecycleAction::Export)
                .then(|| "file:///approved/export.srpack".to_owned()),
            includes_audit_log: false,
            legal_hold: false,
            approval: None,
        }
    }

    fn approval(action: DataLifecycleAction) -> LifecycleApproval {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            ..lifecycle_request(action)
        };
        LifecycleApproval {
            approval_id: "approval-1".to_owned(),
            approved_by: "owner-1".to_owned(),
            review_id: "review-1".to_owned(),
            action,
            request_digest: request.effects_digest(),
            policy_id: "policy-1".to_owned(),
            nonce: "nonce-1".to_owned(),
            approved_at: "2026-08-13T00:00:00Z".to_owned(),
            expires_at: "2026-08-14T00:00:00Z".to_owned(),
        }
    }

    fn lifecycle_decision() -> DataLifecycleDecision {
        DataLifecycleDecision {
            schema_version: DATA_LIFECYCLE_DECISION_SCHEMA_VERSION.to_owned(),
            request_id: "request-1".to_owned(),
            policy_id: "policy-1".to_owned(),
            permitted: true,
            effects_authorized: false,
            blockers: Vec::new(),
            warnings: Vec::new(),
            immutable_audit_preserved: true,
            tombstone_target_ids: Vec::new(),
            receipt_required: false,
        }
    }

    #[test]
    fn lifecycle_request_rejects_invalid_identity_and_targets() {
        let invalid_schema = DataLifecycleRequest {
            schema_version: "unsupported".to_owned(),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(invalid_schema.validate().is_err());

        let empty_request_id = DataLifecycleRequest {
            request_id: " ".to_owned(),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(empty_request_id.validate().is_err());

        let empty_review_id = DataLifecycleRequest {
            review_id: String::new(),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(empty_review_id.validate().is_err());

        let no_targets = DataLifecycleRequest {
            target_ids: Vec::new(),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(no_targets.validate().is_err());

        let blank_target = DataLifecycleRequest {
            target_ids: vec![" ".to_owned()],
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(blank_target.validate().is_err());
    }

    #[test]
    fn lifecycle_request_rejects_action_field_mismatches() {
        for request in [
            DataLifecycleRequest {
                retention_days: None,
                ..lifecycle_request(DataLifecycleAction::Retain)
            },
            DataLifecycleRequest {
                retention_days: Some(0),
                ..lifecycle_request(DataLifecycleAction::Retain)
            },
            DataLifecycleRequest {
                export_destination: Some("file:///unexpected".to_owned()),
                ..lifecycle_request(DataLifecycleAction::Retain)
            },
            DataLifecycleRequest {
                export_destination: None,
                ..lifecycle_request(DataLifecycleAction::Export)
            },
            DataLifecycleRequest {
                export_destination: Some(" ".to_owned()),
                ..lifecycle_request(DataLifecycleAction::Export)
            },
            DataLifecycleRequest {
                retention_days: Some(1),
                ..lifecycle_request(DataLifecycleAction::Export)
            },
            DataLifecycleRequest {
                retention_days: Some(1),
                ..lifecycle_request(DataLifecycleAction::Delete)
            },
            DataLifecycleRequest {
                export_destination: Some("file:///unexpected".to_owned()),
                ..lifecycle_request(DataLifecycleAction::Delete)
            },
        ] {
            assert!(request.validate().is_err());
        }
    }

    #[test]
    fn lifecycle_request_rejects_invalid_approval_use_and_fields() {
        let preview_with_approval = DataLifecycleRequest {
            approval: Some(approval(DataLifecycleAction::Delete)),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(preview_with_approval.validate().is_err());

        for invalid in [
            LifecycleApproval {
                approval_id: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                approved_by: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                review_id: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                approved_at: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                request_digest: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                policy_id: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                nonce: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
            LifecycleApproval {
                expires_at: String::new(),
                ..approval(DataLifecycleAction::Delete)
            },
        ] {
            let request = DataLifecycleRequest {
                execution_mode: LifecycleExecutionMode::Apply,
                approval: Some(invalid),
                ..lifecycle_request(DataLifecycleAction::Delete)
            };
            assert!(request.validate().is_err());
        }
    }

    #[test]
    fn lifecycle_decision_rejects_fail_open_states() {
        for decision in [
            DataLifecycleDecision {
                permitted: true,
                blockers: vec!["blocked".to_owned()],
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                permitted: false,
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                effects_authorized: true,
                receipt_required: false,
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                permitted: false,
                effects_authorized: true,
                blockers: vec!["blocked".to_owned()],
                receipt_required: true,
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                immutable_audit_preserved: false,
                ..lifecycle_decision()
            },
        ] {
            assert!(decision.validate().is_err());
        }
    }

    #[test]
    fn lifecycle_decision_rejects_invalid_identity() {
        for decision in [
            DataLifecycleDecision {
                schema_version: "unsupported".to_owned(),
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                request_id: String::new(),
                ..lifecycle_decision()
            },
            DataLifecycleDecision {
                policy_id: String::new(),
                ..lifecycle_decision()
            },
        ] {
            assert!(decision.validate().is_err());
        }
    }

    #[test]
    fn lifecycle_effects_digest_is_canonical_and_sensitive() {
        let request = lifecycle_request(DataLifecycleAction::Delete);
        assert_eq!(
            request.effects_digest(),
            "0b361cdaa5b4c39340b159a21fadf9ca32ee3eccb5ec9a7b7269a7969d5873d6"
        );
        let broadened = DataLifecycleRequest {
            target_ids: vec!["record-1".to_owned(), "record-2".to_owned()],
            ..request.clone()
        };
        assert_ne!(request.effects_digest(), broadened.effects_digest());
    }
}
