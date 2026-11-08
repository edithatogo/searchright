//! Comprehensive integration scenario tests for Track 26: Formal assurance and contract evolution.

#![forbid(unsafe_code)]

use searchright_assurance::{
    AssuranceError, allowed_transition, requires_human_approval, verify_trace, verify_transition,
};
use searchright_contracts::{
    LifecycleStage, LifecycleTransition, TransitionActorKind, WORKFLOW_TRACE_SCHEMA_VERSION,
    WorkflowTrace,
};

fn make_step(
    id: &str,
    from: LifecycleStage,
    to: LifecycleStage,
    actor_kind: TransitionActorKind,
    actor_id: &str,
    approved: bool,
) -> LifecycleTransition {
    LifecycleTransition {
        transition_id: id.to_string(),
        from,
        to,
        actor_kind,
        actor_id: actor_id.to_string(),
        occurred_at: "2026-11-09T00:00:00Z".to_string(),
        evidence_ids: vec![format!("ev-{id}")],
        approved,
    }
}

#[test]
fn test_full_systematic_review_and_living_update_lifecycle() {
    let transitions = vec![
        make_step(
            "t01",
            LifecycleStage::Draft,
            LifecycleStage::PlanApproved,
            TransitionActorKind::Human,
            "lead-investigator",
            true,
        ),
        make_step(
            "t02",
            LifecycleStage::PlanApproved,
            LifecycleStage::StrategyValidated,
            TransitionActorKind::Human,
            "information-specialist",
            true,
        ),
        make_step(
            "t03",
            LifecycleStage::StrategyValidated,
            LifecycleStage::ExecutionApproved,
            TransitionActorKind::Human,
            "review-lead",
            true,
        ),
        make_step(
            "t04",
            LifecycleStage::ExecutionApproved,
            LifecycleStage::SearchExecuted,
            TransitionActorKind::Tool,
            "search-adapter",
            true,
        ),
        make_step(
            "t05",
            LifecycleStage::SearchExecuted,
            LifecycleStage::Deduplicated,
            TransitionActorKind::Tool,
            "dedup-pipeline",
            true,
        ),
        make_step(
            "t06",
            LifecycleStage::Deduplicated,
            LifecycleStage::TitleAbstractComplete,
            TransitionActorKind::Tool,
            "screening-triage",
            true,
        ),
        make_step(
            "t07",
            LifecycleStage::TitleAbstractComplete,
            LifecycleStage::FullTextComplete,
            TransitionActorKind::Human,
            "human-adjudicator",
            true,
        ),
        make_step(
            "t08",
            LifecycleStage::FullTextComplete,
            LifecycleStage::Reported,
            TransitionActorKind::Human,
            "principal-author",
            true,
        ),
        // Living review cycle
        make_step(
            "t09",
            LifecycleStage::Reported,
            LifecycleStage::UpdatePlanned,
            TransitionActorKind::Human,
            "living-review-chair",
            true,
        ),
        make_step(
            "t10",
            LifecycleStage::UpdatePlanned,
            LifecycleStage::StrategyValidated,
            TransitionActorKind::Human,
            "information-specialist",
            true,
        ),
        make_step(
            "t11",
            LifecycleStage::StrategyValidated,
            LifecycleStage::ExecutionApproved,
            TransitionActorKind::Human,
            "review-lead",
            true,
        ),
        make_step(
            "t12",
            LifecycleStage::ExecutionApproved,
            LifecycleStage::SearchExecuted,
            TransitionActorKind::Tool,
            "search-adapter-v2",
            true,
        ),
        make_step(
            "t13",
            LifecycleStage::SearchExecuted,
            LifecycleStage::Deduplicated,
            TransitionActorKind::Tool,
            "dedup-pipeline-v2",
            true,
        ),
        make_step(
            "t14",
            LifecycleStage::Deduplicated,
            LifecycleStage::TitleAbstractComplete,
            TransitionActorKind::Tool,
            "screening-triage-v2",
            true,
        ),
        make_step(
            "t15",
            LifecycleStage::TitleAbstractComplete,
            LifecycleStage::FullTextComplete,
            TransitionActorKind::Human,
            "human-adjudicator",
            true,
        ),
        make_step(
            "t16",
            LifecycleStage::FullTextComplete,
            LifecycleStage::Reported,
            TransitionActorKind::Human,
            "principal-author",
            true,
        ),
    ];

    let trace = WorkflowTrace {
        schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_string(),
        review_id: "rev-assurance-living-001".to_string(),
        initial_stage: LifecycleStage::Draft,
        transitions: transitions.clone(),
    };

    let report = match verify_trace(&trace) {
        Ok(r) => r,
        Err(e) => panic!("living review trace must verify successfully: {e:?}"),
    };
    assert_eq!(report.final_stage, LifecycleStage::Reported);
    assert_eq!(report.transition_count, 16);
    assert_eq!(report.verified_invariants.len(), 5);

    // Metamorphic check: Every prefix of the valid trace must also verify and conclude at the expected stage
    let mut running_transitions = Vec::new();
    for step in &transitions {
        running_transitions.push(step.clone());
        let prefix_trace = WorkflowTrace {
            schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_string(),
            review_id: "rev-assurance-living-001".to_string(),
            initial_stage: LifecycleStage::Draft,
            transitions: running_transitions.clone(),
        };
        let prefix_report = match verify_trace(&prefix_trace) {
            Ok(r) => r,
            Err(e) => panic!("valid prefix must verify: {e:?}"),
        };
        assert_eq!(prefix_report.transition_count, running_transitions.len());
        assert_eq!(prefix_report.final_stage, step.to);
    }
}

#[test]
fn test_forbidden_shortcuts_and_retrograde_skips_fail_closed() {
    let forbidden_pairs = [
        (LifecycleStage::Draft, LifecycleStage::ExecutionApproved),
        (LifecycleStage::Draft, LifecycleStage::SearchExecuted),
        (LifecycleStage::Draft, LifecycleStage::Reported),
        (LifecycleStage::PlanApproved, LifecycleStage::Deduplicated),
        (LifecycleStage::StrategyValidated, LifecycleStage::Reported),
        (LifecycleStage::SearchExecuted, LifecycleStage::Reported),
        (
            LifecycleStage::Deduplicated,
            LifecycleStage::FullTextComplete,
        ),
        (LifecycleStage::Reported, LifecycleStage::Draft),
        (LifecycleStage::Reported, LifecycleStage::SearchExecuted),
        (
            LifecycleStage::UpdatePlanned,
            LifecycleStage::ExecutionApproved,
        ),
    ];

    for (from, to) in forbidden_pairs {
        assert!(
            !allowed_transition(from, to),
            "pair {from:?} -> {to:?} must be disallowed"
        );
        let step = make_step(
            "forbidden",
            from,
            to,
            TransitionActorKind::Human,
            "unauthorized",
            true,
        );
        match verify_transition(&step) {
            Err(AssuranceError::TransitionDenied {
                from: actual_from,
                to: actual_to,
            }) => {
                assert_eq!(actual_from, from);
                assert_eq!(actual_to, to);
            }
            other => panic!("expected TransitionDenied, got: {other:?}"),
        }
    }
}

#[test]
fn test_agent_authority_boundaries_are_strictly_enforced() {
    // Agents cannot perform human-gated steps even if approved is set to true
    let human_gated_steps = [
        (LifecycleStage::Draft, LifecycleStage::PlanApproved),
        (
            LifecycleStage::PlanApproved,
            LifecycleStage::StrategyValidated,
        ),
        (
            LifecycleStage::StrategyValidated,
            LifecycleStage::ExecutionApproved,
        ),
        (
            LifecycleStage::TitleAbstractComplete,
            LifecycleStage::FullTextComplete,
        ),
        (LifecycleStage::FullTextComplete, LifecycleStage::Reported),
        (LifecycleStage::Reported, LifecycleStage::UpdatePlanned),
        (
            LifecycleStage::UpdatePlanned,
            LifecycleStage::StrategyValidated,
        ),
    ];

    for (from, to) in human_gated_steps {
        assert!(requires_human_approval(from, to));
        let step = make_step(
            "agent-attempt",
            from,
            to,
            TransitionActorKind::Agent,
            "autonomous-agent",
            true,
        );
        match verify_transition(&step) {
            Err(AssuranceError::HumanApprovalRequired {
                from: actual_from,
                to: actual_to,
            }) => {
                assert_eq!(actual_from, from);
                assert_eq!(actual_to, to);
            }
            Err(AssuranceError::AgentAuthorityDenied { to: actual_to }) => {
                assert_eq!(actual_to, to);
            }
            other => panic!("unexpected error for agent on {from:?}->{to:?}: {other:?}"),
        }
    }
}

#[test]
fn test_unapproved_human_transitions_fail_closed() {
    let step = make_step(
        "unapproved-human",
        LifecycleStage::Draft,
        LifecycleStage::PlanApproved,
        TransitionActorKind::Human,
        "reviewer",
        false,
    );
    assert!(matches!(
        verify_transition(&step),
        Err(AssuranceError::HumanApprovalRequired { .. })
    ));
}

#[test]
fn test_tool_allowed_on_automated_phases() {
    // Automated data manipulation stages allow tool actors
    let automated_steps = [
        (
            LifecycleStage::ExecutionApproved,
            LifecycleStage::SearchExecuted,
        ),
        (LifecycleStage::SearchExecuted, LifecycleStage::Deduplicated),
        (
            LifecycleStage::Deduplicated,
            LifecycleStage::TitleAbstractComplete,
        ),
    ];

    for (from, to) in automated_steps {
        assert!(!requires_human_approval(from, to));
        let step = make_step(
            "tool-step",
            from,
            to,
            TransitionActorKind::Tool,
            "batch-job",
            true,
        );
        assert!(
            verify_transition(&step).is_ok(),
            "tool actor should be permitted for {from:?} -> {to:?}"
        );
    }
}

#[test]
fn test_trace_discontinuity_detection() {
    let steps = vec![
        make_step(
            "t1",
            LifecycleStage::Draft,
            LifecycleStage::PlanApproved,
            TransitionActorKind::Human,
            "user",
            true,
        ),
        // Step 2 jumps from ExecutionApproved instead of PlanApproved
        make_step(
            "t2",
            LifecycleStage::ExecutionApproved,
            LifecycleStage::SearchExecuted,
            TransitionActorKind::Tool,
            "engine",
            true,
        ),
    ];
    let trace = WorkflowTrace {
        schema_version: WORKFLOW_TRACE_SCHEMA_VERSION.to_string(),
        review_id: "rev-broken-continuity".to_string(),
        initial_stage: LifecycleStage::Draft,
        transitions: steps,
    };

    match verify_trace(&trace) {
        Err(AssuranceError::Discontinuous {
            index,
            expected,
            actual,
        }) => {
            assert_eq!(index, 1);
            assert_eq!(expected, LifecycleStage::PlanApproved);
            assert_eq!(actual, LifecycleStage::ExecutionApproved);
        }
        other => panic!("expected Discontinuous, got: {other:?}"),
    }
}

#[test]
fn test_assurance_report_serialization_roundtrip() {
    let report = searchright_assurance::AssuranceReport {
        final_stage: LifecycleStage::Reported,
        transition_count: 8,
        verified_invariants: vec![
            "state continuity".to_string(),
            "critical approvals are human".to_string(),
        ],
    };

    let json_bytes = match serde_json::to_string(&report) {
        Ok(b) => b,
        Err(e) => panic!("must serialize to JSON: {e:?}"),
    };
    let deserialized: searchright_assurance::AssuranceReport =
        match serde_json::from_str(&json_bytes) {
            Ok(d) => d,
            Err(e) => panic!("must deserialize from JSON: {e:?}"),
        };
    assert_eq!(report, deserialized);
}
