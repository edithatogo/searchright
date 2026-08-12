//! Deterministic institutional data-governance decisions.

#![forbid(unsafe_code)]

mod approval;

pub use approval::{
    ApprovalClock, BoundedLifecycleApprovalRegistry, VerifiedLifecycleApprovalRecord,
};

use searchright_contracts::{
    DATA_HANDLING_DECISION_SCHEMA_VERSION, DATA_LIFECYCLE_DECISION_SCHEMA_VERSION,
    DataClassification, DataHandlingDecision, DataHandlingRequest, DataLifecycleAction,
    DataLifecycleDecision, DataLifecycleRequest, DataOperationKind, InstitutionalPolicy,
    LifecycleExecutionMode, Validate,
};
use sha2::{Digest, Sha256};

/// Durable result returned by a lifecycle-effect sink after an exact apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleEffectReceipt {
    /// Request whose exact effects were applied.
    pub request_id: String,
    /// Store head required before applying the effects.
    pub previous_head: String,
    /// Store head after the atomic effect/tombstone commit.
    pub resulting_head: String,
    /// Lowercase SHA-256 of the durable receipt bytes.
    pub receipt_digest: String,
    /// SHA-256 binding the exact requested effects.
    pub request_digest: String,
    /// SHA-256 binding the exact authorized decision.
    pub decision_digest: String,
}

/// Opaque, verifier-produced authority for one exact lifecycle effect set.
pub struct LifecycleAuthorization {
    request: DataLifecycleRequest,
    decision: DataLifecycleDecision,
}

impl LifecycleAuthorization {
    /// Exact authorised request.
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "avoid creating a public const-evaluation compatibility commitment"
    )]
    pub fn request(&self) -> &DataLifecycleRequest {
        &self.request
    }

    /// Exact policy decision produced after approval verification.
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "avoid creating a public const-evaluation compatibility commitment"
    )]
    pub fn decision(&self) -> &DataLifecycleDecision {
        &self.decision
    }
}

/// Storage boundary capable of atomically applying exact lifecycle effects.
pub trait LifecycleEffectSink {
    /// Current immutable store head used for optimistic concurrency.
    fn current_head(&self) -> Result<String, String>;
    /// Apply exactly the authorised request and durably persist tombstones and receipt.
    fn apply(
        &mut self,
        authorization: &LifecycleAuthorization,
        expected_head: &str,
    ) -> Result<LifecycleEffectReceipt, String>;
}

/// Evaluate a data-handling request against an approved institutional policy.
pub fn evaluate(
    policy: &InstitutionalPolicy,
    request: &DataHandlingRequest,
) -> Result<DataHandlingDecision, GovernanceError> {
    policy.validate()?;
    request.validate()?;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    if !policy.deployment_modes.contains(&request.deployment_mode) {
        blockers.push("governance.deployment_mode.denied".to_owned());
    }
    if !policy
        .allowed_classifications
        .contains(&request.classification)
    {
        blockers.push("governance.classification.denied".to_owned());
    }
    if request.retention_days > policy.maximum_retention_days {
        blockers.push("governance.retention.exceeds_policy".to_owned());
    }
    if matches!(request.operation, DataOperationKind::Telemetry) && !policy.telemetry_allowed {
        blockers.push("governance.telemetry.denied".to_owned());
    }
    if matches!(request.operation, DataOperationKind::FullTextPersistence)
        && !policy.full_text_persistence_allowed
    {
        blockers.push("governance.full_text_persistence.denied".to_owned());
    }
    if matches!(
        request.operation,
        DataOperationKind::ExternalModelProcessing
    ) && !policy.external_model_processing_allowed
    {
        blockers.push("governance.external_model.denied".to_owned());
    }
    if request.cross_border_transfer && !policy.cross_border_transfer_allowed {
        blockers.push("governance.cross_border_transfer.denied".to_owned());
    }
    let requires_human_approval = matches!(
        request.classification,
        DataClassification::Confidential
            | DataClassification::RestrictedFullText
            | DataClassification::SensitivePersonalData
    );
    if requires_human_approval && !request.dry_run {
        blockers.push("governance.human_approval.not_recorded".to_owned());
    }
    if request.deployment_mode == searchright_contracts::DeploymentMode::HostedMultiTenant
        && request.classification != DataClassification::PublicMetadata
    {
        blockers.push("governance.multi_tenant.isolation_unproven".to_owned());
    }
    if matches!(request.operation, DataOperationKind::Export)
        && request.classification != DataClassification::PublicMetadata
        && !request.dry_run
    {
        blockers.push("governance.export.approval_not_recorded".to_owned());
    }
    if let Some(region) = request.region.as_deref() {
        if !policy.permitted_regions.is_empty()
            && !policy
                .permitted_regions
                .iter()
                .any(|allowed| allowed == region)
        {
            blockers.push("governance.region.denied".to_owned());
        }
    } else if !policy.permitted_regions.is_empty() {
        blockers.push("governance.region.required".to_owned());
    }
    if request.classification == DataClassification::SensitivePersonalData {
        warnings.push("governance.sensitive_data.manual_review".to_owned());
    }
    if request.dry_run {
        warnings.push("governance.dry_run.no_effects".to_owned());
    }

    let permitted = blockers.is_empty();
    Ok(DataHandlingDecision {
        schema_version: DATA_HANDLING_DECISION_SCHEMA_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        policy_id: policy.policy_id.clone(),
        permitted,
        blockers,
        warnings,
        human_approval_required: !permitted || requires_human_approval,
    })
}

/// Evaluate a retention, export or deletion request without performing effects.
///
/// Preview is always non-authorising. Apply requires an approval scoped to the
/// exact review and action. Audit-ledger deletion is denied and successful
/// content deletion requires tombstones plus a durable receipt.
pub fn evaluate_lifecycle(
    policy: &InstitutionalPolicy,
    request: &DataLifecycleRequest,
) -> Result<DataLifecycleDecision, GovernanceError> {
    evaluate_lifecycle_inner(policy, request, None)
}

/// Separately verify approval evidence before an apply request can authorise effects.
pub trait LifecycleApprovalVerifier {
    /// Verify identity evidence, expiry and exact request/policy scope.
    fn verify(
        &self,
        approval: &searchright_contracts::LifecycleApproval,
        request_digest: &str,
        policy_id: &str,
    ) -> Result<(), String>;
}

/// Evaluate lifecycle apply with separately supplied approval verification.
pub fn evaluate_lifecycle_with_verifier(
    policy: &InstitutionalPolicy,
    request: &DataLifecycleRequest,
    verifier: &dyn LifecycleApprovalVerifier,
) -> Result<DataLifecycleDecision, GovernanceError> {
    evaluate_lifecycle_inner(policy, request, Some(verifier))
}

/// Produce opaque authority for an exact lifecycle request after approval verification.
pub fn authorize_lifecycle(
    policy: &InstitutionalPolicy,
    request: &DataLifecycleRequest,
    verifier: &dyn LifecycleApprovalVerifier,
) -> Result<LifecycleAuthorization, LifecycleExecutionError> {
    let decision = evaluate_lifecycle_with_verifier(policy, request, verifier)?;
    if !decision.effects_authorized {
        return Err(LifecycleExecutionError::NotAuthorized(decision.blockers));
    }
    Ok(LifecycleAuthorization {
        request: request.clone(),
        decision,
    })
}

/// Verify authority, bind the expected store head, and apply lifecycle effects atomically.
pub fn execute_lifecycle(
    policy: &InstitutionalPolicy,
    request: &DataLifecycleRequest,
    verifier: &dyn LifecycleApprovalVerifier,
    sink: &mut dyn LifecycleEffectSink,
    expected_head: &str,
) -> Result<LifecycleEffectReceipt, LifecycleExecutionError> {
    // The sink owns exact-head and crash-replay semantics. A durable receipt may already have
    // advanced its observable head while an idempotent effect still needs forward completion.
    let authorization = authorize_lifecycle(policy, request, verifier)?;
    let decision = authorization.decision().clone();
    let receipt = sink
        .apply(&authorization, expected_head)
        .map_err(LifecycleExecutionError::Sink)?;
    if receipt.request_id != request.request_id
        || receipt.previous_head != expected_head
        || receipt.resulting_head.trim().is_empty()
        || receipt.receipt_digest.len() != 64
        || !receipt
            .receipt_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(LifecycleExecutionError::InvalidReceipt);
    }
    let request_digest = request.effects_digest();
    let decision_digest =
        lifecycle_decision_digest(&decision).map_err(LifecycleExecutionError::Sink)?;
    if receipt.request_digest != request_digest
        || receipt.decision_digest != decision_digest
        || receipt.resulting_head
            != lifecycle_resulting_head(expected_head, &request_digest, &decision_digest)
    {
        return Err(LifecycleExecutionError::InvalidReceipt);
    }
    Ok(receipt)
}

/// Digest the exact serialized lifecycle decision for durable-effect binding.
pub fn lifecycle_decision_digest(decision: &DataLifecycleDecision) -> Result<String, String> {
    serde_json::to_vec(decision)
        .map(|bytes| hex_digest(&Sha256::digest(bytes)))
        .map_err(|error| error.to_string())
}

/// Derive the lifecycle head from the prior head and exact request/decision digests.
#[must_use]
pub fn lifecycle_resulting_head(
    previous_head: &str,
    request_digest: &str,
    decision_digest: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(previous_head.as_bytes());
    hasher.update([0]);
    hasher.update(request_digest.as_bytes());
    hasher.update([0]);
    hasher.update(decision_digest.as_bytes());
    hex_digest(&hasher.finalize())
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(
            *HEX.get(usize::from(byte >> 4)).unwrap_or(&b'?'),
        ));
        output.push(char::from(
            *HEX.get(usize::from(byte & 0x0f)).unwrap_or(&b'?'),
        ));
    }
    output
}

fn evaluate_lifecycle_inner(
    policy: &InstitutionalPolicy,
    request: &DataLifecycleRequest,
    verifier: Option<&dyn LifecycleApprovalVerifier>,
) -> Result<DataLifecycleDecision, GovernanceError> {
    policy.validate()?;
    request.validate()?;

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    if !policy
        .allowed_classifications
        .contains(&request.classification)
    {
        blockers.push("lifecycle.classification.denied".to_owned());
    }
    if matches!(request.action, DataLifecycleAction::Retain)
        && request.retention_days.unwrap_or_default() > policy.maximum_retention_days
    {
        blockers.push("lifecycle.retention.exceeds_policy".to_owned());
    }
    if matches!(request.action, DataLifecycleAction::Delete) && request.includes_audit_log {
        blockers.push("lifecycle.delete.immutable_audit_denied".to_owned());
    }
    if matches!(request.action, DataLifecycleAction::Delete) && request.legal_hold {
        blockers.push("lifecycle.delete.legal_hold".to_owned());
    }

    let apply = matches!(request.execution_mode, LifecycleExecutionMode::Apply);
    if apply {
        match request.approval.as_ref() {
            None => blockers.push("lifecycle.apply.approval_required".to_owned()),
            Some(approval) => {
                if approval.review_id != request.review_id {
                    blockers.push("lifecycle.apply.approval_review_mismatch".to_owned());
                }
                if approval.action != request.action {
                    blockers.push("lifecycle.apply.approval_action_mismatch".to_owned());
                }
                let digest = request.effects_digest();
                if approval.request_digest != digest {
                    blockers.push("lifecycle.apply.approval_digest_mismatch".to_owned());
                }
                if approval.policy_id != policy.policy_id {
                    blockers.push("lifecycle.apply.approval_policy_mismatch".to_owned());
                }
                match verifier {
                    Some(verifier) => {
                        if verifier
                            .verify(approval, &digest, &policy.policy_id)
                            .is_err()
                        {
                            blockers
                                .push("lifecycle.apply.approval_verification_failed".to_owned());
                        }
                    }
                    None => blockers.push("lifecycle.apply.approval_verifier_required".to_owned()),
                }
            }
        }
    } else {
        warnings.push("lifecycle.preview.no_effects".to_owned());
    }

    if matches!(request.action, DataLifecycleAction::Export) {
        warnings.push("lifecycle.export.destination_review_required".to_owned());
    }
    if matches!(request.action, DataLifecycleAction::Delete) {
        warnings.push("lifecycle.delete.tombstones_required".to_owned());
    }

    let permitted = blockers.is_empty();
    Ok(DataLifecycleDecision {
        schema_version: DATA_LIFECYCLE_DECISION_SCHEMA_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        policy_id: policy.policy_id.clone(),
        permitted,
        effects_authorized: permitted && apply,
        blockers,
        warnings,
        immutable_audit_preserved: true,
        tombstone_target_ids: if matches!(request.action, DataLifecycleAction::Delete) {
            request.target_ids.clone()
        } else {
            Vec::new()
        },
        receipt_required: apply,
    })
}

/// Institutional policy-evaluation failure.
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
}

/// Fail-closed lifecycle effect execution error.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleExecutionError {
    /// Policy or contract validation failed.
    #[error(transparent)]
    Governance(#[from] GovernanceError),
    /// The store changed between preview/approval and apply.
    #[error("lifecycle store head mismatch: expected {expected}, found {actual}")]
    HeadMismatch {
        /// Store head required by the authorised operation.
        expected: String,
        /// Store head observed immediately before apply.
        actual: String,
    },
    /// The request did not carry verified, exact-scope authority.
    #[error("lifecycle effects are not authorised: {0:?}")]
    NotAuthorized(Vec<String>),
    /// The storage boundary failed.
    #[error("lifecycle effect sink failed: {0}")]
    Sink(String),
    /// The storage boundary returned an unbound or malformed receipt.
    #[error("lifecycle effect receipt is not bound to the request and store heads")]
    InvalidReceipt,
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{
        DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION, DataClassification, DataHandlingRequest,
        DataLifecycleAction, DataLifecycleRequest, DataOperationKind, DeploymentMode,
        INSTITUTIONAL_POLICY_SCHEMA_VERSION, InstitutionalPolicy, LifecycleApproval,
        LifecycleExecutionMode,
    };

    use super::*;

    fn policy() -> InstitutionalPolicy {
        InstitutionalPolicy {
            schema_version: INSTITUTIONAL_POLICY_SCHEMA_VERSION.to_owned(),
            policy_id: "policy-1".to_owned(),
            institution: "Example institution".to_owned(),
            deployment_modes: vec![DeploymentMode::LocalOnly, DeploymentMode::HostedMultiTenant],
            allowed_classifications: vec![
                DataClassification::PublicMetadata,
                DataClassification::Confidential,
                DataClassification::RestrictedFullText,
            ],
            permitted_regions: vec!["AU".to_owned()],
            maximum_retention_days: 30,
            telemetry_allowed: false,
            full_text_persistence_allowed: false,
            external_model_processing_allowed: false,
            cross_border_transfer_allowed: false,
            approved_by: "governance officer".to_owned(),
            effective_from: "2026-08-06".to_owned(),
            review_by: Some("2027-08-06".to_owned()),
        }
    }

    fn request() -> DataHandlingRequest {
        DataHandlingRequest {
            schema_version: searchright_contracts::DATA_HANDLING_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "request-1".to_owned(),
            review_id: "review-1".to_owned(),
            classification: DataClassification::PublicMetadata,
            operation: DataOperationKind::Metadata,
            deployment_mode: DeploymentMode::LocalOnly,
            region: Some("AU".to_owned()),
            retention_days: 7,
            cross_border_transfer: false,
            dry_run: true,
        }
    }

    fn lifecycle_request(action: DataLifecycleAction) -> DataLifecycleRequest {
        DataLifecycleRequest {
            schema_version: DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "lifecycle-1".to_owned(),
            review_id: "review-1".to_owned(),
            classification: DataClassification::PublicMetadata,
            action,
            execution_mode: LifecycleExecutionMode::Preview,
            target_ids: vec!["record-1".to_owned(), "record-2".to_owned()],
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
            approved_by: "accountable-owner".to_owned(),
            review_id: "review-1".to_owned(),
            action,
            request_digest: request.effects_digest(),
            policy_id: "policy-1".to_owned(),
            nonce: "nonce-1".to_owned(),
            approved_at: "2026-08-13T00:00:00Z".to_owned(),
            expires_at: "2026-08-14T00:00:00Z".to_owned(),
        }
    }

    struct AcceptVerifier;

    impl LifecycleApprovalVerifier for AcceptVerifier {
        fn verify(
            &self,
            approval: &LifecycleApproval,
            request_digest: &str,
            policy_id: &str,
        ) -> Result<(), String> {
            if approval.nonce == "nonce-1"
                && approval.expires_at == "2026-08-14T00:00:00Z"
                && approval.request_digest == request_digest
                && approval.policy_id == policy_id
            {
                Ok(())
            } else {
                Err("approval rejected".to_owned())
            }
        }
    }

    #[test]
    fn restricted_full_text_is_denied_when_persistence_is_disabled() {
        let policy = policy();
        let request = DataHandlingRequest {
            classification: DataClassification::RestrictedFullText,
            operation: DataOperationKind::FullTextPersistence,
            ..request()
        };
        let decision = evaluate(&policy, &request);
        assert!(decision.is_ok());
        if let Ok(decision) = decision {
            assert!(!decision.permitted);
            assert!(
                decision
                    .blockers
                    .iter()
                    .any(|code| code == "governance.full_text_persistence.denied")
            );
        }
    }

    #[test]
    fn high_risk_apply_is_denied_without_recorded_human_approval() -> Result<(), GovernanceError> {
        let request = DataHandlingRequest {
            classification: DataClassification::Confidential,
            dry_run: false,
            ..request()
        };

        let decision = evaluate(&policy(), &request)?;

        assert!(!decision.permitted);
        assert!(decision.human_approval_required);
        assert_eq!(
            decision.blockers,
            ["governance.human_approval.not_recorded"]
        );
        Ok(())
    }

    #[test]
    fn dry_run_can_preview_high_risk_request_without_authorising_effects()
    -> Result<(), GovernanceError> {
        let request = DataHandlingRequest {
            classification: DataClassification::Confidential,
            ..request()
        };

        let decision = evaluate(&policy(), &request)?;

        assert!(decision.permitted);
        assert!(decision.human_approval_required);
        assert_eq!(decision.warnings, ["governance.dry_run.no_effects"]);
        Ok(())
    }

    #[test]
    fn multi_tenant_non_public_data_is_denied_without_isolation_evidence()
    -> Result<(), GovernanceError> {
        let request = DataHandlingRequest {
            classification: DataClassification::Confidential,
            deployment_mode: DeploymentMode::HostedMultiTenant,
            ..request()
        };

        let decision = evaluate(&policy(), &request)?;

        assert!(!decision.permitted);
        assert!(
            decision
                .blockers
                .contains(&"governance.multi_tenant.isolation_unproven".to_owned())
        );
        Ok(())
    }

    #[test]
    fn non_public_export_apply_requires_separate_approval_record() -> Result<(), GovernanceError> {
        let request = DataHandlingRequest {
            classification: DataClassification::Confidential,
            operation: DataOperationKind::Export,
            dry_run: false,
            ..request()
        };

        let decision = evaluate(&policy(), &request)?;

        assert!(!decision.permitted);
        assert!(
            decision
                .blockers
                .contains(&"governance.export.approval_not_recorded".to_owned())
        );
        Ok(())
    }

    #[test]
    fn policy_limit_failures_are_reported_in_deterministic_order() -> Result<(), GovernanceError> {
        let request = DataHandlingRequest {
            classification: DataClassification::RestrictedFullText,
            operation: DataOperationKind::FullTextPersistence,
            deployment_mode: DeploymentMode::HostedMultiTenant,
            region: Some("NZ".to_owned()),
            retention_days: 31,
            cross_border_transfer: true,
            dry_run: false,
            ..request()
        };

        let first = evaluate(&policy(), &request)?;
        let second = evaluate(&policy(), &request)?;

        assert_eq!(first, second);
        assert_eq!(
            first.blockers,
            [
                "governance.retention.exceeds_policy",
                "governance.full_text_persistence.denied",
                "governance.cross_border_transfer.denied",
                "governance.human_approval.not_recorded",
                "governance.multi_tenant.isolation_unproven",
                "governance.region.denied",
            ]
        );
        Ok(())
    }

    #[test]
    fn lifecycle_preview_never_authorises_effects() -> Result<(), GovernanceError> {
        for action in [
            DataLifecycleAction::Retain,
            DataLifecycleAction::Export,
            DataLifecycleAction::Delete,
        ] {
            let decision = evaluate_lifecycle(&policy(), &lifecycle_request(action))?;
            assert!(decision.permitted);
            assert!(!decision.effects_authorized);
            assert!(!decision.receipt_required);
            assert!(decision.immutable_audit_preserved);
        }
        Ok(())
    }

    #[test]
    fn lifecycle_apply_fails_closed_without_scoped_approval() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(!decision.permitted);
        assert!(!decision.effects_authorized);
        assert_eq!(decision.blockers, ["lifecycle.apply.approval_required"]);
        Ok(())
    }

    #[test]
    fn lifecycle_apply_requires_exact_approval_scope() -> Result<(), GovernanceError> {
        let mut mismatched = approval(DataLifecycleAction::Export);
        mismatched.action = DataLifecycleAction::Retain;
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(mismatched),
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert_eq!(
            decision.blockers,
            ["lifecycle.apply.approval_action_mismatch"]
        );
        Ok(())
    }

    #[test]
    fn approved_delete_requires_receipt_and_tombstones() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(approval(DataLifecycleAction::Delete)),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(decision.permitted);
        assert!(decision.effects_authorized);
        assert!(decision.receipt_required);
        assert_eq!(decision.tombstone_target_ids, ["record-1", "record-2"]);
        Ok(())
    }

    #[test]
    fn audit_ledger_deletion_is_never_permitted() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            includes_audit_log: true,
            approval: Some(approval(DataLifecycleAction::Delete)),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(!decision.permitted);
        assert!(!decision.effects_authorized);
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.delete.immutable_audit_denied".to_owned())
        );
        Ok(())
    }

    #[test]
    fn legal_hold_denies_deletion_even_with_approval() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            legal_hold: true,
            approval: Some(approval(DataLifecycleAction::Delete)),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(!decision.permitted);
        assert!(!decision.effects_authorized);
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.delete.legal_hold".to_owned())
        );
        Ok(())
    }

    #[test]
    fn duplicate_lifecycle_targets_are_rejected() {
        let request = DataLifecycleRequest {
            target_ids: vec!["record-1".to_owned(), "record-1".to_owned()],
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn lifecycle_policy_denials_are_deterministic() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            classification: DataClassification::SensitivePersonalData,
            retention_days: Some(31),
            ..lifecycle_request(DataLifecycleAction::Retain)
        };

        let first = evaluate_lifecycle(&policy(), &request)?;
        let second = evaluate_lifecycle(&policy(), &request)?;

        assert_eq!(first, second);
        assert_eq!(
            first.blockers,
            [
                "lifecycle.classification.denied",
                "lifecycle.retention.exceeds_policy",
            ]
        );
        assert!(!first.effects_authorized);
        Ok(())
    }

    #[test]
    fn lifecycle_apply_rejects_approval_for_another_review() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(LifecycleApproval {
                review_id: "review-other".to_owned(),
                ..approval(DataLifecycleAction::Export)
            }),
            ..lifecycle_request(DataLifecycleAction::Export)
        };

        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;

        assert_eq!(
            decision.blockers,
            ["lifecycle.apply.approval_review_mismatch"]
        );
        assert!(decision.receipt_required);
        assert!(!decision.effects_authorized);
        Ok(())
    }

    #[test]
    fn approved_retain_and_export_authorise_effects_without_tombstones()
    -> Result<(), GovernanceError> {
        for action in [DataLifecycleAction::Retain, DataLifecycleAction::Export] {
            let request = DataLifecycleRequest {
                execution_mode: LifecycleExecutionMode::Apply,
                approval: Some(approval(action)),
                ..lifecycle_request(action)
            };
            let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
            assert!(decision.permitted);
            assert!(decision.effects_authorized);
            assert!(decision.receipt_required);
            assert!(decision.tombstone_target_ids.is_empty());
        }
        Ok(())
    }

    #[test]
    fn plain_lifecycle_evaluation_never_self_authorises_apply() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(approval(DataLifecycleAction::Export)),
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle(&policy(), &request)?;
        assert!(!decision.effects_authorized);
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.apply.approval_verifier_required".to_owned())
        );
        Ok(())
    }

    #[test]
    fn approval_digest_prevents_effect_broadening_and_policy_reuse() -> Result<(), GovernanceError>
    {
        let base = lifecycle_request(DataLifecycleAction::Retain);
        let approved = approval(DataLifecycleAction::Retain);
        for broadened in [
            DataLifecycleRequest {
                target_ids: vec!["record-1".to_owned(), "record-3".to_owned()],
                ..base.clone()
            },
            DataLifecycleRequest {
                retention_days: Some(8),
                ..base
            },
        ] {
            let request = DataLifecycleRequest {
                execution_mode: LifecycleExecutionMode::Apply,
                approval: Some(approved.clone()),
                ..broadened
            };
            let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
            assert!(
                decision
                    .blockers
                    .contains(&"lifecycle.apply.approval_digest_mismatch".to_owned())
            );
        }

        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(LifecycleApproval {
                policy_id: "policy-other".to_owned(),
                ..approval(DataLifecycleAction::Export)
            }),
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.apply.approval_policy_mismatch".to_owned())
        );
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.apply.approval_verification_failed".to_owned())
        );
        Ok(())
    }

    #[test]
    fn approval_digest_binds_export_destination() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            export_destination: Some("file:///broadened/export.srpack".to_owned()),
            approval: Some(approval(DataLifecycleAction::Export)),
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &AcceptVerifier)?;
        assert!(
            decision
                .blockers
                .contains(&"lifecycle.apply.approval_digest_mismatch".to_owned())
        );
        Ok(())
    }

    #[test]
    fn verifier_denies_expired_or_replayed_approval() -> Result<(), GovernanceError> {
        struct RejectVerifier;
        impl LifecycleApprovalVerifier for RejectVerifier {
            fn verify(
                &self,
                _approval: &LifecycleApproval,
                _request_digest: &str,
                _policy_id: &str,
            ) -> Result<(), String> {
                Err("expired or nonce already consumed".to_owned())
            }
        }

        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(approval(DataLifecycleAction::Delete)),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        let decision = evaluate_lifecycle_with_verifier(&policy(), &request, &RejectVerifier)?;
        assert_eq!(
            decision.blockers,
            ["lifecycle.apply.approval_verification_failed"]
        );
        assert!(!decision.effects_authorized);
        Ok(())
    }

    #[test]
    fn lifecycle_evaluation_propagates_contract_validation_failure() {
        let request = DataLifecycleRequest {
            target_ids: Vec::new(),
            ..lifecycle_request(DataLifecycleAction::Delete)
        };

        assert!(matches!(
            evaluate_lifecycle(&policy(), &request),
            Err(GovernanceError::Contract(_))
        ));
    }

    #[test]
    fn data_handling_covers_default_deny_operation_and_region_branches()
    -> Result<(), GovernanceError> {
        let denied_policy = InstitutionalPolicy {
            deployment_modes: vec![DeploymentMode::InstitutionSelfHosted],
            allowed_classifications: vec![DataClassification::PublicMetadata],
            permitted_regions: vec![],
            ..policy()
        };
        let denied_request = DataHandlingRequest {
            classification: DataClassification::SensitivePersonalData,
            operation: DataOperationKind::Telemetry,
            region: None,
            ..request()
        };
        let decision = evaluate(&denied_policy, &denied_request)?;
        assert_eq!(
            decision.blockers,
            [
                "governance.deployment_mode.denied",
                "governance.classification.denied",
                "governance.telemetry.denied",
            ]
        );
        assert_eq!(
            decision.warnings,
            [
                "governance.sensitive_data.manual_review",
                "governance.dry_run.no_effects",
            ]
        );

        let external_model = DataHandlingRequest {
            operation: DataOperationKind::ExternalModelProcessing,
            region: None,
            ..request()
        };
        let decision = evaluate(&policy(), &external_model)?;
        assert_eq!(
            decision.blockers,
            [
                "governance.external_model.denied",
                "governance.region.required",
            ]
        );
        Ok(())
    }

    #[test]
    fn data_handling_evaluation_propagates_invalid_request() {
        let request = DataHandlingRequest {
            retention_days: 0,
            ..request()
        };
        assert!(matches!(
            evaluate(&policy(), &request),
            Err(GovernanceError::Contract(_))
        ));
    }

    struct MemorySink {
        head: String,
        apply_count: usize,
    }

    impl LifecycleEffectSink for MemorySink {
        fn current_head(&self) -> Result<String, String> {
            Ok(self.head.clone())
        }

        fn apply(
            &mut self,
            authorization: &LifecycleAuthorization,
            expected_head: &str,
        ) -> Result<LifecycleEffectReceipt, String> {
            let request = authorization.request();
            let decision = authorization.decision();
            if !decision.effects_authorized {
                return Err("unauthorised decision".to_owned());
            }
            self.apply_count += 1;
            let request_digest = request.effects_digest();
            let decision_digest = lifecycle_decision_digest(decision)?;
            self.head = lifecycle_resulting_head(expected_head, &request_digest, &decision_digest);
            Ok(LifecycleEffectReceipt {
                request_id: request.request_id.clone(),
                previous_head: expected_head.to_owned(),
                resulting_head: self.head.clone(),
                receipt_digest: "a".repeat(64),
                request_digest,
                decision_digest,
            })
        }
    }

    #[test]
    fn lifecycle_executor_binds_authority_and_expected_head() -> Result<(), LifecycleExecutionError>
    {
        let action = DataLifecycleAction::Delete;
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(approval(action)),
            ..lifecycle_request(action)
        };
        let mut sink = MemorySink {
            head: "head-0".to_owned(),
            apply_count: 0,
        };
        let receipt = execute_lifecycle(&policy(), &request, &AcceptVerifier, &mut sink, "head-0")?;
        assert_eq!(receipt.request_id, request.request_id);
        assert_eq!(sink.apply_count, 1);

        let replay = execute_lifecycle(&policy(), &request, &AcceptVerifier, &mut sink, "head-0")?;
        assert_eq!(replay, receipt);
        assert_eq!(sink.apply_count, 2);
        Ok(())
    }

    #[test]
    fn lifecycle_executor_never_calls_sink_without_verified_authority() {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: None,
            ..lifecycle_request(DataLifecycleAction::Delete)
        };
        let mut sink = MemorySink {
            head: "head-0".to_owned(),
            apply_count: 0,
        };
        assert!(matches!(
            execute_lifecycle(&policy(), &request, &AcceptVerifier, &mut sink, "head-0"),
            Err(LifecycleExecutionError::NotAuthorized(_))
        ));
        assert_eq!(sink.apply_count, 0);
    }
}
