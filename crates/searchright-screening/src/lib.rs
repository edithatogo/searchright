//! Screening-state rules shared by CLI, MCP and future user interfaces.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use searchright_contracts::{
    AgentAuthority, ConflictResolution, DecisionValue, ReviewerKind, ScreeningDecision,
    ScreeningPolicy, ScreeningRound,
};

/// Derived status for a subject and round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreeningStatus {
    /// No or insufficient independent decisions.
    Pending {
        /// Number of independent decisions submitted so far.
        submitted: usize,
        /// Number of independent decisions required by policy.
        required: usize,
    },
    /// Required decisions agree.
    Consensus(DecisionValue),
    /// Submitted decisions disagree.
    Conflict,
    /// Conflict has been adjudicated.
    Resolved(DecisionValue),
}

/// In-memory decision board with deterministic policy enforcement.
#[derive(Debug, Clone)]
pub struct ScreeningBoard {
    policy: ScreeningPolicy,
    decisions: BTreeMap<(String, ScreeningRound), Vec<ScreeningDecision>>,
    resolutions: BTreeMap<(String, ScreeningRound), ConflictResolution>,
}

impl ScreeningBoard {
    /// Create a board under a validated review policy.
    pub fn new(policy: ScreeningPolicy) -> Result<Self, ScreeningError> {
        validate_policy(&policy)?;
        Ok(Self {
            policy,
            decisions: BTreeMap::new(),
            resolutions: BTreeMap::new(),
        })
    }

    /// Borrow the policy enforced by this board.
    #[must_use]
    pub fn policy(&self) -> &ScreeningPolicy {
        &self.policy
    }

    /// Record an independent decision after authority checks.
    pub fn submit(&mut self, decision: ScreeningDecision) -> Result<(), ScreeningError> {
        validate_decision(&decision, &self.policy)?;
        let key = (decision.subject_id.clone(), decision.round);
        let existing = self.decisions.entry(key).or_default();
        if existing
            .iter()
            .any(|item| item.reviewer_id == decision.reviewer_id)
        {
            return Err(ScreeningError::DuplicateReviewer {
                reviewer_id: decision.reviewer_id,
            });
        }
        if existing
            .iter()
            .any(|item| item.decision_id == decision.decision_id)
        {
            return Err(ScreeningError::DuplicateDecisionId {
                decision_id: decision.decision_id,
            });
        }
        existing.push(decision);
        Ok(())
    }

    /// Resolve a conflict using an explicit human adjudication record.
    pub fn resolve(&mut self, resolution: ConflictResolution) -> Result<(), ScreeningError> {
        validate_resolution_fields(&resolution)?;
        let key = (resolution.subject_id.clone(), resolution.round);
        if !matches!(
            self.status(&resolution.subject_id, resolution.round),
            ScreeningStatus::Conflict
        ) {
            return Err(ScreeningError::NoConflict);
        }
        let known_decisions = self.decisions.get(&key).map_or(&[][..], Vec::as_slice);
        let known_human_ids: BTreeSet<&str> = known_decisions
            .iter()
            .filter(|decision| decision.reviewer_kind != ReviewerKind::Agent)
            .map(|decision| decision.decision_id.as_str())
            .collect();
        let supplied_ids: BTreeSet<&str> =
            resolution.decision_ids.iter().map(String::as_str).collect();
        if supplied_ids.iter().any(|id| !known_human_ids.contains(id)) {
            return Err(ScreeningError::UnknownDecision);
        }
        if supplied_ids != known_human_ids {
            return Err(ScreeningError::IncompleteResolution);
        }
        self.resolutions.insert(key, resolution);
        Ok(())
    }

    /// Derive current status from independent human decisions.
    #[must_use]
    pub fn status(&self, subject_id: &str, round: ScreeningRound) -> ScreeningStatus {
        let key = (subject_id.to_owned(), round);
        if let Some(resolution) = self.resolutions.get(&key) {
            return ScreeningStatus::Resolved(resolution.resolved_decision);
        }
        let decisions = self.decisions.get(&key).map_or(&[][..], Vec::as_slice);
        let human_decisions: Vec<DecisionValue> = decisions
            .iter()
            .filter(|decision| decision.reviewer_kind != ReviewerKind::Agent)
            .map(|decision| decision.decision)
            .collect();
        let required = match round {
            ScreeningRound::TitleAbstract => usize::from(self.policy.title_abstract_reviewers),
            ScreeningRound::FullText => usize::from(self.policy.full_text_reviewers),
        };
        if human_decisions.len() < required {
            return ScreeningStatus::Pending {
                submitted: human_decisions.len(),
                required,
            };
        }
        if let Some(first) = human_decisions.first().copied() {
            if human_decisions.iter().all(|decision| *decision == first) {
                ScreeningStatus::Consensus(first)
            } else {
                ScreeningStatus::Conflict
            }
        } else {
            ScreeningStatus::Pending {
                submitted: 0,
                required,
            }
        }
    }

    /// Borrow submitted decisions for export.
    pub fn decisions(&self) -> impl Iterator<Item = &ScreeningDecision> {
        self.decisions.values().flatten()
    }
}

/// Screening-policy error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ScreeningError {
    /// The policy requested no reviewer at a stage.
    #[error("at least one independent human reviewer is required at each screening stage")]
    InvalidReviewerCount,
    /// Calibrated sensitivity was outside zero to one.
    #[error("minimum agent sensitivity must be finite and between zero and one")]
    InvalidSensitivity,
    /// Adjudication procedure was omitted.
    #[error("screening policy requires a non-empty adjudication rule")]
    MissingAdjudicationRule,
    /// A required decision or resolution field was empty.
    #[error("required screening field `{field}` is empty")]
    EmptyField {
        /// Name of the required field that was empty.
        field: &'static str,
    },
    /// The same reviewer submitted more than once for a subject/round.
    #[error("reviewer `{reviewer_id}` has already submitted a decision")]
    DuplicateReviewer {
        /// Identifier of the reviewer whose additional decision was rejected.
        reviewer_id: String,
    },
    /// A decision identifier was reused within a subject/round.
    #[error("decision identifier `{decision_id}` is already present")]
    DuplicateDecisionId {
        /// Reused decision identifier that caused the conflict.
        decision_id: String,
    },
    /// Agent attempted a prohibited exclusion.
    #[error("agent exclusion is not permitted by the current screening policy")]
    AgentExclusionDenied,
    /// Agent decision omitted model/tool provenance.
    #[error("agent decisions require model/tool provenance")]
    MissingAgentProvenance,
    /// Agent decision omitted calibrated confidence when the policy requires a threshold.
    #[error("agent decisions require calibrated confidence under this policy")]
    MissingAgentConfidence,
    /// Exclusion omitted its structured reason.
    #[error("an exclusion decision requires a structured exclusion reason")]
    MissingExclusionReason,
    /// A non-exclusion carried an exclusion reason.
    #[error("only exclusion decisions may carry an exclusion reason")]
    UnexpectedExclusionReason,
    /// Confidence was outside zero to one.
    #[error("confidence must be finite and between zero and one")]
    InvalidConfidence,
    /// Resolution was submitted without a current conflict.
    #[error("there is no unresolved screening conflict")]
    NoConflict,
    /// Resolution refers to a decision not held by the board.
    #[error("conflict resolution refers to an unknown human decision")]
    UnknownDecision,
    /// Resolution did not include every conflicting human decision.
    #[error("conflict resolution must reference every conflicting human decision exactly once")]
    IncompleteResolution,
}

fn validate_policy(policy: &ScreeningPolicy) -> Result<(), ScreeningError> {
    if policy.title_abstract_reviewers == 0 || policy.full_text_reviewers == 0 {
        return Err(ScreeningError::InvalidReviewerCount);
    }
    if let Some(sensitivity) = policy.minimum_agent_sensitivity
        && (!sensitivity.is_finite() || !(0.0..=1.0).contains(&sensitivity))
    {
        return Err(ScreeningError::InvalidSensitivity);
    }
    if policy.adjudication_rule.trim().is_empty() {
        return Err(ScreeningError::MissingAdjudicationRule);
    }
    Ok(())
}

fn validate_decision(
    decision: &ScreeningDecision,
    policy: &ScreeningPolicy,
) -> Result<(), ScreeningError> {
    for (field, value) in [
        ("decision_id", decision.decision_id.as_str()),
        ("review_id", decision.review_id.as_str()),
        ("subject_id", decision.subject_id.as_str()),
        ("reviewer_id", decision.reviewer_id.as_str()),
        ("decided_at", decision.decided_at.as_str()),
        ("rationale", decision.rationale.as_str()),
        ("eligibility_version", decision.eligibility_version.as_str()),
    ] {
        require_screening_text(value, field)?;
    }
    if let Some(confidence) = decision.confidence
        && (!confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
    {
        return Err(ScreeningError::InvalidConfidence);
    }
    match decision.decision {
        DecisionValue::Exclude if decision.exclusion_reason.is_none() => {
            return Err(ScreeningError::MissingExclusionReason);
        }
        DecisionValue::Include | DecisionValue::Unclear if decision.exclusion_reason.is_some() => {
            return Err(ScreeningError::UnexpectedExclusionReason);
        }
        _ => {}
    }
    if let Some(reason) = &decision.exclusion_reason {
        require_screening_text(&reason.reason_id, "exclusion_reason.reason_id")?;
        require_screening_text(&reason.criterion_id, "exclusion_reason.criterion_id")?;
        require_screening_text(&reason.label, "exclusion_reason.label")?;
    }
    if decision.reviewer_kind == ReviewerKind::Agent {
        if decision
            .agent_provenance
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(ScreeningError::MissingAgentProvenance);
        }
        if policy.minimum_agent_sensitivity.is_some() && decision.confidence.is_none() {
            return Err(ScreeningError::MissingAgentConfidence);
        }
        if decision.decision == DecisionValue::Exclude
            && matches!(
                policy.agent_authority,
                AgentAuthority::AdvisoryOnly | AgentAuthority::IncludeOnly
            )
        {
            return Err(ScreeningError::AgentExclusionDenied);
        }
    }
    Ok(())
}

fn validate_resolution_fields(resolution: &ConflictResolution) -> Result<(), ScreeningError> {
    for (field, value) in [
        ("resolution.subject_id", resolution.subject_id.as_str()),
        (
            "resolution.adjudicator_id",
            resolution.adjudicator_id.as_str(),
        ),
        ("resolution.rationale", resolution.rationale.as_str()),
        ("resolution.resolved_at", resolution.resolved_at.as_str()),
    ] {
        require_screening_text(value, field)?;
    }
    if resolution.decision_ids.is_empty() {
        return Err(ScreeningError::IncompleteResolution);
    }
    let unique: BTreeSet<&str> = resolution.decision_ids.iter().map(String::as_str).collect();
    if unique.len() != resolution.decision_ids.len() {
        return Err(ScreeningError::IncompleteResolution);
    }
    Ok(())
}

fn require_screening_text(value: &str, field: &'static str) -> Result<(), ScreeningError> {
    if value.trim().is_empty() {
        Err(ScreeningError::EmptyField { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{ExclusionReason, ScreeningDecision};

    use super::*;

    fn policy() -> ScreeningPolicy {
        ScreeningPolicy {
            schema_version: searchright_contracts::SCREENING_POLICY_SCHEMA_VERSION.to_owned(),
            title_abstract_reviewers: 2,
            full_text_reviewers: 2,
            agent_authority: AgentAuthority::AdvisoryOnly,
            minimum_agent_sensitivity: Some(0.99),
            independent_blinding: true,
            adjudication_rule: "third human".to_owned(),
        }
    }

    fn decision(reviewer: &str, kind: ReviewerKind, value: DecisionValue) -> ScreeningDecision {
        ScreeningDecision {
            decision_id: format!("d-{reviewer}"),
            review_id: "r1".to_owned(),
            subject_id: "record-1".to_owned(),
            round: ScreeningRound::TitleAbstract,
            reviewer_id: reviewer.to_owned(),
            reviewer_kind: kind.clone(),
            decision: value,
            exclusion_reason: (value == DecisionValue::Exclude).then(|| ExclusionReason {
                reason_id: "wrong-population".to_owned(),
                criterion_id: "population".to_owned(),
                label: "Wrong population".to_owned(),
                evidence: None,
            }),
            confidence: (kind == ReviewerKind::Agent).then_some(0.995),
            decided_at: "2026-08-05T00:00:00Z".to_owned(),
            rationale: "test".to_owned(),
            eligibility_version: "1".to_owned(),
            agent_provenance: (kind == ReviewerKind::Agent)
                .then(|| "model=test;version=1;prompt=sha256:fixture".to_owned()),
        }
    }

    #[test]
    fn invalid_policy_is_rejected() {
        let mut invalid = policy();
        invalid.full_text_reviewers = 0;
        assert!(matches!(
            ScreeningBoard::new(invalid),
            Err(ScreeningError::InvalidReviewerCount)
        ));
    }

    #[test]
    fn agent_cannot_exclude_by_default() {
        let board = ScreeningBoard::new(policy());
        assert!(board.is_ok());
        if let Ok(mut board) = board {
            let result = board.submit(decision(
                "agent",
                ReviewerKind::Agent,
                DecisionValue::Exclude,
            ));
            assert_eq!(result, Err(ScreeningError::AgentExclusionDenied));
        }
    }

    #[test]
    fn disagreement_creates_conflict() {
        let board = ScreeningBoard::new(policy());
        assert!(board.is_ok());
        if let Ok(mut board) = board {
            assert!(
                board
                    .submit(decision("a", ReviewerKind::Human, DecisionValue::Include))
                    .is_ok()
            );
            assert!(
                board
                    .submit(decision("b", ReviewerKind::Human, DecisionValue::Exclude))
                    .is_ok()
            );
            assert_eq!(
                board.status("record-1", ScreeningRound::TitleAbstract),
                ScreeningStatus::Conflict
            );
        }
    }
}
