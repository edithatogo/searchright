//! Deterministic agent workflow policy. This crate does not invoke an LLM.

#![forbid(unsafe_code)]

use schemars::JsonSchema;
use searchright_contracts::{
    AGENT_WORKFLOW_SCHEMA_VERSION, AgentAuthority, ContractError, ReviewPlan, Validate,
};
use serde::{Deserialize, Serialize};

/// Stage in the systematic-search agent workflow.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStage {
    /// Clarify question and review type.
    Scope,
    /// Define operational eligibility rules.
    Eligibility,
    /// Select sources and access routes.
    SourceSelection,
    /// Build controlled vocabulary and free-text concepts.
    StrategyDesign,
    /// Independent PRESS-style review.
    PressReview,
    /// Explicitly authorised provider execution.
    Execute,
    /// Deduplicate with retained evidence.
    Deduplicate,
    /// Title/abstract screening.
    TitleAbstractScreening,
    /// Full-text screening and exclusion reasons.
    FullTextScreening,
    /// Generate standards ledgers and flow outputs.
    Report,
    /// Plan an update or living search.
    Update,
}

/// Authority gate applied to a stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityGate {
    /// May run without confirmation and cannot write canonical state.
    ReadOnlyAutomatic,
    /// May create a local draft but requires human approval to promote it.
    HumanConfirmation,
    /// May access a network or create durable state only after explicit approval.
    ExplicitApproval,
    /// Subject to review-role and screening-policy rules.
    RolePolicy,
    /// Human-only final authority.
    HumanOnly,
}

/// One workflow step and its mandatory evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowStep {
    /// Stage.
    pub stage: WorkflowStage,
    /// Authority gate.
    pub authority: AuthorityGate,
    /// Required input artefacts.
    pub required_inputs: Vec<String>,
    /// Produced artefacts.
    pub outputs: Vec<String>,
    /// Conditions that prevent progression.
    pub blocking_conditions: Vec<String>,
}

/// Complete systematic-search agent workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentWorkflow {
    /// Workflow contract identifier.
    pub schema_version: String,
    /// Ordered steps.
    pub steps: Vec<WorkflowStep>,
    /// Screening authority applied to agent decisions.
    pub screening_authority: AgentAuthority,
}

impl AgentWorkflow {
    /// Conservative default workflow. Agents cannot exclude records.
    #[must_use]
    pub fn systematic_search() -> Self {
        let step = |stage, authority, inputs: &[&str], outputs: &[&str], blockers: &[&str]| {
            WorkflowStep {
                stage,
                authority,
                required_inputs: inputs.iter().map(|value| (*value).to_owned()).collect(),
                outputs: outputs.iter().map(|value| (*value).to_owned()).collect(),
                blocking_conditions: blockers.iter().map(|value| (*value).to_owned()).collect(),
            }
        };
        Self {
            schema_version: AGENT_WORKFLOW_SCHEMA_VERSION.to_owned(),
            screening_authority: AgentAuthority::AdvisoryOnly,
            steps: vec![
                step(WorkflowStage::Scope, AuthorityGate::HumanConfirmation, &[], &["review-plan-draft"], &["question unresolved"]),
                step(WorkflowStage::Eligibility, AuthorityGate::HumanConfirmation, &["review-plan-draft"], &["eligibility-contract"], &["criteria not operational"]),
                step(WorkflowStage::SourceSelection, AuthorityGate::HumanConfirmation, &["review-plan-draft"], &["information-source-plan"], &["required source or access path unresolved"]),
                step(WorkflowStage::StrategyDesign, AuthorityGate::HumanConfirmation, &["review-plan", "information-source-plan"], &["search-strategy"], &["lossy translation not reviewed"]),
                step(WorkflowStage::PressReview, AuthorityGate::HumanOnly, &["search-strategy"], &["press-review"], &["blocking PRESS finding"]),
                step(WorkflowStage::Execute, AuthorityGate::ExplicitApproval, &["approved strategy", "execution policy"], &["source receipts", "records"], &["live permission absent", "secrets unavailable"]),
                step(WorkflowStage::Deduplicate, AuthorityGate::HumanConfirmation, &["records"], &["duplicate clusters", "deduplication log"], &["ambiguous fuzzy cluster"]),
                step(WorkflowStage::TitleAbstractScreening, AuthorityGate::RolePolicy, &["deduplicated records", "eligibility contract"], &["screening decisions"], &["unresolved conflict"]),
                step(WorkflowStage::FullTextScreening, AuthorityGate::HumanOnly, &["full text", "eligibility contract"], &["screening decisions", "exclusion reasons"], &["missing full text", "unresolved conflict"]),
                step(WorkflowStage::Report, AuthorityGate::ReadOnlyAutomatic, &["audit ledger"], &["PRISMA-S ledger", "PRISMA flow", "search appendix"], &["flow arithmetic invalid"]),
                step(WorkflowStage::Update, AuthorityGate::HumanConfirmation, &["prior search run", "protocol"], &["update plan"], &["amendment not recorded"]),
            ],
        }
    }
}

impl Validate for AgentWorkflow {
    fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version != AGENT_WORKFLOW_SCHEMA_VERSION {
            return Err(ContractError::Invariant(format!(
                "agent workflow schema version must be `{AGENT_WORKFLOW_SCHEMA_VERSION}`"
            )));
        }
        if self.steps.is_empty() {
            return Err(ContractError::EmptyCollection("agent_workflow.steps"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for step in &self.steps {
            if !seen.insert(step.stage) {
                return Err(ContractError::Invariant(
                    "agent workflow stages must be unique".to_owned(),
                ));
            }
            if step.outputs.is_empty() {
                return Err(ContractError::Invariant(format!(
                    "workflow stage {:?} requires at least one output",
                    step.stage
                )));
            }
            if step.outputs.iter().chain(&step.required_inputs).chain(&step.blocking_conditions).any(|value| value.trim().is_empty()) {
                return Err(ContractError::Invariant(
                    "workflow evidence collections must not contain empty values".to_owned(),
                ));
            }
        }
        let full_text = self
            .steps
            .iter()
            .find(|step| step.stage == WorkflowStage::FullTextScreening);
        if !matches!(full_text.map(|step| step.authority), Some(AuthorityGate::HumanOnly)) {
            return Err(ContractError::Invariant(
                "full-text screening must retain human-only final authority".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Deterministic readiness finding for a review plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessFinding {
    /// Stable code.
    pub code: String,
    /// Whether it blocks execution.
    pub blocking: bool,
    /// Explanation.
    pub message: String,
}

/// Check whether a plan has enough structure to start strategy design.
#[must_use]
pub fn assess_plan_readiness(plan: &ReviewPlan) -> Vec<ReadinessFinding> {
    let mut findings = Vec::new();
    if let Err(error) = plan.validate() {
        findings.push(ReadinessFinding {
            code: "plan.contract.invalid".to_owned(),
            blocking: true,
            message: error.to_string(),
        });
        return findings;
    }
    if plan.information_sources.len() < 2 {
        findings.push(ReadinessFinding {
            code: "plan.sources.single".to_owned(),
            blocking: false,
            message: "Only one information source is planned; justify this explicitly or add complementary sources.".to_owned(),
        });
    }
    if !plan.governance.press_review_required {
        findings.push(ReadinessFinding {
            code: "plan.press.not_required".to_owned(),
            blocking: false,
            message: "PRESS-style peer review is disabled in governance; record the rationale.".to_owned(),
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_workflow_reserves_full_text_for_humans() {
        let workflow = AgentWorkflow::systematic_search();
        let full_text = workflow
            .steps
            .iter()
            .find(|step| step.stage == WorkflowStage::FullTextScreening);
        assert!(full_text.is_some());
        if let Some(full_text) = full_text {
            assert_eq!(full_text.authority, AuthorityGate::HumanOnly);
        }
        assert_eq!(workflow.screening_authority, AgentAuthority::AdvisoryOnly);
    }
}
