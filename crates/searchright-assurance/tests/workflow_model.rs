//! Exhaustive and metamorphic checks for the lifecycle policy model.

use searchright_assurance::{
    AssuranceError, allowed_transition, requires_human_approval, verify_trace, verify_transition,
};
use searchright_contracts::{
    LifecycleStage, LifecycleTransition, TransitionActorKind, WORKFLOW_TRACE_SCHEMA_VERSION,
    WorkflowTrace,
};

const STAGES: [LifecycleStage; 10] = [
    LifecycleStage::Draft,
    LifecycleStage::PlanApproved,
    LifecycleStage::StrategyValidated,
    LifecycleStage::ExecutionApproved,
    LifecycleStage::SearchExecuted,
    LifecycleStage::Deduplicated,
    LifecycleStage::TitleAbstractComplete,
    LifecycleStage::FullTextComplete,
    LifecycleStage::Reported,
    LifecycleStage::UpdatePlanned,
];

fn transition(
    from: LifecycleStage,
    to: LifecycleStage,
    actor_kind: TransitionActorKind,
) -> LifecycleTransition {
    LifecycleTransition {
        transition_id: format!("{from:?}-{to:?}"),
        from,
        to,
        actor_kind,
        actor_id: "reviewer".to_owned(),
        occurred_at: "2026-08-12T00:00:00Z".to_owned(),
        evidence_ids: vec!["evidence-a".to_owned(), "evidence-b".to_owned()],
        approved: true,
    }
}

#[test]
fn every_declared_edge_accepts_a_human_actor() {
    for from in STAGES {
        for to in STAGES {
            let result = verify_transition(&transition(from, to, TransitionActorKind::Human));
            assert_eq!(
                result.is_ok(),
                allowed_transition(from, to),
                "human result disagrees with the declared edge {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn every_approval_gated_edge_denies_tools_and_agents() {
    for from in STAGES {
        for to in STAGES {
            if !requires_human_approval(from, to) {
                continue;
            }
            for actor_kind in [TransitionActorKind::Tool, TransitionActorKind::Agent] {
                assert!(matches!(
                    verify_transition(&transition(from, to, actor_kind)),
                    Err(AssuranceError::HumanApprovalRequired { .. })
                ));
            }
        }
    }
}

#[test]
fn approval_gated_edges_require_an_affirmative_human_approval() {
    for from in STAGES {
        for to in STAGES {
            if !requires_human_approval(from, to) {
                continue;
            }
            let mut candidate = transition(from, to, TransitionActorKind::Human);
            candidate.approved = false;
            assert!(matches!(
                verify_transition(&candidate),
                Err(AssuranceError::HumanApprovalRequired { .. })
            ));
        }
    }
}

#[test]
fn evidence_order_does_not_change_an_authority_decision() {
    let candidate = transition(
        LifecycleStage::StrategyValidated,
        LifecycleStage::ExecutionApproved,
        TransitionActorKind::Agent,
    );
    let mut reordered = candidate.clone();
    reordered.evidence_ids.reverse();

    assert!(matches!(
        verify_transition(&candidate),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
    assert!(matches!(
        verify_transition(&reordered),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
}

#[test]
fn a_valid_trace_prefix_reports_its_exact_final_state() {
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
        transition(
            LifecycleStage::StrategyValidated,
            LifecycleStage::ExecutionApproved,
            TransitionActorKind::Human,
        ),
    ];
    let trace = WorkflowTrace {
        schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_owned(),
        review_id: "review-1".to_owned(),
        initial_stage: LifecycleStage::Draft,
        transitions,
    };

    let result = verify_trace(&trace);
    assert!(result.is_ok(), "declared lifecycle prefix must verify");
    if let Ok(report) = result {
        assert_eq!(report.final_stage, LifecycleStage::ExecutionApproved);
        assert_eq!(report.transition_count, 3);
    }
}

#[test]
fn reordering_a_valid_trace_is_rejected_as_discontinuous() {
    let mut transitions = vec![
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
    transitions.swap(0, 1);
    let trace = WorkflowTrace {
        schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_owned(),
        review_id: "review-1".to_owned(),
        initial_stage: LifecycleStage::Draft,
        transitions,
    };

    assert!(matches!(
        verify_trace(&trace),
        Err(AssuranceError::Discontinuous { index: 0, .. })
    ));
}
