//! Screening-state rules shared by CLI, MCP and future user interfaces.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use searchright_contracts::{
    AgentAuthority, ConflictResolution, DecisionValue, ExclusionReason, ReviewerKind,
    ScreeningDecision, ScreeningPolicy, ScreeningRound,
};
use serde::{Deserialize, Serialize};

/// Reviewer role in the governed screening workflow.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerRole {
    /// Independent primary screener.
    Screener,
    /// Independent adjudicator for conflicting decisions.
    Adjudicator,
    /// Policy administrator managing protocol rules, amendments and re-screening.
    PolicyAdmin,
}

/// Blinded view of a screening decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlindedDecisionView {
    /// Decision identifier.
    pub decision_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Subject identifier.
    pub subject_id: String,
    /// Screening round.
    pub round: ScreeningRound,
    /// Reviewer identifier or blinded pseudonym.
    pub reviewer_id: String,
    /// Reviewer class.
    pub reviewer_kind: ReviewerKind,
    /// Decision value, or `None` if blinded to the requesting principal.
    pub decision: Option<DecisionValue>,
    /// Exclusion reason, or `None` if blinded.
    pub exclusion_reason: Option<ExclusionReason>,
    /// Confidence for agent recommendations or optional human calibration.
    pub confidence: Option<f64>,
    /// Decision timestamp.
    pub decided_at: String,
    /// Evidence-bearing explanation, or `None` if blinded.
    pub rationale: Option<String>,
    /// Eligibility-contract version applied.
    pub eligibility_version: String,
    /// Model/tool provenance for agent decisions.
    pub agent_provenance: Option<String>,
    /// Whether this view has been blinded/masked.
    pub is_blinded: bool,
}

/// Governed amendment to an existing screening decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScreeningAmendment {
    /// Stable amendment identifier.
    pub amendment_id: String,
    /// Decision identifier of the prior decision being superseded.
    pub target_decision_id: String,
    /// New replacement decision.
    pub updated_decision: ScreeningDecision,
    /// Rationale for the amendment.
    pub amendment_reason: String,
    /// Principal who authorized the amendment.
    pub amended_by: String,
    /// Timestamp of amendment.
    pub amended_at: String,
}

/// Request to re-open a subject/round for re-screening under a new eligibility version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReScreenRequest {
    /// Stable re-screen request identifier.
    pub request_id: String,
    /// Subject identifier to re-screen.
    pub subject_id: String,
    /// Screening round.
    pub round: ScreeningRound,
    /// New eligibility version contract to apply.
    pub new_eligibility_version: String,
    /// Justification/rationale for the re-screen.
    pub reason: String,
    /// Principal requesting the re-screen.
    pub requested_by: String,
    /// Timestamp.
    pub requested_at: String,
}

/// Derived status for a subject and round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    /// A prior decision was amended under governance.
    Amendment(ScreeningAmendment),
    /// A subject/round was re-opened for governed re-screening.
    ReScreen(ReScreenRequest),
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
    roles: BTreeMap<String, BTreeSet<ReviewerRole>>,
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
            roles: BTreeMap::new(),
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

    /// Assign a role to a reviewer.
    pub fn assign_role(&mut self, reviewer_id: &str, role: ReviewerRole) {
        self.roles
            .entry(reviewer_id.to_owned())
            .or_default()
            .insert(role);
    }

    /// Revoke a role from a reviewer.
    pub fn revoke_role(&mut self, reviewer_id: &str, role: ReviewerRole) {
        if let Some(roles) = self.roles.get_mut(reviewer_id) {
            roles.remove(&role);
        }
    }

    /// Query the assigned roles for a reviewer.
    #[must_use]
    pub fn reviewer_roles(&self, reviewer_id: &str) -> BTreeSet<ReviewerRole> {
        self.roles.get(reviewer_id).cloned().unwrap_or_default()
    }

    /// Record an independent decision after authority checks.
    pub fn submit(&mut self, decision: ScreeningDecision) -> Result<(), ScreeningError> {
        self.submit_inner(decision)
    }

    fn submit_inner(&mut self, decision: ScreeningDecision) -> Result<(), ScreeningError> {
        validate_decision(&decision, &self.policy)?;
        if let Some(assigned) = self.roles.get(&decision.reviewer_id)
            && !assigned.is_empty()
            && !assigned.contains(&ReviewerRole::Screener)
        {
            return Err(ScreeningError::UnauthorizedRole {
                reviewer_id: decision.reviewer_id,
                required_role: "screener",
            });
        }
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
        if let Some(assigned) = self.roles.get(&resolution.adjudicator_id)
            && !assigned.is_empty()
            && !assigned.contains(&ReviewerRole::Adjudicator)
        {
            return Err(ScreeningError::UnauthorizedRole {
                reviewer_id: resolution.adjudicator_id,
                required_role: "adjudicator",
            });
        }
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

    /// Govern an amendment to a prior screening decision.
    pub fn amend_decision(&mut self, amendment: ScreeningAmendment) -> Result<(), ScreeningError> {
        require_screening_text(&amendment.amendment_id, "amendment.amendment_id")?;
        require_screening_text(
            &amendment.target_decision_id,
            "amendment.target_decision_id",
        )?;
        require_screening_text(&amendment.amendment_reason, "amendment.amendment_reason")?;
        require_screening_text(&amendment.amended_by, "amendment.amended_by")?;
        require_screening_text(&amendment.amended_at, "amendment.amended_at")?;

        if let Some(assigned) = self.roles.get(&amendment.amended_by)
            && !assigned.is_empty()
            && !assigned.contains(&ReviewerRole::PolicyAdmin)
            && !assigned.contains(&ReviewerRole::Screener)
        {
            return Err(ScreeningError::UnauthorizedRole {
                reviewer_id: amendment.amended_by.clone(),
                required_role: "policy_admin_or_screener",
            });
        }

        validate_decision(&amendment.updated_decision, &self.policy)?;

        let key = (
            amendment.updated_decision.subject_id.clone(),
            amendment.updated_decision.round,
        );
        if self.resolutions.contains_key(&key) {
            return Err(ScreeningError::RoundFinalized);
        }

        let existing =
            self.decisions
                .get_mut(&key)
                .ok_or_else(|| ScreeningError::UnknownTargetDecision {
                    decision_id: amendment.target_decision_id.clone(),
                })?;

        let pos = existing
            .iter()
            .position(|d| d.decision_id == amendment.target_decision_id)
            .ok_or_else(|| ScreeningError::UnknownTargetDecision {
                decision_id: amendment.target_decision_id.clone(),
            })?;

        let target_item =
            existing
                .get_mut(pos)
                .ok_or_else(|| ScreeningError::UnknownTargetDecision {
                    decision_id: amendment.target_decision_id.clone(),
                })?;

        if target_item.reviewer_id != amendment.updated_decision.reviewer_id {
            return Err(ScreeningError::ReviewerMismatch);
        }

        *target_item = amendment.updated_decision.clone();
        self.history
            .push(ScreeningHistoryEntry::Amendment(amendment));
        self.revision += 1;
        Ok(())
    }

    /// Re-open a subject and round for governed re-screening under a new eligibility version.
    pub fn rescreen_subject(&mut self, request: ReScreenRequest) -> Result<(), ScreeningError> {
        require_screening_text(&request.request_id, "rescreen.request_id")?;
        require_screening_text(&request.subject_id, "rescreen.subject_id")?;
        require_screening_text(
            &request.new_eligibility_version,
            "rescreen.new_eligibility_version",
        )?;
        require_screening_text(&request.reason, "rescreen.reason")?;
        require_screening_text(&request.requested_by, "rescreen.requested_by")?;
        require_screening_text(&request.requested_at, "rescreen.requested_at")?;

        if let Some(assigned) = self.roles.get(&request.requested_by)
            && !assigned.is_empty()
            && !assigned.contains(&ReviewerRole::PolicyAdmin)
        {
            return Err(ScreeningError::UnauthorizedRole {
                reviewer_id: request.requested_by.clone(),
                required_role: "policy_admin",
            });
        }

        let key = (request.subject_id.clone(), request.round);
        self.resolutions.remove(&key);
        self.decisions.remove(&key);

        self.history.push(ScreeningHistoryEntry::ReScreen(request));
        self.revision += 1;
        Ok(())
    }

    /// Project decisions for a requesting principal with independent blinding applied.
    #[must_use]
    pub fn decisions_for_reviewer(
        &self,
        requesting_reviewer_id: &str,
        round: Option<ScreeningRound>,
    ) -> Vec<BlindedDecisionView> {
        let requesting_roles = self.reviewer_roles(requesting_reviewer_id);
        let is_admin = requesting_roles.contains(&ReviewerRole::PolicyAdmin);

        let mut views = Vec::new();
        for ((subject_id, dec_round), decisions) in &self.decisions {
            if let Some(target_round) = round
                && *dec_round != target_round
            {
                continue;
            }
            let status = self.status(subject_id, *dec_round);
            let round_unblinded = matches!(
                status,
                ScreeningStatus::Consensus(_) | ScreeningStatus::Resolved(_)
            );

            for decision in decisions {
                let is_self = decision.reviewer_id == requesting_reviewer_id;
                let should_reveal =
                    !self.policy.independent_blinding || is_self || round_unblinded || is_admin;

                views.push(BlindedDecisionView {
                    decision_id: decision.decision_id.clone(),
                    review_id: decision.review_id.clone(),
                    subject_id: decision.subject_id.clone(),
                    round: decision.round,
                    reviewer_id: if should_reveal {
                        decision.reviewer_id.clone()
                    } else {
                        "blinded-reviewer".to_owned()
                    },
                    reviewer_kind: decision.reviewer_kind.clone(),
                    decision: if should_reveal {
                        Some(decision.decision)
                    } else {
                        None
                    },
                    exclusion_reason: if should_reveal {
                        decision.exclusion_reason.clone()
                    } else {
                        None
                    },
                    confidence: if should_reveal {
                        decision.confidence
                    } else {
                        None
                    },
                    decided_at: decision.decided_at.clone(),
                    rationale: if should_reveal {
                        Some(decision.rationale.clone())
                    } else {
                        None
                    },
                    eligibility_version: decision.eligibility_version.clone(),
                    agent_provenance: if should_reveal {
                        decision.agent_provenance.clone()
                    } else {
                        None
                    },
                    is_blinded: !should_reveal,
                });
            }
        }
        views
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

/// Validate a decision against configured operational criteria.
pub fn validate_decision_against_criteria(
    decision: &ScreeningDecision,
    allowed_criteria: &[(&str, &str)],
) -> Result<(), ScreeningError> {
    if let Some(reason) = &decision.exclusion_reason {
        let matched = allowed_criteria
            .iter()
            .any(|(c_id, r_id)| *c_id == reason.criterion_id && *r_id == reason.reason_id);
        if !matched && !allowed_criteria.is_empty() {
            return Err(ScreeningError::CriterionMismatch {
                criterion_id: reason.criterion_id.clone(),
                reason_id: reason.reason_id.clone(),
            });
        }
    }
    Ok(())
}

/// Export decisions to Covidence-compatible CSV format.
#[must_use]
pub fn export_covidence_csv(board: &ScreeningBoard) -> String {
    let mut csv = String::from(
        "Study ID,Round,Reviewer ID,Decision,Exclusion Reason,Rationale,Eligibility Version,Decided At\n",
    );
    for decision in board.decisions() {
        let dec_str = match decision.decision {
            DecisionValue::Include => "Include",
            DecisionValue::Exclude => "Exclude",
            DecisionValue::Unclear => "Unclear",
        };
        let reason_str = decision
            .exclusion_reason
            .as_ref()
            .map_or("", |r| r.label.as_str());
        let round_str = match decision.round {
            ScreeningRound::TitleAbstract => "Title/Abstract",
            ScreeningRound::FullText => "Full Text",
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            escape_csv(&decision.subject_id),
            escape_csv(round_str),
            escape_csv(&decision.reviewer_id),
            escape_csv(dec_str),
            escape_csv(reason_str),
            escape_csv(&decision.rationale),
            escape_csv(&decision.eligibility_version),
            escape_csv(&decision.decided_at),
        ));
    }
    csv
}

/// Export decisions to Rayyan-compatible CSV format.
#[must_use]
pub fn export_rayyan_csv(board: &ScreeningBoard) -> String {
    let mut csv = String::from(
        "key,rayyan_round,user,rayyan_decision,rayyan_reason,rayyan_notes,eligibility_version,decided_at\n",
    );
    for decision in board.decisions() {
        let dec_str = match decision.decision {
            DecisionValue::Include => "Included",
            DecisionValue::Exclude => "Excluded",
            DecisionValue::Unclear => "Maybe",
        };
        let reason_str = decision
            .exclusion_reason
            .as_ref()
            .map_or("", |r| r.label.as_str());
        let round_str = match decision.round {
            ScreeningRound::TitleAbstract => "title_abstract",
            ScreeningRound::FullText => "full_text",
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            escape_csv(&decision.subject_id),
            escape_csv(round_str),
            escape_csv(&decision.reviewer_id),
            escape_csv(dec_str),
            escape_csv(reason_str),
            escape_csv(&decision.rationale),
            escape_csv(&decision.eligibility_version),
            escape_csv(&decision.decided_at),
        ));
    }
    csv
}

/// Export decisions to EPPI-Reviewer compatible JSON structure.
pub fn export_eppi_reviewer_json(board: &ScreeningBoard) -> Result<String, ScreeningError> {
    let decisions: Vec<&ScreeningDecision> = board.decisions().collect();
    serde_json::to_string_pretty(&decisions).map_err(|err| ScreeningError::SerializationError {
        message: err.to_string(),
    })
}

/// Import decisions from Covidence-compatible CSV format.
pub fn import_covidence_csv(
    csv_content: &str,
    review_id: &str,
    round: ScreeningRound,
    eligibility_version: &str,
) -> Result<Vec<ScreeningDecision>, ScreeningError> {
    let mut rows = csv_records(csv_content);
    let header = rows
        .next()
        .transpose()
        .map_err(csv_import_error)?
        .ok_or_else(|| ScreeningError::InvalidImportData {
            reason: "empty CSV file".to_owned(),
        })?;
    if !header
        .iter()
        .any(|field| field.contains("Study ID") || field.contains("Decision"))
    {
        return Err(ScreeningError::InvalidImportData {
            reason: "invalid Covidence CSV header".to_owned(),
        });
    }

    let mut decisions = Vec::new();
    for row in rows {
        let cols = row.map_err(csv_import_error)?;
        if cols.len() == 1 && cols.get(0).is_some_and(|field| field.trim().is_empty()) {
            continue;
        }
        if cols.len() < 4 {
            let line_num = csv_start_line(csv_content, &cols);
            return Err(ScreeningError::InvalidImportData {
                reason: format!("insufficient columns on line {line_num}"),
            });
        }
        let subject_id = cols.get(0).unwrap_or_default().trim().to_owned();
        let reviewer_id = cols
            .get(2)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("imported-reviewer")
            .to_owned();
        let dec_str = cols.get(3).map_or("", str::trim).to_lowercase();
        let decision_val = match dec_str.as_str() {
            "include" | "included" => DecisionValue::Include,
            "exclude" | "excluded" => DecisionValue::Exclude,
            "unclear" | "maybe" => DecisionValue::Unclear,
            other => {
                let line_num = csv_start_line(csv_content, &cols);
                return Err(ScreeningError::InvalidImportData {
                    reason: format!("unrecognized decision `{other}` on line {line_num}"),
                });
            }
        };
        let reason_str = cols.get(4).map_or("", str::trim);
        let rationale = cols
            .get(5)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Imported from Covidence");
        let decided_at = cols
            .get(7)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("2026-08-01T00:00:00Z")
            .to_owned();

        let exclusion_reason = if decision_val == DecisionValue::Exclude {
            Some(ExclusionReason {
                reason_id: if reason_str.is_empty() {
                    "unspecified-exclusion".to_owned()
                } else {
                    slugify(reason_str)
                },
                criterion_id: "eligibility".to_owned(),
                label: if reason_str.is_empty() {
                    "Unspecified exclusion".to_owned()
                } else {
                    reason_str.to_owned()
                },
                evidence: None,
            })
        } else {
            None
        };

        decisions.push(ScreeningDecision {
            decision_id: format!("covidence-import-{subject_id}-{reviewer_id}"),
            review_id: review_id.to_owned(),
            subject_id,
            round,
            reviewer_id,
            reviewer_kind: ReviewerKind::Human,
            decision: decision_val,
            exclusion_reason,
            confidence: None,
            decided_at,
            rationale: rationale.to_owned(),
            eligibility_version: eligibility_version.to_owned(),
            agent_provenance: None,
        });
    }
    Ok(decisions)
}

/// Import decisions from Rayyan-compatible CSV format.
pub fn import_rayyan_csv(
    csv_content: &str,
    review_id: &str,
    round: ScreeningRound,
    eligibility_version: &str,
) -> Result<Vec<ScreeningDecision>, ScreeningError> {
    let mut rows = csv_records(csv_content);
    let header = rows
        .next()
        .transpose()
        .map_err(csv_import_error)?
        .ok_or_else(|| ScreeningError::InvalidImportData {
            reason: "empty CSV file".to_owned(),
        })?;
    if !header
        .iter()
        .any(|field| field.contains("key") || field.contains("rayyan_decision"))
    {
        return Err(ScreeningError::InvalidImportData {
            reason: "invalid Rayyan CSV header".to_owned(),
        });
    }

    let mut decisions = Vec::new();
    for row in rows {
        let cols = row.map_err(csv_import_error)?;
        if cols.len() == 1 && cols.get(0).is_some_and(|field| field.trim().is_empty()) {
            continue;
        }
        if cols.len() < 4 {
            let line_num = csv_start_line(csv_content, &cols);
            return Err(ScreeningError::InvalidImportData {
                reason: format!("insufficient columns on line {line_num}"),
            });
        }
        let subject_id = cols.get(0).unwrap_or_default().trim().to_owned();
        let reviewer_id = cols
            .get(2)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("imported-reviewer")
            .to_owned();
        let dec_str = cols.get(3).map_or("", str::trim).to_lowercase();
        let decision_val = match dec_str.as_str() {
            "included" | "include" => DecisionValue::Include,
            "excluded" | "exclude" => DecisionValue::Exclude,
            "maybe" | "unclear" => DecisionValue::Unclear,
            other => {
                let line_num = csv_start_line(csv_content, &cols);
                return Err(ScreeningError::InvalidImportData {
                    reason: format!("unrecognized decision `{other}` on line {line_num}"),
                });
            }
        };
        let reason_str = cols.get(4).map_or("", str::trim);
        let notes = cols
            .get(5)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Imported from Rayyan");
        let decided_at = cols
            .get(7)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("2026-08-01T00:00:00Z")
            .to_owned();

        let exclusion_reason = if decision_val == DecisionValue::Exclude {
            Some(ExclusionReason {
                reason_id: if reason_str.is_empty() {
                    "unspecified-exclusion".to_owned()
                } else {
                    slugify(reason_str)
                },
                criterion_id: "eligibility".to_owned(),
                label: if reason_str.is_empty() {
                    "Unspecified exclusion".to_owned()
                } else {
                    reason_str.to_owned()
                },
                evidence: None,
            })
        } else {
            None
        };

        decisions.push(ScreeningDecision {
            decision_id: format!("rayyan-import-{subject_id}-{reviewer_id}"),
            review_id: review_id.to_owned(),
            subject_id,
            round,
            reviewer_id,
            reviewer_kind: ReviewerKind::Human,
            decision: decision_val,
            exclusion_reason,
            confidence: None,
            decided_at,
            rationale: notes.to_owned(),
            eligibility_version: eligibility_version.to_owned(),
            agent_provenance: None,
        });
    }
    Ok(decisions)
}

fn escape_csv(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        val.to_owned()
    }
}

fn csv_records(input: &str) -> impl Iterator<Item = csv::Result<csv::StringRecord>> + '_ {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(input.as_bytes())
        .into_records()
}

fn csv_import_error(error: csv::Error) -> ScreeningError {
    ScreeningError::InvalidImportData {
        reason: error.to_string(),
    }
}

fn csv_start_line(input: &str, record: &csv::StringRecord) -> usize {
    // Reader positions can precede skipped blank lines or the LF of CRLF.
    // Count original line endings only on error paths, without re-parsing CSV.
    let offset = record
        .position()
        .and_then(|position| usize::try_from(position.byte()).ok())
        .unwrap_or_default();
    let bytes = input.as_bytes();
    let skipped = bytes
        .get(offset..)
        .unwrap_or_default()
        .iter()
        .take_while(|&&byte| matches!(byte, b'\r' | b'\n'))
        .count();
    let prefix = bytes
        .get(..offset.saturating_add(skipped))
        .unwrap_or_default();
    let mut line = 1_usize;
    let mut after_cr = false;
    for &byte in prefix {
        if byte == b'\r' || (byte == b'\n' && !after_cr) {
            line = line.saturating_add(1);
        }
        after_cr = byte == b'\r';
    }
    line
}

fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
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
    /// Principal attempted an operation without the required role assignment.
    #[error("reviewer `{reviewer_id}` does not have required role `{required_role}`")]
    UnauthorizedRole {
        /// Identifier of reviewer.
        reviewer_id: String,
        /// Required role name.
        required_role: &'static str,
    },
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
    /// Target decision for amendment was not found.
    #[error("target decision `{decision_id}` not found on board")]
    UnknownTargetDecision {
        /// Target decision ID.
        decision_id: String,
    },
    /// Reviewer ID in amendment did not match original decision reviewer.
    #[error("reviewer ID in amended decision does not match original decision reviewer")]
    ReviewerMismatch,
    /// Criterion or reason does not match allowed criteria for this review.
    #[error(
        "exclusion reason `{reason_id}` for criterion `{criterion_id}` is not in allowed review criteria"
    )]
    CriterionMismatch {
        /// Criterion ID.
        criterion_id: String,
        /// Reason ID.
        reason_id: String,
    },
    /// Import data format error.
    #[error("failed to import external screening data: {reason}")]
    InvalidImportData {
        /// Details.
        reason: String,
    },
    /// Serialization error.
    #[error("failed to serialize screening data: {message}")]
    SerializationError {
        /// Details.
        message: String,
    },
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

/// Return whether a decision represents exclusion at the final-eligibility boundary.
#[must_use]
pub const fn is_exclusion_decision(decision: DecisionValue) -> bool {
    matches!(decision, DecisionValue::Exclude)
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
    fn invalid_policy_is_rejected() -> Result<(), ScreeningError> {
        let mut invalid = policy();
        invalid.full_text_reviewers = 0;
        assert!(matches!(
            ScreeningBoard::new(invalid),
            Err(ScreeningError::InvalidReviewerCount)
        ));
        Ok(())
    }

    #[test]
    fn agent_cannot_exclude_by_default() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        let result = board.submit(decision(
            "agent",
            ReviewerKind::Agent,
            DecisionValue::Exclude,
        ));
        assert_eq!(result, Err(ScreeningError::AgentExclusionDenied));
        Ok(())
    }

    #[test]
    fn disagreement_creates_conflict() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Exclude))?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Conflict
        );
        Ok(())
    }

    #[test]
    fn blinded_view_hides_other_reviewer_decisions_until_unblinded() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;

        let view_a = board.decisions_for_reviewer("a", Some(ScreeningRound::TitleAbstract));
        assert_eq!(view_a.len(), 1);
        let first_a = view_a
            .first()
            .ok_or(ScreeningError::EmptyField { field: "view_a" })?;
        assert_eq!(first_a.decision, Some(DecisionValue::Include));
        assert!(!first_a.is_blinded);

        let view_b = board.decisions_for_reviewer("b", Some(ScreeningRound::TitleAbstract));
        assert_eq!(view_b.len(), 1);
        let first_b = view_b
            .first()
            .ok_or(ScreeningError::EmptyField { field: "view_b" })?;
        assert_eq!(first_b.decision, None);
        assert_eq!(first_b.rationale, None);
        assert!(first_b.is_blinded);

        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Include))?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Consensus(DecisionValue::Include)
        );

        let view_b_unblinded =
            board.decisions_for_reviewer("b", Some(ScreeningRound::TitleAbstract));
        assert_eq!(view_b_unblinded.len(), 2);
        assert!(view_b_unblinded.iter().all(|v| !v.is_blinded));
        let unblinded_0 = view_b_unblinded.first().ok_or(ScreeningError::EmptyField {
            field: "view_b_unblinded",
        })?;
        let unblinded_1 = view_b_unblinded.get(1).ok_or(ScreeningError::EmptyField {
            field: "view_b_unblinded",
        })?;
        assert_eq!(unblinded_0.decision, Some(DecisionValue::Include));
        assert_eq!(unblinded_1.decision, Some(DecisionValue::Include));
        Ok(())
    }

    #[test]
    fn reviewer_role_enforcement() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.assign_role("screener-1", ReviewerRole::Screener);
        board.assign_role("adjudicator-1", ReviewerRole::Adjudicator);

        let mut dec = decision("adjudicator-1", ReviewerKind::Human, DecisionValue::Include);
        dec.decision_id = "d-adj".to_owned();
        let submit_err = board.submit(dec);
        assert!(matches!(
            submit_err,
            Err(ScreeningError::UnauthorizedRole {
                required_role: "screener",
                ..
            })
        ));

        let mut screener_dec = decision("screener-1", ReviewerKind::Human, DecisionValue::Include);
        screener_dec.decision_id = "d-scr".to_owned();
        board.submit(screener_dec)?;
        Ok(())
    }

    #[test]
    fn criteria_validation_rejects_unlisted_reasons() -> Result<(), ScreeningError> {
        let dec = decision("a", ReviewerKind::Human, DecisionValue::Exclude);
        let allowed = [("population", "wrong-population")];
        assert!(validate_decision_against_criteria(&dec, &allowed).is_ok());

        let invalid_allowed = [("intervention", "wrong-intervention")];
        assert!(matches!(
            validate_decision_against_criteria(&dec, &invalid_allowed),
            Err(ScreeningError::CriterionMismatch { .. })
        ));
        Ok(())
    }

    #[test]
    fn governed_amendment_preserves_history() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;

        let mut updated = decision("a", ReviewerKind::Human, DecisionValue::Exclude);
        updated.decision_id = "d-a-amended".to_owned();
        let amendment = ScreeningAmendment {
            amendment_id: "amend-001".to_owned(),
            target_decision_id: "d-a".to_owned(),
            updated_decision: updated,
            amendment_reason: "Found paediatric exclusion criterion in appendix".to_owned(),
            amended_by: "a".to_owned(),
            amended_at: "2026-08-10T00:00:00Z".to_owned(),
        };

        board.amend_decision(amendment)?;
        assert_eq!(board.history().len(), 2);
        assert!(matches!(
            board.history().last(),
            Some(ScreeningHistoryEntry::Amendment(_))
        ));

        let current: Vec<&ScreeningDecision> = board.decisions().collect();
        assert_eq!(current.len(), 1);
        let first_current = current
            .first()
            .ok_or(ScreeningError::EmptyField { field: "current" })?;
        assert_eq!(first_current.decision, DecisionValue::Exclude);
        Ok(())
    }

    #[test]
    fn governed_rescreen_resets_round_state() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.assign_role("admin-1", ReviewerRole::PolicyAdmin);
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Include))?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Consensus(DecisionValue::Include)
        );

        let rescreen = ReScreenRequest {
            request_id: "rescreen-001".to_owned(),
            subject_id: "record-1".to_owned(),
            round: ScreeningRound::TitleAbstract,
            new_eligibility_version: "2.0".to_owned(),
            reason: "Protocol amendment broadening eligible population".to_owned(),
            requested_by: "admin-1".to_owned(),
            requested_at: "2026-08-20T00:00:00Z".to_owned(),
        };

        board.rescreen_subject(rescreen)?;
        assert_eq!(
            board.status("record-1", ScreeningRound::TitleAbstract),
            ScreeningStatus::Pending {
                submitted: 0,
                required: 2,
            }
        );
        assert!(matches!(
            board.history().last(),
            Some(ScreeningHistoryEntry::ReScreen(_))
        ));
        Ok(())
    }

    #[test]
    fn covidence_and_rayyan_export_import_roundtrip() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        board.submit(decision("a", ReviewerKind::Human, DecisionValue::Include))?;
        board.submit(decision("b", ReviewerKind::Human, DecisionValue::Exclude))?;

        let cov_csv = export_covidence_csv(&board);
        assert!(cov_csv.contains("record-1"));
        assert!(cov_csv.contains("Wrong population"));

        let imported_cov =
            import_covidence_csv(&cov_csv, "r1", ScreeningRound::TitleAbstract, "1")?;
        assert_eq!(imported_cov.len(), 2);
        let first_cov = imported_cov.first().ok_or(ScreeningError::EmptyField {
            field: "imported_cov",
        })?;
        let second_cov = imported_cov.get(1).ok_or(ScreeningError::EmptyField {
            field: "imported_cov",
        })?;
        assert_eq!(first_cov.decision, DecisionValue::Include);
        assert_eq!(second_cov.decision, DecisionValue::Exclude);

        let rayyan_csv = export_rayyan_csv(&board);
        assert!(rayyan_csv.contains("record-1"));
        assert!(rayyan_csv.contains("Wrong population"));

        let imported_rayyan =
            import_rayyan_csv(&rayyan_csv, "r1", ScreeningRound::TitleAbstract, "1")?;
        assert_eq!(imported_rayyan.len(), 2);

        let eppi_json = export_eppi_reviewer_json(&board)?;
        assert!(eppi_json.contains("wrong-population"));
        Ok(())
    }

    type CsvImporter =
        fn(&str, &str, ScreeningRound, &str) -> Result<Vec<ScreeningDecision>, ScreeningError>;

    fn csv_importers() -> [(&'static str, CsvImporter); 2] {
        [
            (
                "Study ID,Round,Reviewer ID,Decision,Exclusion Reason,Rationale,Eligibility Version,Decided At",
                import_covidence_csv,
            ),
            (
                "key,rayyan_round,user,rayyan_decision,rayyan_reason,rayyan_notes,eligibility_version,decided_at",
                import_rayyan_csv,
            ),
        ]
    }

    #[test]
    fn csv_exports_round_trip_multiline_rationale_and_reason() -> Result<(), ScreeningError> {
        let mut board = ScreeningBoard::new(policy())?;
        let mut original = decision("reviewer", ReviewerKind::Human, DecisionValue::Exclude);
        original.rationale = "First, line\r\nSecond \"quoted\" line".to_owned();
        let reason = original
            .exclusion_reason
            .as_mut()
            .ok_or(ScreeningError::MissingExclusionReason)?;
        reason.label = "Wrong, population\nand \"setting\"".to_owned();
        board.submit(original.clone())?;

        for (input, importer) in [
            (
                export_covidence_csv(&board),
                import_covidence_csv as CsvImporter,
            ),
            (export_rayyan_csv(&board), import_rayyan_csv as CsvImporter),
        ] {
            let imported = importer(&input, "r1", ScreeningRound::TitleAbstract, "1")?;
            assert_eq!(imported.len(), 1);
            let restored = imported.first().ok_or(ScreeningError::EmptyField {
                field: "imported_csv",
            })?;
            assert_eq!(restored.rationale, original.rationale);
            assert_eq!(
                restored
                    .exclusion_reason
                    .as_ref()
                    .map(|reason| &reason.label),
                original
                    .exclusion_reason
                    .as_ref()
                    .map(|reason| &reason.label)
            );
            assert_eq!(restored.subject_id, original.subject_id);
            assert_eq!(restored.reviewer_id, original.reviewer_id);
            assert_eq!(restored.decision, original.decision);
            assert_eq!(restored.decided_at, original.decided_at);
        }
        Ok(())
    }

    #[test]
    fn csv_imports_accept_bom_crlf_blank_lines_and_variable_width() -> Result<(), ScreeningError> {
        for (header, importer) in csv_importers() {
            let input = format!(
                "\u{feff}{header}\r\n\r\n \r\n\
                 record-1,round,reviewer,Include\r\n\
                 record-2,round,reviewer,Include,,\"First line\r\nSecond, \"\"quoted\"\" line\",1,2026-08-01T00:00:00Z,extra\r\n"
            );
            let imported = importer(&input, "r1", ScreeningRound::TitleAbstract, "1")?;
            assert_eq!(imported.len(), 2);
            let second = imported.get(1).ok_or(ScreeningError::EmptyField {
                field: "second_imported_csv",
            })?;
            assert_eq!(second.subject_id, "record-2");
            assert_eq!(second.rationale, "First line\r\nSecond, \"quoted\" line");
            assert_eq!(second.eligibility_version, "1");
        }
        Ok(())
    }

    #[test]
    fn csv_errors_after_multiline_records_report_physical_start_lines() {
        for (header, importer) in csv_importers() {
            for newline in ["\n", "\r\n", "\r"] {
                let prefix = format!(
                    "{header}{newline}record-1,round,reviewer,Include,,\"First line{newline}Second line\"{newline}"
                );
                let input = format!("{prefix}record-2,round,reviewer,unknown{newline}");
                assert!(matches!(
                    importer(&input, "r1", ScreeningRound::TitleAbstract, "1"),
                    Err(ScreeningError::InvalidImportData { reason })
                        if reason == "unrecognized decision `unknown` on line 4"
                ));
                let input = format!("{prefix}too,short{newline}");
                assert!(matches!(
                    importer(&input, "r1", ScreeningRound::TitleAbstract, "1"),
                    Err(ScreeningError::InvalidImportData { reason })
                        if reason == "insufficient columns on line 4"
                ));
            }
        }
    }

    #[test]
    fn csv_error_lines_handle_blank_records_and_all_line_endings() {
        for (header, importer) in csv_importers() {
            for newline in ["\n", "\r\n", "\r"] {
                for (padding, expected_line) in [(String::new(), 2), (newline.repeat(2), 4)] {
                    let input = format!(
                        "{header}{newline}{padding}record-1,round,reviewer,unknown{newline}"
                    );
                    assert!(matches!(
                        importer(&input, "r1", ScreeningRound::TitleAbstract, "1"),
                        Err(ScreeningError::InvalidImportData { reason })
                            if reason == format!("unrecognized decision `unknown` on line {expected_line}")
                    ));
                }
            }
        }
    }

    #[test]
    fn csv_imports_reject_empty_invalid_headers_and_empty_decisions() {
        for (header, importer) in csv_importers() {
            for input in ["", "\n\r\n", "\u{feff}", "unrecognized,header\n"] {
                assert!(matches!(
                    importer(input, "r1", ScreeningRound::TitleAbstract, "1"),
                    Err(ScreeningError::InvalidImportData { .. })
                ));
            }
            let input = format!("{header}\n,,,\n");
            assert!(matches!(
                importer(&input, "r1", ScreeningRound::TitleAbstract, "1"),
                Err(ScreeningError::InvalidImportData { reason })
                    if reason == "unrecognized decision `` on line 2"
            ));
        }
    }

    #[test]
    fn csv_unclosed_quotes_follow_the_upstream_permissive_boundary() -> Result<(), ScreeningError> {
        for (header, importer) in csv_importers() {
            let input = format!(
                "{header}\nrecord-1,round,reviewer,Include,,\"Unclosed rationale\nrecord-2,round,reviewer,Exclude\n"
            );
            let imported = importer(&input, "r1", ScreeningRound::TitleAbstract, "1")?;
            assert_eq!(imported.len(), 1);
            let first = imported.first().ok_or(ScreeningError::EmptyField {
                field: "imported_csv",
            })?;
            assert_eq!(first.subject_id, "record-1");
            assert!(first.rationale.contains("record-2,round,reviewer,Exclude"));
        }
        Ok(())
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
        assert_eq!(board.history(), []);
        Ok(())
    }
}
