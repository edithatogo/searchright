use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    ContractError, SEARCH_VALIDATION_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// PRESS 2015 review element.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PressElement {
    /// Translation of the research question.
    TranslationOfQuestion,
    /// Boolean and proximity operators.
    BooleanAndProximity,
    /// Subject headings.
    SubjectHeadings,
    /// Text words.
    TextWords,
    /// Spelling, syntax and line numbers.
    SpellingSyntaxAndLines,
    /// Limits and filters.
    LimitsAndFilters,
}

/// Severity of one search-strategy review finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Informational observation.
    Note,
    /// Improvement is recommended but not required for execution.
    Advisory,
    /// Material defect that should be corrected.
    Major,
    /// Defect that invalidates or dangerously narrows the strategy.
    Critical,
}

/// One PRESS finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PressFinding {
    /// Stable finding identifier.
    pub finding_id: String,
    /// PRESS element.
    pub element: PressElement,
    /// Severity.
    pub severity: FindingSeverity,
    /// Evidence-bearing finding.
    pub message: String,
    /// Proposed correction.
    pub recommendation: String,
    /// Whether the finding has been resolved.
    pub resolved: bool,
}

/// Independent peer review of a search strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PressReview {
    /// Stable review identifier.
    pub press_review_id: String,
    /// Search strategy identifier.
    pub strategy_id: String,
    /// Strategy version reviewed.
    pub strategy_version: String,
    /// Reviewer identifier.
    pub reviewer_id: String,
    /// RFC 3339 review timestamp.
    pub reviewed_at: String,
    /// Findings.
    #[serde(default)]
    pub findings: Vec<PressFinding>,
    /// Overall decision.
    pub decision: String,
}

/// Known relevant report used for recall-oriented validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeedRecord {
    /// Stable seed identifier.
    pub seed_id: String,
    /// DOI, PMID, title fingerprint or another explicit identifier.
    pub identifier: String,
    /// Why the seed is considered relevant.
    pub relevance_basis: String,
    /// Whether the executed strategy retrieved the seed.
    pub retrieved: bool,
    /// Provider/source where retrieval was checked.
    pub source_id: String,
}

/// Translation-loss budget and observed loss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TranslationLossAssessment {
    /// Source strategy identifier.
    pub strategy_id: String,
    /// Target dialect label.
    pub target_dialect: String,
    /// Maximum accepted count of material warnings.
    pub maximum_material_warnings: u32,
    /// Observed material-warning count.
    pub observed_material_warnings: u32,
    /// Whether manual review approved the translation.
    pub human_approved: bool,
    /// Explanatory notes.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// Combined validation report for one strategy version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SearchValidationReport {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Search strategy identifier.
    pub strategy_id: String,
    /// Strategy version.
    pub strategy_version: String,
    /// PRESS reviews.
    #[serde(default)]
    pub press_reviews: Vec<PressReview>,
    /// Known relevant seeds.
    #[serde(default)]
    pub seed_records: Vec<SeedRecord>,
    /// Dialect translation assessments.
    #[serde(default)]
    pub translation_assessments: Vec<TranslationLossAssessment>,
    /// Minimum accepted seed recall, zero to one.
    pub minimum_seed_recall: Option<f64>,
    /// Whether the strategy may proceed under the protocol.
    pub approved_for_execution: bool,
    /// Human approval identifier when execution is approved.
    pub approved_by: Option<String>,
}

impl Validate for PressFinding {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.finding_id, "search_validation.press.finding_id")?;
        require_text(&self.message, "search_validation.press.message")?;
        require_text(
            &self.recommendation,
            "search_validation.press.recommendation",
        )
    }
}

impl Validate for PressReview {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.press_review_id, "search_validation.press_review_id")?;
        require_text(&self.strategy_id, "search_validation.strategy_id")?;
        require_text(&self.strategy_version, "search_validation.strategy_version")?;
        require_text(&self.reviewer_id, "search_validation.reviewer_id")?;
        require_text(&self.reviewed_at, "search_validation.reviewed_at")?;
        OffsetDateTime::parse(&self.reviewed_at, &Rfc3339).map_err(|_| {
            ContractError::Invariant(
                "search_validation.reviewed_at must be an RFC 3339 timestamp".to_owned(),
            )
        })?;
        require_text(&self.decision, "search_validation.decision")?;
        for finding in &self.findings {
            finding.validate()?;
        }
        Ok(())
    }
}

impl Validate for SeedRecord {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.seed_id, "search_validation.seed.seed_id")?;
        require_text(&self.identifier, "search_validation.seed.identifier")?;
        require_text(
            &self.relevance_basis,
            "search_validation.seed.relevance_basis",
        )?;
        require_text(&self.source_id, "search_validation.seed.source_id")
    }
}

impl Validate for TranslationLossAssessment {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(
            &self.strategy_id,
            "search_validation.translation.strategy_id",
        )?;
        require_text(
            &self.target_dialect,
            "search_validation.translation.target_dialect",
        )?;
        Ok(())
    }
}

impl Validate for SearchValidationReport {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            SEARCH_VALIDATION_SCHEMA_VERSION,
            "search_validation.schema_version",
        )?;
        require_text(&self.review_id, "search_validation.review_id")?;
        require_text(&self.strategy_id, "search_validation.strategy_id")?;
        require_text(&self.strategy_version, "search_validation.strategy_version")?;
        if let Some(minimum) = self.minimum_seed_recall
            && !(0.0..=1.0).contains(&minimum)
        {
            return Err(ContractError::Invariant(
                "minimum seed recall must be between zero and one".to_owned(),
            ));
        }
        for review in &self.press_reviews {
            review.validate()?;
            if review.strategy_id != self.strategy_id {
                return Err(ContractError::Invariant(
                    "PRESS review strategy identifier does not match validation report".to_owned(),
                ));
            }
        }
        for seed in &self.seed_records {
            seed.validate()?;
        }
        for assessment in &self.translation_assessments {
            assessment.validate()?;
            if assessment.strategy_id != self.strategy_id {
                return Err(ContractError::Invariant(
                    "translation assessment strategy identifier does not match validation report"
                        .to_owned(),
                ));
            }
        }
        if self.approved_for_execution {
            let approver = self.approved_by.as_deref().ok_or_else(|| {
                ContractError::Invariant(
                    "approved validation report must identify a human approver".to_owned(),
                )
            })?;
            require_text(approver, "search_validation.approved_by")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod press_review_tests {
    use super::*;

    fn review(reviewed_at: &str) -> PressReview {
        PressReview {
            press_review_id: "press-1".to_owned(),
            strategy_id: "strategy-1".to_owned(),
            strategy_version: "1".to_owned(),
            reviewer_id: "reviewer-1".to_owned(),
            reviewed_at: reviewed_at.to_owned(),
            findings: Vec::new(),
            decision: "approved".to_owned(),
        }
    }

    #[test]
    fn press_review_requires_rfc3339_timestamp() {
        assert!(review("2026-08-29T01:02:03Z").validate().is_ok());
        assert!(review("2026-08-29T11:02:03+10:00").validate().is_ok());
        assert!(matches!(
            review("29 August 2026").validate(),
            Err(ContractError::Invariant(message)) if message.contains("RFC 3339")
        ));
    }
}
