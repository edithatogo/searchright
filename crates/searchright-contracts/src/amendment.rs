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
        for change in &self.changes {
            change.validate()?;
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
