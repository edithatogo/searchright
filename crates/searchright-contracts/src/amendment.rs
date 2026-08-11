use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, PROTOCOL_AMENDMENT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// Kind of protocol amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AmendmentKind {
    /// Research question or scope changed.
    Scope,
    /// Eligibility criteria changed.
    Eligibility,
    /// Information sources changed.
    InformationSources,
    /// Search strategy changed.
    SearchStrategy,
    /// Screening process changed.
    Screening,
    /// Analysis or synthesis changed.
    Analysis,
    /// Governance or authorship changed.
    Governance,
    /// Another declared amendment kind.
    Other(String),
}

/// One machine-readable change within an amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AmendmentChange {
    /// JSON Pointer or stable contract path.
    pub path: String,
    /// Prior value represented as JSON text or a concise description.
    pub before: Option<String>,
    /// Replacement value represented as JSON text or a concise description.
    pub after: String,
    /// Rationale for this change.
    pub rationale: String,
}

/// Approval state for an amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AmendmentDecision {
    /// Proposed but not yet approved.
    Proposed,
    /// Approved for prospective use.
    Approved,
    /// Rejected.
    Rejected,
    /// Withdrawn by the review team.
    Withdrawn,
}

/// Auditable protocol amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolAmendment {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable amendment identifier.
    pub amendment_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Amendment type.
    pub kind: AmendmentKind,
    /// Protocol version before the change.
    pub version_before: String,
    /// Protocol version after the change.
    pub version_after: String,
    /// RFC 3339 proposal timestamp.
    pub proposed_at: String,
    /// Proposer identifier.
    pub proposed_by: String,
    /// Approval state.
    pub decision: AmendmentDecision,
    /// Approver identifier when decided.
    pub decided_by: Option<String>,
    /// RFC 3339 decision timestamp when decided.
    pub decided_at: Option<String>,
    /// Structured changes.
    pub changes: Vec<AmendmentChange>,
    /// Expected effect on already completed work.
    pub retrospective_impact: String,
    /// Whether affected prior records/runs must be reprocessed.
    pub requires_reprocessing: bool,
}

impl Validate for AmendmentChange {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.path, "amendment.changes.path")?;
        require_text(&self.after, "amendment.changes.after")?;
        require_text(&self.rationale, "amendment.changes.rationale")?;
        if let Some(before) = &self.before {
            require_text(before, "amendment.changes.before")?;
        }
        Ok(())
    }
}

impl Validate for ProtocolAmendment {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            PROTOCOL_AMENDMENT_SCHEMA_VERSION,
            "amendment.schema_version",
        )?;
        require_text(&self.amendment_id, "amendment.amendment_id")?;
        require_text(&self.review_id, "amendment.review_id")?;
        require_text(&self.version_before, "amendment.version_before")?;
        require_text(&self.version_after, "amendment.version_after")?;
        require_text(&self.proposed_at, "amendment.proposed_at")?;
        require_text(&self.proposed_by, "amendment.proposed_by")?;
        require_text(&self.retrospective_impact, "amendment.retrospective_impact")?;
        if self.version_before == self.version_after {
            return Err(ContractError::Invariant(
                "protocol amendment must change the protocol version".to_owned(),
            ));
        }
        if self.changes.is_empty() {
            return Err(ContractError::EmptyCollection("amendment.changes"));
        }
        let mut change_paths = BTreeSet::new();
        for change in &self.changes {
            change.validate()?;
            if !change_paths.insert(change.path.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "protocol amendment contains duplicate change path `{}`",
                    change.path
                )));
            }
        }
        match self.decision {
            AmendmentDecision::Proposed => {
                if self.decided_by.is_some() || self.decided_at.is_some() {
                    return Err(ContractError::Invariant(
                        "proposed amendments must not contain decision metadata".to_owned(),
                    ));
                }
            }
            AmendmentDecision::Approved
            | AmendmentDecision::Rejected
            | AmendmentDecision::Withdrawn => {
                let decided_by = self.decided_by.as_deref().ok_or_else(|| {
                    ContractError::Invariant(
                        "decided amendment must identify the decision maker".to_owned(),
                    )
                })?;
                let decided_at = self.decided_at.as_deref().ok_or_else(|| {
                    ContractError::Invariant(
                        "decided amendment must contain a decision timestamp".to_owned(),
                    )
                })?;
                require_text(decided_by, "amendment.decided_by")?;
                require_text(decided_at, "amendment.decided_at")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amendment(decision: AmendmentDecision) -> ProtocolAmendment {
        ProtocolAmendment {
            schema_version: PROTOCOL_AMENDMENT_SCHEMA_VERSION.to_owned(),
            amendment_id: "amendment-1".to_owned(),
            review_id: "review-1".to_owned(),
            kind: AmendmentKind::Eligibility,
            version_before: "1.0".to_owned(),
            version_after: "1.1".to_owned(),
            proposed_at: "2026-08-12T00:00:00Z".to_owned(),
            proposed_by: "reviewer-1".to_owned(),
            decision,
            decided_by: None,
            decided_at: None,
            changes: vec![AmendmentChange {
                path: "/eligibility/population".to_owned(),
                before: Some("adults".to_owned()),
                after: "adults and adolescents".to_owned(),
                rationale: "Protocol clarification".to_owned(),
            }],
            retrospective_impact: "Previously screened records require reassessment".to_owned(),
            requires_reprocessing: true,
        }
    }

    #[test]
    fn proposed_amendment_rejects_decision_metadata() {
        let mut value = amendment(AmendmentDecision::Proposed);
        value.decided_by = Some("reviewer-2".to_owned());
        assert!(
            matches!(value.validate(), Err(ContractError::Invariant(message)) if message.contains("must not contain decision metadata"))
        );
    }

    #[test]
    fn decided_amendment_requires_complete_decision_metadata() {
        let mut value = amendment(AmendmentDecision::Approved);
        value.decided_by = Some("reviewer-2".to_owned());
        assert!(
            matches!(value.validate(), Err(ContractError::Invariant(message)) if message.contains("decision timestamp"))
        );
    }

    #[test]
    fn amendment_rejects_duplicate_change_paths() {
        let mut value = amendment(AmendmentDecision::Proposed);
        value.changes.push(AmendmentChange {
            path: "/eligibility/population".to_owned(),
            before: Some("adults".to_owned()),
            after: "adults and adolescents".to_owned(),
            rationale: "Second conflicting change".to_owned(),
        });
        assert!(
            matches!(value.validate(), Err(ContractError::Invariant(message)) if message.contains("duplicate change path"))
        );
    }
}
