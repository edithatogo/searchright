//! Finite-state assurance for review workflow invariants.

#![forbid(unsafe_code)]

use schemars::JsonSchema;
use searchright_contracts::{
    LifecycleStage, LifecycleTransition, TransitionActorKind, Validate, WorkflowTrace,
};
use serde::{Deserialize, Serialize};

/// Result of verifying a complete lifecycle trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AssuranceReport {
    /// Final lifecycle stage.
    pub final_stage: LifecycleStage,
    /// Number of verified transitions.
    pub transition_count: usize,
    /// Stable invariants that were checked.
    pub verified_invariants: Vec<String>,
}

/// Verify state continuity, allowed transitions, evidence and human authority.
pub fn verify_trace(trace: &WorkflowTrace) -> Result<AssuranceReport, AssuranceError> {
    trace.validate()?;
    let mut current = trace.initial_stage;
    for (index, transition) in trace.transitions.iter().enumerate() {
        if transition.from != current {
            return Err(AssuranceError::Discontinuous {
                index,
                expected: current,
                actual: transition.from,
            });
        }
        verify_transition(transition)?;
        current = transition.to;
    }
    Ok(AssuranceReport {
        final_stage: current,
        transition_count: trace.transitions.len(),
        verified_invariants: vec![
            "state continuity".to_owned(),
            "no undeclared lifecycle skips".to_owned(),
            "critical approvals are human".to_owned(),
            "each transition carries evidence".to_owned(),
            "living updates re-enter strategy validation".to_owned(),
        ],
    })
}

/// Verify one transition against the finite policy model.
pub fn verify_transition(transition: &LifecycleTransition) -> Result<(), AssuranceError> {
    transition.validate()?;
    if !allowed_transition(transition.from, transition.to) {
        return Err(AssuranceError::TransitionDenied {
            from: transition.from,
            to: transition.to,
        });
    }
    if requires_human_approval(transition.from, transition.to)
        && (transition.actor_kind != TransitionActorKind::Human || !transition.approved)
    {
        return Err(AssuranceError::HumanApprovalRequired {
            from: transition.from,
            to: transition.to,
        });
    }
    if transition.actor_kind == TransitionActorKind::Agent
        && matches!(
            transition.to,
            LifecycleStage::FullTextComplete | LifecycleStage::Reported
        )
    {
        return Err(AssuranceError::AgentAuthorityDenied {
            to: transition.to,
        });
    }
    Ok(())
}

/// Enumerate the explicitly permitted lifecycle edges.
#[must_use]
pub fn allowed_transition(from: LifecycleStage, to: LifecycleStage) -> bool {
    matches!(
        (from, to),
        (LifecycleStage::Draft, LifecycleStage::PlanApproved)
            | (LifecycleStage::PlanApproved, LifecycleStage::StrategyValidated)
            | (
                LifecycleStage::StrategyValidated,
                LifecycleStage::ExecutionApproved
            )
            | (
                LifecycleStage::ExecutionApproved,
                LifecycleStage::SearchExecuted
            )
            | (LifecycleStage::SearchExecuted, LifecycleStage::Deduplicated)
            | (
                LifecycleStage::Deduplicated,
                LifecycleStage::TitleAbstractComplete
            )
            | (
                LifecycleStage::TitleAbstractComplete,
                LifecycleStage::FullTextComplete
            )
            | (LifecycleStage::FullTextComplete, LifecycleStage::Reported)
            | (LifecycleStage::Reported, LifecycleStage::UpdatePlanned)
            | (
                LifecycleStage::UpdatePlanned,
                LifecycleStage::StrategyValidated
            )
    )
}

/// Whether an edge represents a governance decision that an agent/tool cannot approve.
#[must_use]
pub fn requires_human_approval(from: LifecycleStage, to: LifecycleStage) -> bool {
    matches!(
        (from, to),
        (LifecycleStage::Draft, LifecycleStage::PlanApproved)
            | (LifecycleStage::PlanApproved, LifecycleStage::StrategyValidated)
            | (
                LifecycleStage::StrategyValidated,
                LifecycleStage::ExecutionApproved
            )
            | (
                LifecycleStage::TitleAbstractComplete,
                LifecycleStage::FullTextComplete
            )
            | (LifecycleStage::FullTextComplete, LifecycleStage::Reported)
            | (LifecycleStage::Reported, LifecycleStage::UpdatePlanned)
            | (
                LifecycleStage::UpdatePlanned,
                LifecycleStage::StrategyValidated
            )
    )
}

/// Workflow-assurance failure.
#[derive(Debug, thiserror::Error)]
pub enum AssuranceError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Adjacent transitions did not join.
    #[error(
        "workflow transition {index} starts at {actual:?}, but the prior state is {expected:?}"
    )]
    Discontinuous {
        index: usize,
        expected: LifecycleStage,
        actual: LifecycleStage,
    },
    /// The finite policy model has no such edge.
    #[error("workflow transition from {from:?} to {to:?} is not permitted")]
    TransitionDenied {
        from: LifecycleStage,
        to: LifecycleStage,
    },
    /// A critical edge was not human-approved.
    #[error("workflow transition from {from:?} to {to:?} requires explicit human approval")]
    HumanApprovalRequired {
        from: LifecycleStage,
        to: LifecycleStage,
    },
    /// Agent authority was insufficient for the destination state.
    #[error("agent authority cannot close lifecycle stage {to:?}")]
    AgentAuthorityDenied { to: LifecycleStage },
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{LifecycleTransition, TransitionActorKind};

    use super::*;

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
            occurred_at: "2026-08-06T00:00:00Z".to_owned(),
            evidence_ids: vec!["evidence-1".to_owned()],
            approved: true,
        }
    }

    #[test]
    fn finite_policy_denies_every_undeclared_edge() {
        let stages = [
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
        for from in stages.iter().copied() {
            for to in stages.iter().copied() {
                let candidate = transition(from, to, TransitionActorKind::Human);
                assert_eq!(verify_transition(&candidate).is_ok(), allowed_transition(from, to));
            }
        }
    }

    #[test]
    fn critical_approval_cannot_be_delegated_to_an_agent() {
        let candidate = transition(
            LifecycleStage::StrategyValidated,
            LifecycleStage::ExecutionApproved,
            TransitionActorKind::Agent,
        );
        assert!(matches!(
            verify_transition(&candidate),
            Err(AssuranceError::HumanApprovalRequired { .. })
        ));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use searchright_contracts::{
        LifecycleStage, LifecycleTransition, TransitionActorKind,
    };

    use super::{AssuranceError, allowed_transition, verify_transition};

    fn transition(
        from: LifecycleStage,
        to: LifecycleStage,
        actor_kind: TransitionActorKind,
    ) -> LifecycleTransition {
        LifecycleTransition {
            transition_id: "kani-transition".to_owned(),
            from,
            to,
            actor_kind,
            actor_id: "kani-actor".to_owned(),
            occurred_at: "2026-08-06T00:00:00Z".to_owned(),
            evidence_ids: vec!["kani-evidence".to_owned()],
            approved: true,
        }
    }

    #[kani::proof]
    fn an_agent_cannot_approve_the_review_plan() {
        let candidate = transition(
            LifecycleStage::Draft,
            LifecycleStage::PlanApproved,
            TransitionActorKind::Agent,
        );
        assert!(matches!(
            verify_transition(&candidate),
            Err(AssuranceError::HumanApprovalRequired { .. })
        ));
    }

    #[kani::proof]
    fn an_agent_cannot_close_full_text_screening() {
        let candidate = transition(
            LifecycleStage::TitleAbstractComplete,
            LifecycleStage::FullTextComplete,
            TransitionActorKind::Agent,
        );
        assert!(verify_transition(&candidate).is_err());
    }

    fn stage(index: u8) -> LifecycleStage {
        match index % 10 {
            0 => LifecycleStage::Draft,
            1 => LifecycleStage::PlanApproved,
            2 => LifecycleStage::StrategyValidated,
            3 => LifecycleStage::ExecutionApproved,
            4 => LifecycleStage::SearchExecuted,
            5 => LifecycleStage::Deduplicated,
            6 => LifecycleStage::TitleAbstractComplete,
            7 => LifecycleStage::FullTextComplete,
            8 => LifecycleStage::Reported,
            _ => LifecycleStage::UpdatePlanned,
        }
    }

    #[kani::proof]
    fn undeclared_edges_remain_denied() {
        let from = stage(kani::any());
        let to = stage(kani::any());
        kani::assume(!allowed_transition(from, to));
        let candidate = transition(from, to, TransitionActorKind::Human);
        assert!(matches!(
            verify_transition(&candidate),
            Err(AssuranceError::TransitionDenied { .. })
        ));
    }
}
