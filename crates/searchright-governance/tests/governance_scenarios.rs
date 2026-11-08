//! Scenario and adversarial integration tests for `searchright-governance`.
//!
//! Validates:
//! 1. Deny-by-default enforcement across deployment mode, classification, retention,
//!    telemetry, external models, cross-border transfers and regions.
//! 2. Human approval requirements for high-risk data classifications.
//! 3. Lifecycle authority scoping, audit ledger immutability and legal hold enforcement.
//! 4. Optimistic concurrency, store head bindings and verified execution sink contracts.
//! 5. Bounded lifecycle approval registry validation, clock boundaries and fail-closed verification.

use searchright_contracts::{
    DATA_HANDLING_REQUEST_SCHEMA_VERSION, DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION,
    DataClassification, DataHandlingRequest, DataLifecycleAction, DataLifecycleRequest,
    DataOperationKind, DeploymentMode, InstitutionalPolicy, LifecycleApproval,
    LifecycleExecutionMode,
};
use searchright_governance::{
    ApprovalClock, BoundedLifecycleApprovalRegistry, GovernanceError, LifecycleApprovalVerifier,
    LifecycleAuthorization, LifecycleEffectReceipt, LifecycleEffectSink, LifecycleExecutionError,
    VerifiedLifecycleApprovalRecord, evaluate, evaluate_lifecycle,
    evaluate_lifecycle_with_verifier, execute_lifecycle, lifecycle_decision_digest,
    lifecycle_resulting_head,
};

fn standard_policy() -> InstitutionalPolicy {
    InstitutionalPolicy {
        schema_version: "org.searchright.institutional-policy.v1".to_owned(),
        policy_id: "pol-curtin-health-2026".to_owned(),
        institution: "Curtin University".to_owned(),
        deployment_modes: vec![
            DeploymentMode::LocalOnly,
            DeploymentMode::InstitutionSelfHosted,
        ],
        allowed_classifications: vec![
            DataClassification::PublicMetadata,
            DataClassification::InternalReviewData,
            DataClassification::Confidential,
        ],
        permitted_regions: vec!["AU".to_owned()],
        maximum_retention_days: 90,
        telemetry_allowed: false,
        full_text_persistence_allowed: false,
        external_model_processing_allowed: false,
        cross_border_transfer_allowed: false,
        approved_by: "governance-office@curtin.edu.au".to_owned(),
        effective_from: "2026-01-01T00:00:00Z".to_owned(),
        review_by: Some("2027-01-01T00:00:00Z".to_owned()),
    }
}

fn standard_request() -> DataHandlingRequest {
    DataHandlingRequest {
        schema_version: DATA_HANDLING_REQUEST_SCHEMA_VERSION.to_owned(),
        request_id: "req-screening-001".to_owned(),
        review_id: "rev-long-covid-2026".to_owned(),
        classification: DataClassification::InternalReviewData,
        operation: DataOperationKind::Metadata,
        deployment_mode: DeploymentMode::LocalOnly,
        region: Some("AU".to_owned()),
        retention_days: 30,
        cross_border_transfer: false,
        dry_run: false,
    }
}

fn standard_lifecycle_request(action: DataLifecycleAction) -> DataLifecycleRequest {
    DataLifecycleRequest {
        schema_version: DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION.to_owned(),
        request_id: "req-lifecycle-001".to_owned(),
        review_id: "rev-long-covid-2026".to_owned(),
        classification: DataClassification::InternalReviewData,
        action,
        execution_mode: LifecycleExecutionMode::Preview,
        target_ids: vec!["doc-001".to_owned(), "doc-002".to_owned()],
        retention_days: if matches!(action, DataLifecycleAction::Retain) {
            Some(60)
        } else {
            None
        },
        export_destination: if matches!(action, DataLifecycleAction::Export) {
            Some("curtin-secure-vault://exports/2026".to_owned())
        } else {
            None
        },
        includes_audit_log: false,
        legal_hold: false,
        approval: None,
    }
}

fn valid_approval_for(
    request: &DataLifecycleRequest,
    policy_id: &str,
    approved_by: &str,
) -> LifecycleApproval {
    LifecycleApproval {
        approval_id: "appr-lifecycle-001".to_owned(),
        approved_by: approved_by.to_owned(),
        review_id: request.review_id.clone(),
        action: request.action,
        request_digest: request.effects_digest(),
        policy_id: policy_id.to_owned(),
        nonce: "nonce-random-fixed-001".to_owned(),
        approved_at: "2026-06-01T12:00:00Z".to_owned(),
        expires_at: "2026-06-02T12:00:00Z".to_owned(),
    }
}

struct StaticClock(String);

impl ApprovalClock for StaticClock {
    fn now_utc(&self) -> String {
        self.0.clone()
    }
}

struct AlwaysAcceptVerifier;

impl LifecycleApprovalVerifier for AlwaysAcceptVerifier {
    fn verify(
        &self,
        _approval: &LifecycleApproval,
        _request_digest: &str,
        _policy_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

struct MemorySink {
    head: String,
    applied_count: usize,
    should_fail: bool,
}

impl MemorySink {
    fn new(initial_head: &str) -> Self {
        Self {
            head: initial_head.to_owned(),
            applied_count: 0,
            should_fail: false,
        }
    }
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
        if self.should_fail {
            return Err("simulated disk I/O failure on sink apply".to_owned());
        }
        if self.head != expected_head {
            return Err(format!(
                "head mismatch in sink: current={}, expected={}",
                self.head, expected_head
            ));
        }
        let request = authorization.request();
        let decision = authorization.decision();
        if !decision.effects_authorized {
            return Err("cannot apply unauthorized decision".to_owned());
        }
        self.applied_count += 1;
        let request_digest = request.effects_digest();
        let decision_digest =
            lifecycle_decision_digest(decision).map_err(|e| format!("decision digest: {e}"))?;
        let resulting_head =
            lifecycle_resulting_head(expected_head, &request_digest, &decision_digest);
        self.head.clone_from(&resulting_head);
        Ok(LifecycleEffectReceipt {
            request_id: request.request_id.clone(),
            previous_head: expected_head.to_owned(),
            resulting_head,
            receipt_digest: "b".repeat(64),
            request_digest,
            decision_digest,
        })
    }
}

// ---------------------------------------------------------------------------
// 1. Data handling deny-by-default and policy limit scenarios
// ---------------------------------------------------------------------------

#[test]
fn test_unapproved_deployment_mode_is_blocked() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataHandlingRequest {
        deployment_mode: DeploymentMode::HostedMultiTenant,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.deployment_mode.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_unapproved_classification_is_blocked() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataHandlingRequest {
        classification: DataClassification::SensitivePersonalData,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.classification.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_retention_days_exceeding_policy_is_blocked() -> Result<(), GovernanceError> {
    let policy = standard_policy(); // max 90 days
    let request = DataHandlingRequest {
        retention_days: 120,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.retention.exceeds_policy".to_owned())
    );
    Ok(())
}

#[test]
fn test_telemetry_denied_when_policy_forbids() -> Result<(), GovernanceError> {
    let policy = standard_policy(); // telemetry_allowed = false
    let request = DataHandlingRequest {
        operation: DataOperationKind::Telemetry,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.telemetry.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_external_model_processing_denied_when_policy_forbids() -> Result<(), GovernanceError> {
    let policy = standard_policy(); // external_model_processing_allowed = false
    let request = DataHandlingRequest {
        operation: DataOperationKind::ExternalModelProcessing,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.external_model.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_cross_border_transfer_denied_when_policy_forbids() -> Result<(), GovernanceError> {
    let policy = standard_policy(); // cross_border_transfer_allowed = false
    let request = DataHandlingRequest {
        cross_border_transfer: true,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.cross_border_transfer.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_unapproved_region_is_blocked() -> Result<(), GovernanceError> {
    let policy = standard_policy(); // permitted: ["AU"]
    let request = DataHandlingRequest {
        region: Some("US".to_owned()),
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.region.denied".to_owned())
    );
    Ok(())
}

#[test]
fn test_missing_region_is_blocked_when_policy_constrains_regions() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataHandlingRequest {
        region: None,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(
        decision
            .blockers
            .contains(&"governance.region.required".to_owned())
    );
    Ok(())
}

#[test]
fn test_high_risk_classification_mandates_human_approval() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataHandlingRequest {
        classification: DataClassification::Confidential,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(decision.human_approval_required);
    assert!(
        decision
            .blockers
            .contains(&"governance.human_approval.not_recorded".to_owned())
    );
    Ok(())
}

#[test]
fn test_dry_run_records_warning_and_does_not_authorize_effects() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataHandlingRequest {
        dry_run: true,
        ..standard_request()
    };
    let decision = evaluate(&policy, &request)?;
    assert!(decision.permitted);
    assert!(
        decision
            .warnings
            .contains(&"governance.dry_run.no_effects".to_owned())
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Lifecycle authority scoping and immutability scenarios
// ---------------------------------------------------------------------------

#[test]
fn test_lifecycle_preview_never_authorises_effects() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let decision = evaluate_lifecycle(&policy, &request)?;
    assert!(decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .warnings
            .contains(&"lifecycle.preview.no_effects".to_owned())
    );
    Ok(())
}

#[test]
fn test_lifecycle_apply_fails_closed_without_approval() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let request = DataLifecycleRequest {
        execution_mode: LifecycleExecutionMode::Apply,
        approval: None,
        ..standard_lifecycle_request(DataLifecycleAction::Delete)
    };
    let decision = evaluate_lifecycle(&policy, &request)?;
    assert!(!decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .blockers
            .contains(&"lifecycle.apply.approval_required".to_owned())
    );
    Ok(())
}

#[test]
fn test_audit_log_deletion_is_permanently_forbidden() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.includes_audit_log = true;
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
    assert!(!decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .blockers
            .contains(&"lifecycle.delete.immutable_audit_denied".to_owned())
    );
    assert!(decision.immutable_audit_preserved);
    Ok(())
}

#[test]
fn test_legal_hold_blocks_deletion_even_with_valid_approval() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.legal_hold = true;
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
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
fn test_mismatched_review_id_scope_is_rejected() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    let mut approval = valid_approval_for(&request, &policy.policy_id, &policy.approved_by);
    approval.review_id = "rev-unrelated-study-999".to_owned();
    request.approval = Some(approval);

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
    assert!(!decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .blockers
            .contains(&"lifecycle.apply.approval_review_mismatch".to_owned())
    );
    Ok(())
}

#[test]
fn test_mismatched_action_in_approval_is_rejected() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    let mut approval = valid_approval_for(&request, &policy.policy_id, &policy.approved_by);
    approval.action = DataLifecycleAction::Retain;
    request.approval = Some(approval);

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
    assert!(!decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .blockers
            .contains(&"lifecycle.apply.approval_action_mismatch".to_owned())
    );
    Ok(())
}

#[test]
fn test_mismatched_request_digest_is_rejected() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    let mut approval = valid_approval_for(&request, &policy.policy_id, &policy.approved_by);
    approval.request_digest = "f".repeat(64);
    request.approval = Some(approval);

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
    assert!(!decision.permitted);
    assert!(!decision.effects_authorized);
    assert!(
        decision
            .blockers
            .contains(&"lifecycle.apply.approval_digest_mismatch".to_owned())
    );
    Ok(())
}

#[test]
fn test_delete_action_authorizes_tombstones_and_requires_receipt() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
    assert!(decision.permitted);
    assert!(decision.effects_authorized);
    assert!(decision.receipt_required);
    assert_eq!(decision.tombstone_target_ids, request.target_ids);
    Ok(())
}

#[test]
fn test_retain_and_export_do_not_generate_tombstones() -> Result<(), GovernanceError> {
    let policy = standard_policy();
    for action in [DataLifecycleAction::Retain, DataLifecycleAction::Export] {
        let mut request = standard_lifecycle_request(action);
        request.execution_mode = LifecycleExecutionMode::Apply;
        request.approval = Some(valid_approval_for(
            &request,
            &policy.policy_id,
            &policy.approved_by,
        ));

        let decision = evaluate_lifecycle_with_verifier(&policy, &request, &AlwaysAcceptVerifier)?;
        assert!(decision.permitted);
        assert!(decision.effects_authorized);
        assert!(decision.receipt_required);
        assert!(decision.tombstone_target_ids.is_empty());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Execution sink, head bindings and optimistic concurrency
// ---------------------------------------------------------------------------

#[test]
fn test_execute_lifecycle_updates_store_head_and_produces_receipt()
-> Result<(), LifecycleExecutionError> {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let mut sink = MemorySink::new("store-head-epoch-1");
    let receipt = execute_lifecycle(
        &policy,
        &request,
        &AlwaysAcceptVerifier,
        &mut sink,
        "store-head-epoch-1",
    )?;

    assert_eq!(receipt.request_id, request.request_id);
    assert_eq!(receipt.previous_head, "store-head-epoch-1");
    assert_eq!(sink.head, receipt.resulting_head);
    assert_eq!(sink.applied_count, 1);
    Ok(())
}

#[test]
fn test_execute_lifecycle_detects_optimistic_head_mismatch() {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let mut sink = MemorySink::new("store-head-current");
    let result = execute_lifecycle(
        &policy,
        &request,
        &AlwaysAcceptVerifier,
        &mut sink,
        "store-head-stale",
    );

    assert!(matches!(result, Err(LifecycleExecutionError::Sink(_))));
    assert_eq!(sink.applied_count, 0);
}

#[test]
fn test_execute_lifecycle_never_invokes_sink_for_unauthorized_request() {
    let policy = standard_policy();
    let request = DataLifecycleRequest {
        execution_mode: LifecycleExecutionMode::Apply,
        approval: None,
        ..standard_lifecycle_request(DataLifecycleAction::Delete)
    };

    let mut sink = MemorySink::new("store-head-1");
    let result = execute_lifecycle(
        &policy,
        &request,
        &AlwaysAcceptVerifier,
        &mut sink,
        "store-head-1",
    );

    assert!(matches!(
        result,
        Err(LifecycleExecutionError::NotAuthorized(_))
    ));
    assert_eq!(sink.applied_count, 0);
}

#[test]
fn test_execute_lifecycle_propagates_sink_failure() {
    let policy = standard_policy();
    let mut request = standard_lifecycle_request(DataLifecycleAction::Delete);
    request.execution_mode = LifecycleExecutionMode::Apply;
    request.approval = Some(valid_approval_for(
        &request,
        &policy.policy_id,
        &policy.approved_by,
    ));

    let mut sink = MemorySink::new("store-head-1");
    sink.should_fail = true;

    let result = execute_lifecycle(
        &policy,
        &request,
        &AlwaysAcceptVerifier,
        &mut sink,
        "store-head-1",
    );

    assert!(matches!(result, Err(LifecycleExecutionError::Sink(_))));
}

// ---------------------------------------------------------------------------
// 4. Bounded lifecycle approval registry scenarios
// ---------------------------------------------------------------------------

#[test]
fn test_approval_registry_rejects_unverified_evidence() {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let approval = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");
    let record = VerifiedLifecycleApprovalRecord {
        approval,
        authority_evidence_ref: "did:key:12345".to_owned(),
        authority_evidence_verified: false, // NOT verified
    };

    let clock = StaticClock("2026-06-01T15:00:00Z".to_owned());
    let result = BoundedLifecycleApprovalRegistry::new(vec![record], 10, clock);
    assert!(result.is_err());
}

#[test]
fn test_approval_registry_rejects_empty_evidence_ref() {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let approval = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");
    let record = VerifiedLifecycleApprovalRecord {
        approval,
        authority_evidence_ref: "   ".to_owned(), // empty
        authority_evidence_verified: true,
    };

    let clock = StaticClock("2026-06-01T15:00:00Z".to_owned());
    let result = BoundedLifecycleApprovalRegistry::new(vec![record], 10, clock);
    assert!(result.is_err());
}

#[test]
fn test_approval_registry_rejects_inverted_or_malformed_expiry() {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let mut approval = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");
    // Expiry before approval time
    approval.approved_at = "2026-06-02T12:00:00Z".to_owned();
    approval.expires_at = "2026-06-01T12:00:00Z".to_owned();

    let record = VerifiedLifecycleApprovalRecord {
        approval,
        authority_evidence_ref: "sig-record-001".to_owned(),
        authority_evidence_verified: true,
    };

    let clock = StaticClock("2026-06-01T15:00:00Z".to_owned());
    let result = BoundedLifecycleApprovalRegistry::new(vec![record], 10, clock);
    assert!(result.is_err());
}

#[test]
fn test_approval_registry_rejects_capacity_overflow() {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let approval1 = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");
    let mut approval2 = approval1.clone();
    approval2.approval_id = "appr-lifecycle-002".to_owned();

    let records = vec![
        VerifiedLifecycleApprovalRecord {
            approval: approval1,
            authority_evidence_ref: "sig-1".to_owned(),
            authority_evidence_verified: true,
        },
        VerifiedLifecycleApprovalRecord {
            approval: approval2,
            authority_evidence_ref: "sig-2".to_owned(),
            authority_evidence_verified: true,
        },
    ];

    let clock = StaticClock("2026-06-01T15:00:00Z".to_owned());
    // Max capacity 1, but supplied 2 records
    let result = BoundedLifecycleApprovalRegistry::new(records, 1, clock);
    assert!(result.is_err());
}

#[test]
fn test_approval_registry_rejects_duplicate_approval_ids() {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let approval = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");

    let records = vec![
        VerifiedLifecycleApprovalRecord {
            approval: approval.clone(),
            authority_evidence_ref: "sig-1".to_owned(),
            authority_evidence_verified: true,
        },
        VerifiedLifecycleApprovalRecord {
            approval,
            authority_evidence_ref: "sig-2".to_owned(),
            authority_evidence_verified: true,
        },
    ];

    let clock = StaticClock("2026-06-01T15:00:00Z".to_owned());
    let result = BoundedLifecycleApprovalRegistry::new(records, 10, clock);
    assert!(result.is_err());
}

#[test]
fn test_approval_registry_verifier_enforces_clock_window() -> Result<(), String> {
    let request = standard_lifecycle_request(DataLifecycleAction::Delete);
    let approval = valid_approval_for(&request, "pol-001", "admin@curtin.edu.au");
    // Approved: 2026-06-01T12:00:00Z, Expires: 2026-06-02T12:00:00Z

    let record = VerifiedLifecycleApprovalRecord {
        approval: approval.clone(),
        authority_evidence_ref: "sig-1".to_owned(),
        authority_evidence_verified: true,
    };

    // Case 1: Active within window -> Passes
    let active_clock = StaticClock("2026-06-01T18:00:00Z".to_owned());
    let registry = BoundedLifecycleApprovalRegistry::new(vec![record.clone()], 10, active_clock)?;
    assert!(
        registry
            .verify(&approval, &request.effects_digest(), "pol-001")
            .is_ok()
    );

    // Case 2: Past expiry -> Fails closed
    let expired_clock = StaticClock("2026-06-02T15:00:00Z".to_owned());
    let registry = BoundedLifecycleApprovalRegistry::new(vec![record.clone()], 10, expired_clock)?;
    assert!(
        registry
            .verify(&approval, &request.effects_digest(), "pol-001")
            .is_err()
    );

    // Case 3: Future approval (now < approved_at) -> Fails closed
    let future_clock = StaticClock("2026-06-01T08:00:00Z".to_owned());
    let registry = BoundedLifecycleApprovalRegistry::new(vec![record], 10, future_clock)?;
    assert!(
        registry
            .verify(&approval, &request.effects_digest(), "pol-001")
            .is_err()
    );
    Ok(())
}
