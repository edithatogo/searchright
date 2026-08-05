use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ScreeningStage;

/// Screening round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningRound {
    /// Title and abstract.
    TitleAbstract,
    /// Full text.
    FullText,
}

impl From<ScreeningRound> for ScreeningStage {
    fn from(value: ScreeningRound) -> Self {
        match value {
            ScreeningRound::TitleAbstract => Self::TitleAbstract,
            ScreeningRound::FullText => Self::FullText,
        }
    }
}

/// Human, agent or adjudicator reviewer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerKind {
    /// Human reviewer.
    Human,
    /// AI/agent recommendation.
    Agent,
    /// Human adjudicator.
    Adjudicator,
}

/// Screening decision value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecisionValue {
    /// Progress/include at this stage.
    Include,
    /// Exclude at this stage.
    Exclude,
    /// Insufficient evidence; seek clarification or full text.
    Unclear,
}

/// Structured exclusion reason tied to eligibility criteria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExclusionReason {
    /// Stable reason identifier.
    pub reason_id: String,
    /// Eligibility criterion identifier.
    pub criterion_id: String,
    /// Human-readable label.
    pub label: String,
    /// Optional evidence excerpt or note; no unnecessary full text.
    pub evidence: Option<String>,
}

/// One independent screening decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScreeningDecision {
    /// Decision identifier.
    pub decision_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Record or report identifier.
    pub subject_id: String,
    /// Screening round.
    pub round: ScreeningRound,
    /// Reviewer identifier or pseudonym.
    pub reviewer_id: String,
    /// Reviewer class.
    pub reviewer_kind: ReviewerKind,
    /// Decision.
    pub decision: DecisionValue,
    /// Exclusion reason when excluding.
    pub exclusion_reason: Option<ExclusionReason>,
    /// Confidence for agent recommendations or optional human calibration.
    pub confidence: Option<f64>,
    /// Decision timestamp.
    pub decided_at: String,
    /// Evidence-bearing explanation.
    pub rationale: String,
    /// Eligibility-contract version applied.
    pub eligibility_version: String,
    /// Model/tool provenance for agent decisions.
    pub agent_provenance: Option<String>,
}

/// Permitted agent authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentAuthority {
    /// Rank/recommend only; no state transition.
    AdvisoryOnly,
    /// May automatically progress likely includes, never exclude.
    IncludeOnly,
    /// Exclusion recommendation requires explicit human confirmation.
    ExclusionWithHumanConfirmation,
}

/// Screening governance policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScreeningPolicy {
    /// Independent reviewer count for title/abstract.
    pub title_abstract_reviewers: u8,
    /// Independent reviewer count for full text.
    pub full_text_reviewers: u8,
    /// Agent authority.
    pub agent_authority: AgentAuthority,
    /// Minimum calibrated sensitivity before agent prioritisation is used.
    pub minimum_agent_sensitivity: Option<f64>,
    /// Whether reviewers are blinded to one another until submission.
    pub independent_blinding: bool,
    /// Adjudication procedure.
    pub adjudication_rule: String,
}

/// Reconciled conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictResolution {
    /// Subject identifier.
    pub subject_id: String,
    /// Round.
    pub round: ScreeningRound,
    /// Input decision identifiers.
    pub decision_ids: Vec<String>,
    /// Final value.
    pub resolved_decision: DecisionValue,
    /// Adjudicator identifier.
    pub adjudicator_id: String,
    /// Resolution rationale.
    pub rationale: String,
    /// Resolution timestamp.
    pub resolved_at: String,
}
