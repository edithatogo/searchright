//! Deterministic institutional data-governance decisions.

#![forbid(unsafe_code)]

use searchright_contracts::{
    DATA_HANDLING_DECISION_SCHEMA_VERSION, DataClassification, DataHandlingDecision,
    DataHandlingRequest, DataOperationKind, InstitutionalPolicy, Validate,
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
    if matches!(request.operation, DataOperationKind::ExternalModelProcessing)
        && !policy.external_model_processing_allowed
    {
        blockers.push("governance.external_model.denied".to_owned());
    }
    if request.cross_border_transfer && !policy.cross_border_transfer_allowed {
        blockers.push("governance.cross_border_transfer.denied".to_owned());
    }
    if let Some(region) = request.region.as_deref() {
        if !policy.permitted_regions.is_empty()
            && !policy.permitted_regions.iter().any(|allowed| allowed == region)
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
        human_approval_required: !permitted
            || matches!(
                request.classification,
                DataClassification::Confidential
                    | DataClassification::RestrictedFullText
                    | DataClassification::SensitivePersonalData
            ),
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
        DataClassification, DataHandlingRequest, DataOperationKind, DeploymentMode,
        INSTITUTIONAL_POLICY_SCHEMA_VERSION, InstitutionalPolicy,
    };

    use super::*;

    #[test]
    fn restricted_full_text_is_denied_when_persistence_is_disabled() {
        let policy = InstitutionalPolicy {
            schema_version: INSTITUTIONAL_POLICY_SCHEMA_VERSION.to_owned(),
            policy_id: "policy-1".to_owned(),
            institution: "Example institution".to_owned(),
            deployment_modes: vec![DeploymentMode::LocalOnly],
            allowed_classifications: vec![DataClassification::RestrictedFullText],
            permitted_regions: vec!["AU".to_owned()],
            maximum_retention_days: 30,
            telemetry_allowed: false,
            full_text_persistence_allowed: false,
            external_model_processing_allowed: false,
            cross_border_transfer_allowed: false,
            approved_by: "governance officer".to_owned(),
            effective_from: "2026-08-06".to_owned(),
            review_by: Some("2027-08-06".to_owned()),
        };
        let request = DataHandlingRequest {
            schema_version: searchright_contracts::DATA_HANDLING_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "request-1".to_owned(),
            review_id: "review-1".to_owned(),
            classification: DataClassification::RestrictedFullText,
            operation: DataOperationKind::FullTextPersistence,
            deployment_mode: DeploymentMode::LocalOnly,
            region: Some("AU".to_owned()),
            retention_days: 7,
            cross_border_transfer: false,
            dry_run: true,
        };
        let decision = evaluate(&policy, &request);
        assert!(decision.is_ok());
        if let Ok(decision) = decision {
            assert!(!decision.permitted);
            assert!(decision
                .blockers
                .iter()
                .any(|code| code == "governance.full_text_persistence.denied"));
        }
    }
}
