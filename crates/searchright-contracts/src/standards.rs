use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, STANDARD_ASSESSMENT_SCHEMA_VERSION, STANDARD_PACK_SCHEMA_VERSION, Validate,
    require_schema_version,
    require_text,
};

/// Reporting or conduct standard family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StandardFamily {
    /// PRISMA 2020.
    Prisma2020,
    /// PRISMA-S.
    PrismaS,
    /// PRISMA-ScR.
    PrismaScR,
    /// PRISMA for living systematic reviews.
    PrismaLsr,
    /// PRISMA-P.
    PrismaP,
    /// PRESS 2015.
    Press2015,
    /// Cochrane Handbook search methods.
    CochraneHandbook,
    /// Cochrane MECIR expectations.
    Mecir,
    /// JBI evidence-synthesis guidance.
    Jbi,
    /// Campbell Collaboration guidance.
    Campbell,
    /// A named extension or organisational policy pack.
    Custom(String),
}

/// One item in a versioned standard pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StandardItem {
    /// Stable item identifier.
    pub item_id: String,
    /// Human-readable label.
    pub label: String,
    /// Requirement text represented without asserting copyright ownership.
    pub requirement_summary: String,
    /// Whether the item concerns conduct, reporting or both.
    pub scope: String,
    /// Related item identifiers in other packs.
    #[serde(default)]
    pub crosswalks: Vec<String>,
}

/// Versioned standard pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StandardPack {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable pack identifier.
    pub pack_id: String,
    /// Standard family.
    pub family: StandardFamily,
    /// Version or publication year.
    pub version: String,
    /// Source citation or URL label.
    pub source: String,
    /// Licence/provenance note for the represented checklist data.
    pub provenance_note: String,
    /// Pack items.
    pub items: Vec<StandardItem>,
}

/// Assessment state for one standard item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StandardItemState {
    /// Item is met and evidence-linked.
    Met,
    /// Item is partly met.
    Partial,
    /// Item is required but unmet.
    Unmet,
    /// Item does not apply, with a rationale.
    NotApplicable,
    /// Item has not yet been assessed.
    NotAssessed,
}

/// Evidence for one standard item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StandardItemAssessment {
    /// Item identifier from the selected pack.
    pub item_id: String,
    /// Assessment state.
    pub state: StandardItemState,
    /// Audit events, files or receipt identifiers.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// Explanation or not-applicable rationale.
    pub note: String,
}

/// Assessment against one versioned standard pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StandardAssessment {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Pack identifier.
    pub pack_id: String,
    /// Pack version.
    pub pack_version: String,
    /// RFC 3339 assessment timestamp.
    pub assessed_at: String,
    /// Assessor identifier.
    pub assessed_by: String,
    /// Item assessments.
    pub items: Vec<StandardItemAssessment>,
}

impl Validate for StandardItem {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.item_id, "standards.item.item_id")?;
        require_text(&self.label, "standards.item.label")?;
        require_text(
            &self.requirement_summary,
            "standards.item.requirement_summary",
        )?;
        require_text(&self.scope, "standards.item.scope")
    }
}

impl Validate for StandardPack {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            STANDARD_PACK_SCHEMA_VERSION,
            "standards.schema_version",
        )?;
        require_text(&self.pack_id, "standards.pack_id")?;
        require_text(&self.version, "standards.version")?;
        require_text(&self.source, "standards.source")?;
        require_text(&self.provenance_note, "standards.provenance_note")?;
        if self.items.is_empty() {
            return Err(ContractError::EmptyCollection("standards.items"));
        }
        let mut identifiers = BTreeSet::new();
        for item in &self.items {
            item.validate()?;
            if !identifiers.insert(item.item_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "standard item identifier `{}` is duplicated",
                    item.item_id
                )));
            }
        }
        Ok(())
    }
}

impl Validate for StandardItemAssessment {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.item_id, "standard_assessment.item_id")?;
        require_text(&self.note, "standard_assessment.note")?;
        match self.state {
            StandardItemState::Met | StandardItemState::Partial => {
                if self.evidence.is_empty() {
                    return Err(ContractError::EmptyCollection(
                        "standard_assessment.evidence",
                    ));
                }
            }
            StandardItemState::Unmet
            | StandardItemState::NotApplicable
            | StandardItemState::NotAssessed => {}
        }
        Ok(())
    }
}

impl Validate for StandardAssessment {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            STANDARD_ASSESSMENT_SCHEMA_VERSION,
            "standard_assessment.schema_version",
        )?;
        require_text(&self.review_id, "standard_assessment.review_id")?;
        require_text(&self.pack_id, "standard_assessment.pack_id")?;
        require_text(&self.pack_version, "standard_assessment.pack_version")?;
        require_text(&self.assessed_at, "standard_assessment.assessed_at")?;
        require_text(&self.assessed_by, "standard_assessment.assessed_by")?;
        if self.items.is_empty() {
            return Err(ContractError::EmptyCollection("standard_assessment.items"));
        }
        let mut identifiers = BTreeSet::new();
        for item in &self.items {
            item.validate()?;
            if !identifiers.insert(item.item_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "standard assessment item `{}` is duplicated",
                    item.item_id
                )));
            }
        }
        Ok(())
    }
}
