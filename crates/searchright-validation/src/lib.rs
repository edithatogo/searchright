//! Search-strategy validation and approval gates.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use schemars::JsonSchema;
use searchright_contracts::{
    FindingSeverity, PressElement, SearchValidationReport, TranslationLossAssessment, Validate,
};
use serde::{Deserialize, Serialize};

/// Calculated seed-set recall and counts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub const fn translation_acceptable(assessment: &TranslationLossAssessment) -> bool {
    assessment.observed_material_warnings <= assessment.maximum_material_warnings
        && assessment.human_approved
}

/// Count distinct PRESS domains represented by evidence-bearing findings.
#[must_use]
pub fn press_domains_covered(report: &SearchValidationReport) -> usize {
    report
        .press_reviews
        .iter()
        .flat_map(|review| &review.findings)
        .map(|finding| finding.element)
        .collect::<BTreeSet<PressElement>>()
        .len()
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
    let press_domains_covered = press_domains_covered(report);
    let press_review_complete = press_domains_covered == 6;
    let seed_recall_passes = report
        .minimum_seed_recall
        .is_none_or(|minimum| seeds.recall.is_some_and(|observed| observed >= minimum));
    let ready = press_review_complete
        && unresolved_major == 0
        && unacceptable_translations == 0
        && seed_recall_passes
        && report.approved_for_execution;
    Ok(SearchValidationSummary {
        ready,
        seed_recall: seeds,
        unresolved_major_findings: unresolved_major,
        unacceptable_translations,
        press_domains_covered,
        press_review_complete,
    })
}

/// Summary of executable search-validation gates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SearchValidationSummary {
    /// Whether all configured gates pass.
    pub ready: bool,
    /// Seed-set recall.
    pub seed_recall: SeedRecall,
    /// Unresolved major or critical PRESS findings.
    pub unresolved_major_findings: usize,
    /// Dialect translations outside their approved loss budgets.
    pub unacceptable_translations: usize,
    /// Number of distinct PRESS domains represented by findings.
    pub press_domains_covered: usize,
    /// Whether all six PRESS domains have evidence-bearing findings.
    pub press_review_complete: bool,
}

/// Search-validation error.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn approved_but_incomplete_press_record_fails_closed() -> Result<(), Box<dyn Error>> {
        let report: SearchValidationReport = serde_yaml::from_str(include_str!(
            "../../../contracts/examples/search-validation.yaml"
        ))?;
        let summary = assess(&report)?;
        assert_eq!(summary.press_domains_covered, 1);
        assert!(!summary.press_review_complete);
        assert!(!summary.ready);
        Ok(())
    }

    #[test]
    fn six_domain_press_workflow_requires_separate_approval() -> Result<(), Box<dyn Error>> {
        let report: SearchValidationReport = serde_yaml::from_str(include_str!(
            "../../../verification/fixtures/track-08/press-workflow-complete.yaml"
        ))?;
        let summary = assess(&report)?;
        assert_eq!(summary.press_domains_covered, 6);
        assert!(summary.press_review_complete);
        assert!(summary.ready);

        let mut same_person = report;
        same_person.approved_by = Some("fixture-information-specialist".to_owned());
        assert!(assess(&same_person).is_err());
        Ok(())
    }
}
