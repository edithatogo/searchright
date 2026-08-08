//! Search-strategy validation and approval gates.

#![forbid(unsafe_code)]

use searchright_contracts::{
    FindingSeverity, SearchValidationReport, TranslationLossAssessment, Validate,
};
use serde::{Deserialize, Serialize};

/// Calculated seed-set recall and counts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeedRecall {
    /// Retrieved relevant seeds.
    pub retrieved: u64,
    /// Total relevant seeds.
    pub total: u64,
    /// Recall when the total is non-zero.
    pub recall: Option<f64>,
}

/// Compute recall over declared known-relevant seed records.
#[must_use]
pub fn seed_recall(report: &SearchValidationReport) -> SeedRecall {
    let total = u64::try_from(report.seed_records.len()).unwrap_or(u64::MAX);
    let retrieved = u64::try_from(
        report
            .seed_records
            .iter()
            .filter(|record| record.retrieved)
            .count(),
    )
    .unwrap_or(u64::MAX);
    let recall = if total == 0 {
        None
    } else {
        Some(ratio(retrieved, total))
    };
    SeedRecall {
        retrieved,
        total,
        recall,
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "seed-set recall intentionally projects bounded integer counts into f64"
)]
fn ratio(numerator: u64, denominator: u64) -> f64 {
    numerator as f64 / denominator as f64
}

/// Whether a translation assessment stays within its declared loss budget.
#[must_use]
pub fn translation_acceptable(assessment: &TranslationLossAssessment) -> bool {
    assessment.observed_material_warnings <= assessment.maximum_material_warnings
        && assessment.human_approved
}

/// Validate the complete search-validation report and all approval conditions.
pub fn assess(report: &SearchValidationReport) -> Result<SearchValidationSummary, ValidationError> {
    report.validate()?;
    let seeds = seed_recall(report);
    let unresolved_major = report
        .press_reviews
        .iter()
        .flat_map(|review| &review.findings)
        .filter(|finding| {
            !finding.resolved
                && matches!(
                    finding.severity,
                    FindingSeverity::Major | FindingSeverity::Critical
                )
        })
        .count();
    let unacceptable_translations = report
        .translation_assessments
        .iter()
        .filter(|assessment| !translation_acceptable(assessment))
        .count();
    let seed_recall_passes = report.minimum_seed_recall.is_none_or(|minimum| {
        seeds.recall.is_some_and(|observed| observed >= minimum)
    });
    let ready = unresolved_major == 0
        && unacceptable_translations == 0
        && seed_recall_passes
        && report.approved_for_execution;
    Ok(SearchValidationSummary {
        ready,
        seed_recall: seeds,
        unresolved_major_findings: unresolved_major,
        unacceptable_translations,
    })
}

/// Summary of executable search-validation gates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SearchValidationSummary {
    /// Whether all configured gates pass.
    pub ready: bool,
    /// Seed-set recall.
    pub seed_recall: SeedRecall,
    /// Unresolved major or critical PRESS findings.
    pub unresolved_major_findings: usize,
    /// Dialect translations outside their approved loss budgets.
    pub unacceptable_translations: usize,
}

/// Search-validation error.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
}
