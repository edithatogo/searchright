//! Deterministic institutional data-governance decisions.

#![forbid(unsafe_code)]

use searchright_contracts::{
    DATA_HANDLING_DECISION_SCHEMA_VERSION, DATA_LIFECYCLE_DECISION_SCHEMA_VERSION,
    DataClassification, DataHandlingDecision, DataHandlingRequest, DataLifecycleAction,
    DataLifecycleDecision, DataLifecycleRequest, DataOperationKind, InstitutionalPolicy,
    LifecycleExecutionMode, Validate,
};

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
        LifecycleApproval {
            approval_id: "approval-1".to_owned(),
            approved_by: "accountable-owner".to_owned(),
            review_id: "review-1".to_owned(),
            action,
            approved_at: "2026-08-13T00:00:00Z".to_owned(),
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
        let decision = evaluate_lifecycle(&policy(), &request)?;
        assert!(!decision.permitted);
        assert!(!decision.effects_authorized);
        assert_eq!(decision.blockers, ["lifecycle.apply.approval_required"]);
        Ok(())
    }

    #[test]
    fn lifecycle_apply_requires_exact_approval_scope() -> Result<(), GovernanceError> {
        let request = DataLifecycleRequest {
            execution_mode: LifecycleExecutionMode::Apply,
            approval: Some(approval(DataLifecycleAction::Retain)),
            ..lifecycle_request(DataLifecycleAction::Export)
        };
        let decision = evaluate_lifecycle(&policy(), &request)?;
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
        let decision = evaluate_lifecycle(&policy(), &request)?;
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
        let decision = evaluate_lifecycle(&policy(), &request)?;
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
        let decision = evaluate_lifecycle(&policy(), &request)?;
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
}
