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

/// Append-only record of a successful screening-board transition.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreeningHistoryEntry {
    /// An independent human decision or agent recommendation was accepted.
    Decision(ScreeningDecision),
    /// A human explicitly confirmed an agent exclusion recommendation.
    AgentExclusionConfirmed {
        /// Agent recommendation being confirmed.
        agent_decision_id: String,
        /// Human decision recording final authority.
        human_decision_id: String,
    },
    /// A conflict was resolved by an independent human adjudicator.
    Resolution(ConflictResolution),
}

/// Validated, non-mutating preview of a bulk submission.
#[derive(Debug, Clone)]
pub struct BulkSubmissionPreview {
    base_revision: u64,
    decisions: Vec<ScreeningDecision>,
}

impl BulkSubmissionPreview {
    /// Number of decisions admitted by the preview.
    #[must_use]
    pub const fn accepted(&self) -> usize {
        self.decisions.len()
    }
}

/// In-memory decision board with deterministic policy enforcement.
#[derive(Debug, Clone)]
pub struct ScreeningBoard {
    policy: ScreeningPolicy,
    decisions: BTreeMap<(String, ScreeningRound), Vec<ScreeningDecision>>,
    resolutions: BTreeMap<(String, ScreeningRound), ConflictResolution>,
    history: Vec<ScreeningHistoryEntry>,
    revision: u64,
}

impl ScreeningBoard {
    /// Create a board under a validated review policy.
    pub fn new(policy: ScreeningPolicy) -> Result<Self, ScreeningError> {
        validate_policy(&policy)?;
        Ok(Self {
            policy,
            decisions: BTreeMap::new(),
            resolutions: BTreeMap::new(),
            history: Vec::new(),
            revision: 0,
        })
    }

    /// Borrow the policy enforced by this board.
    #[must_use]
    pub const fn policy(&self) -> &ScreeningPolicy {
        &self.policy
    }

    /// Record an independent decision after authority checks.
    pub fn submit(&mut self, decision: ScreeningDecision) -> Result<(), ScreeningError> {
        self.submit_inner(decision)
    }

    fn submit_inner(&mut self, decision: ScreeningDecision) -> Result<(), ScreeningError> {
        validate_decision(&decision, &self.policy)?;
        let key = (decision.subject_id.clone(), decision.round);
        if self.resolutions.contains_key(&key) {
            return Err(ScreeningError::RoundFinalized);
        }
        if self
            .decisions
            .values()
            .flatten()
            .any(|item| item.decision_id == decision.decision_id)
        {
            return Err(ScreeningError::DuplicateDecisionId {
                decision_id: decision.decision_id,
            });
        }
        let existing = self.decisions.entry(key).or_default();
        if existing
            .iter()
            .any(|item| item.reviewer_id == decision.reviewer_id)
        {
            return Err(ScreeningError::DuplicateReviewer {
                reviewer_id: decision.reviewer_id,
            });
        }
        if decision.reviewer_kind == ReviewerKind::Human {
            let required = match decision.round {
                ScreeningRound::TitleAbstract => self.policy.title_abstract_reviewers,
                ScreeningRound::FullText => self.policy.full_text_reviewers,
            };
            let submitted = existing
                .iter()
                .filter(|item| item.reviewer_kind == ReviewerKind::Human)
                .count();
            if submitted >= usize::from(required) {
                return Err(ScreeningError::IndependentReviewComplete);
            }
        }
        existing.push(decision.clone());
        self.history.push(ScreeningHistoryEntry::Decision(decision));
        self.revision += 1;
        Ok(())
    }

    /// Preview a bulk submission without changing canonical board state.
    pub fn preview_bulk(
        &self,
        decisions: Vec<ScreeningDecision>,
    ) -> Result<BulkSubmissionPreview, ScreeningError> {
        if decisions.is_empty() {
            return Err(ScreeningError::EmptyBulkSubmission);
        }
        let mut candidate = self.clone();
        for decision in decisions.iter().cloned() {
            candidate.submit_inner(decision)?;
        }
        Ok(BulkSubmissionPreview {
            base_revision: self.revision,
            decisions,
        })
    }

    /// Atomically apply a preview if board state has not changed since preview.
    pub fn apply_bulk(&mut self, preview: BulkSubmissionPreview) -> Result<(), ScreeningError> {
        if preview.base_revision != self.revision {
            return Err(ScreeningError::StaleBulkPreview);
        }
        let mut candidate = self.clone();
        for decision in preview.decisions {
            candidate.submit_inner(decision)?;
        }
        *self = candidate;
        Ok(())
    }

    /// Record explicit human confirmation of an agent exclusion recommendation.
    pub fn confirm_agent_exclusion(
        &mut self,
        agent_decision_id: &str,
        human_decision: ScreeningDecision,
    ) -> Result<(), ScreeningError> {
        if self.policy.agent_authority != AgentAuthority::ExclusionWithHumanConfirmation {
            return Err(ScreeningError::AgentExclusionDenied);
        }
        let agent = self
            .decisions
            .values()
            .flatten()
            .find(|decision| decision.decision_id == agent_decision_id)
            .ok_or(ScreeningError::UnknownAgentRecommendation)?;
        if agent.reviewer_kind != ReviewerKind::Agent
            || agent.decision != DecisionValue::Exclude
            || human_decision.reviewer_kind != ReviewerKind::Human
            || human_decision.decision != DecisionValue::Exclude
            || agent.review_id != human_decision.review_id
            || agent.subject_id != human_decision.subject_id
            || agent.round != human_decision.round
        {
            return Err(ScreeningError::InvalidHumanConfirmation);
        }
        let human_decision_id = human_decision.decision_id.clone();
        self.submit_inner(human_decision)?;
        self.history
            .push(ScreeningHistoryEntry::AgentExclusionConfirmed {
                agent_decision_id: agent_decision_id.to_owned(),
                human_decision_id,
            });
        self.revision += 1;
        Ok(())
    }

    /// Resolve a conflict using an explicit human adjudication record.
    pub fn resolve(&mut self, resolution: ConflictResolution) -> Result<(), ScreeningError> {
        validate_resolution_fields(&resolution)?;
        let key = (resolution.subject_id.clone(), resolution.round);
        if self.resolutions.contains_key(&key) {
            return Err(ScreeningError::RoundFinalized);
        }
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
        if known_decisions
            .iter()
            .any(|decision| decision.reviewer_id == resolution.adjudicator_id)
        {
            return Err(ScreeningError::AdjudicatorNotIndependent);
        }
        self.resolutions.insert(key, resolution.clone());
        self.history
            .push(ScreeningHistoryEntry::Resolution(resolution));
        self.revision += 1;
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

    /// Borrow the append-only transition history in acceptance order.
    #[must_use]
    pub fn history(&self) -> &[ScreeningHistoryEntry] {
        &self.history
    }
}

/// Screening-policy error.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
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
    /// Adjudicator role was used for an independent screening decision.
    #[error("adjudicators resolve conflicts and cannot submit independent screening decisions")]
    InvalidReviewerRole,
    /// Required independent decisions have already been recorded.
    #[error("the required independent human decisions are already complete")]
    IndependentReviewComplete,
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
    /// A resolved round cannot be amended in place.
    #[error("the screening round is finalized; append a governed re-screen event instead")]
    RoundFinalized,
    /// Conflict adjudicator participated in the original decisions.
    #[error("the conflict adjudicator must be independent of the original reviewers")]
    AdjudicatorNotIndependent,
    /// No matching agent exclusion recommendation exists.
    #[error("human confirmation refers to an unknown agent exclusion recommendation")]
    UnknownAgentRecommendation,
    /// Human confirmation did not match the agent recommendation and authority boundary.
    #[error("agent exclusion confirmation requires a matching explicit human exclusion decision")]
    InvalidHumanConfirmation,
    /// Bulk apply was attempted after canonical state changed.
    #[error("bulk preview is stale; generate a new preview before apply")]
    StaleBulkPreview,
    /// Empty bulk operations are not meaningful or auditable.
    #[error("bulk submission must contain at least one decision")]
    EmptyBulkSubmission,
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
    if decision.reviewer_kind == ReviewerKind::Adjudicator {
        return Err(ScreeningError::InvalidReviewerRole);
    }
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
            && policy.agent_authority != AgentAuthority::ExclusionWithHumanConfirmation
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

    #[test]
    fn agent_exclusion_remains_advisory_until_matching_human_confirmation()
    -> Result<(), ScreeningError> {
        let mut governed = policy();
        governed.agent_authority = AgentAuthority::ExclusionWithHumanConfirmation;
        let mut board = ScreeningBoard::new(governed)?;
        let recommendation = decision("agent", ReviewerKind::Agent, DecisionValue::Exclude);
        board.submit(recommendation.clone())?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Pending {
                submitted: 0,
                required: 2,
            }
        );

        let confirmation = decision("human-a", ReviewerKind::Human, DecisionValue::Exclude);
        board.confirm_agent_exclusion(&recommendation.decision_id, confirmation)?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Pending {
                submitted: 1,
                required: 2,
            }
        );
        assert!(matches!(
            board.history().last(),
            Some(ScreeningHistoryEntry::AgentExclusionConfirmed { .. })
        ));
        Ok(())
    }

    #[test]
    fn mismatched_human_confirmation_is_rejected_without_mutation() -> Result<(), ScreeningError> {
        let mut governed = policy();
        governed.agent_authority = AgentAuthority::ExclusionWithHumanConfirmation;
        let mut board = ScreeningBoard::new(governed)?;
        let recommendation = decision("agent", ReviewerKind::Agent, DecisionValue::Exclude);
        board.submit(recommendation.clone())?;
        let before = board.history().to_vec();
        let confirmation = decision("human-a", ReviewerKind::Human, DecisionValue::Include);
        assert_eq!(
            board.confirm_agent_exclusion(&recommendation.decision_id, confirmation),
            Err(ScreeningError::InvalidHumanConfirmation)
        );
        assert_eq!(board.history(), before);
        Ok(())
    }

    #[test]
    fn adjudication_is_independent_complete_and_append_only() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Exclude))?;
        let resolution = ConflictResolution {
            subject_id: "record-1".to_owned(),
            round: ScreeningRound::TitleAbstract,
            decision_ids: vec!["d-a".to_owned(), "d-b".to_owned()],
            resolved_decision: DecisionValue::Include,
            adjudicator_id: "c".to_owned(),
            rationale: "Independent review of both rationales".to_owned(),
            resolved_at: "2026-08-12T00:00:00Z".to_owned(),
        };
        board.resolve(resolution.clone())?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Resolved(DecisionValue::Include)
        );
        assert_eq!(
            board.resolve(resolution),
            Err(ScreeningError::RoundFinalized)
        );
        assert_eq!(
            board.submit(decision("d", ReviewerKind::Human, DecisionValue::Exclude)),
            Err(ScreeningError::RoundFinalized)
        );
        assert_eq!(board.history().len(), 3);
        Ok(())
    }

    #[test]
    fn participating_reviewer_cannot_adjudicate_conflict() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Exclude))?;
        let result = board.resolve(ConflictResolution {
            subject_id: "record-1".to_owned(),
            round: ScreeningRound::TitleAbstract,
            decision_ids: vec!["d-a".to_owned(), "d-b".to_owned()],
            resolved_decision: DecisionValue::Exclude,
            adjudicator_id: "a".to_owned(),
            rationale: "Cannot adjudicate own decision".to_owned(),
            resolved_at: "2026-08-12T00:00:00Z".to_owned(),
        });
        assert_eq!(result, Err(ScreeningError::AdjudicatorNotIndependent));
        assert_eq!(board.history().len(), 2);
        Ok(())
    }

    #[test]
    fn bulk_preview_is_non_mutating_atomic_and_revision_bound() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        let preview = board.preview_bulk(vec![
            decision("a", ReviewerKind::Human, DecisionValue::Include),
            decision("b", ReviewerKind::Human, DecisionValue::Include),
        ])?;
        assert_eq!(preview.accepted(), 2);
        assert_eq!(board.history().len(), 0);
        board.apply_bulk(preview)?;
        assert_eq!(board.history().len(), 2);
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Consensus(DecisionValue::Include)
        );

        let mut second = ScreeningBoard::new(policy())?;
        let stale = second.preview_bulk(vec![decision(
            "b",
            ReviewerKind::Human,
            DecisionValue::Include,
        )])?;
        second.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        assert_eq!(
            second.apply_bulk(stale),
            Err(ScreeningError::StaleBulkPreview)
        );
        assert_eq!(second.history().len(), 1);
        Ok(())
    }

    #[test]
    fn invalid_bulk_preview_leaves_board_unchanged() -> Result<(), ScreeningError> {
        let board = ScreeningBoard::new(policy())?;
        let first = decision("a", ReviewerKind::Human, DecisionValue::Include);
        let mut duplicate_reviewer = decision("a", ReviewerKind::Human, DecisionValue::Exclude);
        duplicate_reviewer.decision_id = "d-a-second".to_owned();
        let duplicate = vec![first, duplicate_reviewer];
        assert!(matches!(
            board.preview_bulk(duplicate),
            Err(ScreeningError::DuplicateReviewer { .. })
        ));
        assert!(board.history().is_empty());
        Ok(())
    }
}
