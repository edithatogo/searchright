//! Lifecycle policy edge cases.

use searchright_assurance::{
    AssuranceError, allowed_transition, requires_human_approval, verify_trace, verify_transition,
};
use searchright_contracts::{
    LifecycleStage, LifecycleTransition, TransitionActorKind, WORKFLOW_TRACE_SCHEMA_VERSION,
    WorkflowTrace,
};

fn transition(
    from: LifecycleStage,
    to: LifecycleStage,
    kind: TransitionActorKind,
) -> LifecycleTransition {
    LifecycleTransition {
        transition_id: format!("t-{from:?}-{to:?}"),
        from,
        to,
        actor_kind: kind,
        actor_id: "reviewer".to_string(),
        occurred_at: "2026-08-06T00:00:00Z".to_string(),
        evidence_ids: vec!["evidence-1".to_string()],
        approved: true,
    }
}

fn trace(transitions: Vec<LifecycleTransition>) -> WorkflowTrace {
    WorkflowTrace {
        schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_string(),
        review_id: "review-1".to_string(),
        initial_stage: LifecycleStage::Draft,
        transitions,
    }
}

#[test]
fn draft_to_plan_approved_is_allowed_and_human_gated() {
    assert!(allowed_transition(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved
    ));
    assert!(requires_human_approval(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved
    ));
}

#[test]
fn draft_to_reported_skip_is_denied() {
    assert!(!allowed_transition(
        LifecycleStage::Draft,
        LifecycleStage::Reported
    ));
    assert!(matches!(
        verify_transition(&transition(
            LifecycleStage::Draft,
            LifecycleStage::Reported,
            TransitionActorKind::Human
        )),
        Err(AssuranceError::TransitionDenied { .. })
    ));
}

#[test]
fn self_transition_fails_contract() {
    assert!(matches!(
        verify_transition(&transition(
            LifecycleStage::Draft,
            LifecycleStage::Draft,
            TransitionActorKind::Human
        )),
        Err(AssuranceError::Contract(_))
    ));
}

#[test]
fn empty_evidence_fails_contract() {
    let mut t = transition(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved,
        TransitionActorKind::Human,
    );
    t.evidence_ids.clear();
    assert!(matches!(
        verify_transition(&t),
        Err(AssuranceError::Contract(_))
    ));
}

#[test]
fn human_approval_flag_false_fails_critical_edge() {
    let mut t = transition(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved,
        TransitionActorKind::Human,
    );
    t.approved = false;
    assert!(matches!(
        verify_transition(&t),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
}

#[test]
fn tool_actor_can_run_non_critical_edges() {
    // SearchExecuted -> Deduplicated is not human-gated.
    assert!(
        verify_transition(&transition(
            LifecycleStage::SearchExecuted,
            LifecycleStage::Deduplicated,
            TransitionActorKind::Tool
        ))
        .is_ok()
    );
}

#[test]
fn agent_cannot_close_full_text() {
    assert!(matches!(
        verify_transition(&transition(
            LifecycleStage::TitleAbstractComplete,
            LifecycleStage::FullTextComplete,
            TransitionActorKind::Agent
        )),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
}

#[test]
fn update_planned_to_strategy_validated_is_allowed() {
    assert!(allowed_transition(
        LifecycleStage::UpdatePlanned,
        LifecycleStage::StrategyValidated
    ));
    assert!(requires_human_approval(
        LifecycleStage::UpdatePlanned,
        LifecycleStage::StrategyValidated
    ));
}

#[test]
fn reported_to_update_planned_is_allowed() {
    assert!(allowed_transition(
        LifecycleStage::Reported,
        LifecycleStage::UpdatePlanned
    ));
}

#[test]
fn empty_trace_reports_initial_stage() {
    let report = verify_trace(&trace(vec![])).unwrap_or_else(|_| panic!("expected report"));
    assert_eq!(report.final_stage, LifecycleStage::Draft);
    assert_eq!(report.transition_count, 0);
    assert_eq!(report.verified_invariants.len(), 5);
}

#[test]
fn discontinuous_trace_is_rejected_with_index() {
    let mut second = transition(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved,
        TransitionActorKind::Human,
    );
    second.transition_id = "t-second".to_string();
    let transitions = vec![
        transition(
            LifecycleStage::Draft,
            LifecycleStage::PlanApproved,
            TransitionActorKind::Human,
        ),
        second,
    ];
    assert!(matches!(
        verify_trace(&trace(transitions)),
        Err(AssuranceError::Discontinuous { index: 1, .. })
    ));
}

#[test]
fn duplicate_transition_id_fails_contract() {
    let mut t1 = transition(
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved,
        TransitionActorKind::Human,
    );
    t1.transition_id = "dup".to_string();
    let mut t2 = transition(
        LifecycleStage::PlanApproved,
        LifecycleStage::StrategyValidated,
        TransitionActorKind::Human,
    );
    t2.transition_id = "dup".to_string();
    assert!(matches!(
        verify_trace(&trace(vec![t1, t2])),
        Err(AssuranceError::Contract(_))
    ));
}

#[test]
fn two_step_happy_trace_counts_transitions() {
    let transitions = vec![
        transition(
            LifecycleStage::Draft,
            LifecycleStage::PlanApproved,
            TransitionActorKind::Human,
        ),
        transition(
            LifecycleStage::PlanApproved,
            LifecycleStage::StrategyValidated,
            TransitionActorKind::Human,
        ),
    ];
    let report = verify_trace(&trace(transitions)).unwrap_or_else(|_| panic!("expected report"));
    assert_eq!(report.final_stage, LifecycleStage::StrategyValidated);
    assert_eq!(report.transition_count, 2);
}

#[test]
fn non_human_cannot_approve_critical_edge_even_if_flagged() {
    assert!(matches!(
        verify_transition(&transition(
            LifecycleStage::FullTextComplete,
            LifecycleStage::Reported,
            TransitionActorKind::Tool
        )),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
}
