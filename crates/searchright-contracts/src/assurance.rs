use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, Validate, WORKFLOW_TRACE_SCHEMA_VERSION, require_schema_version, require_text,
};

/// High-level lifecycle stage used by the executable workflow assurance model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    /// Review intent exists but has not been approved as a plan.
    Draft,
    /// Review plan and governance were approved.
    PlanApproved,
    /// Source-specific strategies passed configured validation gates.
    StrategyValidated,
    /// A human explicitly authorised execution under a capability envelope.
    ExecutionApproved,
    /// Search execution completed or stopped with an auditable partial result.
    SearchExecuted,
    /// Duplicate candidates were generated and reviewed under policy.
    Deduplicated,
    /// Title and abstract screening reached a governed closeout state.
    TitleAbstractComplete,
    /// Full-text screening reached a governed closeout state.
    FullTextComplete,
    /// Reporting artefacts were generated from reconciled evidence.
    Reported,
    /// A living-review update was approved and scoped.
    UpdatePlanned,
}

/// Actor category for one lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransitionActorKind {
    /// A human with the declared role and authority.
    Human,
    /// An automated tool operating inside an approved policy envelope.
    Tool,
    /// An agent whose authority remains bounded by the review policy.
    Agent,
}

/// One evidence-bearing transition between lifecycle states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LifecycleTransition {
    /// Stable transition identifier.
    pub transition_id: String,
    /// State before the transition.
    pub from: LifecycleStage,
    /// State after the transition.
    pub to: LifecycleStage,
    /// Actor category.
    pub actor_kind: TransitionActorKind,
    /// Human, tool or agent identifier.
    pub actor_id: String,
    /// RFC 3339 timestamp.
    pub occurred_at: String,
    /// Evidence identifiers supporting the transition.
    pub evidence_ids: Vec<String>,
    /// Whether the governing approval requirement was met.
    pub approved: bool,
}

/// Complete lifecycle trace for formal policy verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowTrace {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Initial state, normally draft.
    pub initial_stage: LifecycleStage,
    /// Ordered lifecycle transitions.
    pub transitions: Vec<LifecycleTransition>,
}

impl Validate for LifecycleTransition {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.transition_id, "workflow.transition_id")?;
        require_text(&self.actor_id, "workflow.actor_id")?;
        require_text(&self.occurred_at, "workflow.occurred_at")?;
        if self.from == self.to {
            return Err(ContractError::Invariant(
                "workflow transitions must change lifecycle state".to_owned(),
            ));
        }
        if self.evidence_ids.is_empty() {
            return Err(ContractError::EmptyCollection(
                "workflow.transition.evidence_ids",
            ));
        }
        if self.evidence_ids.iter().any(|value| value.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "workflow transition evidence identifiers must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for WorkflowTrace {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            WORKFLOW_TRACE_SCHEMA_VERSION,
            "workflow.schema_version",
        )?;
        require_text(&self.review_id, "workflow.review_id")?;
        let mut identifiers = BTreeSet::new();
        for transition in &self.transitions {
            transition.validate()?;
            if !identifiers.insert(transition.transition_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "workflow transition identifier `{}` is duplicated",
                    transition.transition_id
                )));
            }
        }
        Ok(())
    }
}
