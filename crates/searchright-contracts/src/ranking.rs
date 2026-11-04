use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, RANKING_CALIBRATION_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// One transparent feature contribution to a ranking score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RankingFeature {
    /// Stable feature name.
    pub name: String,
    /// Raw feature value.
    pub value: f64,
    /// Applied weight.
    pub weight: f64,
    /// Human-readable rationale.
    pub rationale: String,
}

/// Advisory ranking score for one record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RankingScore {
    /// Record identifier.
    pub record_id: String,
    /// Score between zero and one.
    pub score: f64,
    /// Transparent feature contributions.
    pub features: Vec<RankingFeature>,
    /// Ranker version.
    pub ranker_version: String,
    /// Whether a human has reviewed the recommendation.
    pub human_reviewed: bool,
}

/// Confusion-matrix counts for a calibrated threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CalibrationCounts {
    /// Relevant records ranked above the threshold.
    pub true_positive: u64,
    /// Irrelevant records ranked above the threshold.
    pub false_positive: u64,
    /// Irrelevant records ranked below the threshold.
    pub true_negative: u64,
    /// Relevant records ranked below the threshold.
    pub false_negative: u64,
}

/// Calibration evidence for advisory prioritisation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RankingCalibration {
    /// Contract identifier.
    pub schema_version: String,
    /// Review or benchmark identifier.
    pub review_id: String,
    /// Ranker version.
    pub ranker_version: String,
    /// Evaluated threshold.
    pub threshold: f64,
    /// Confusion-matrix counts.
    pub counts: CalibrationCounts,
    /// Minimum permitted sensitivity.
    pub minimum_sensitivity: f64,
    /// Whether the ranker may be used for prioritisation.
    pub approved_for_prioritisation: bool,
    /// Explicit statement that auto-exclusion remains prohibited.
    pub auto_exclusion_prohibited: bool,
    /// Human approver.
    pub approved_by: Option<String>,
}

impl Validate for RankingFeature {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.name, "ranking.feature.name")?;
        require_text(&self.rationale, "ranking.feature.rationale")?;
        if !self.value.is_finite() || !self.weight.is_finite() {
            return Err(ContractError::Invariant(
                "ranking feature values and weights must be finite".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for RankingScore {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.record_id, "ranking.record_id")?;
        require_text(&self.ranker_version, "ranking.ranker_version")?;
        if !(0.0..=1.0).contains(&self.score) {
            return Err(ContractError::Invariant(
                "ranking score must be between zero and one".to_owned(),
            ));
        }
        if self.features.is_empty() {
            return Err(ContractError::EmptyCollection("ranking.features"));
        }
        for feature in &self.features {
            feature.validate()?;
        }
        Ok(())
    }
}

impl CalibrationCounts {
    /// Sensitivity, when at least one relevant item is present.
    #[must_use]
    pub fn sensitivity(self) -> Option<f64> {
        let denominator = self.true_positive.saturating_add(self.false_negative);
        ratio(self.true_positive, denominator)
    }

    /// Specificity, when at least one irrelevant item is present.
    #[must_use]
    pub fn specificity(self) -> Option<f64> {
        let denominator = self.true_negative.saturating_add(self.false_positive);
        ratio(self.true_negative, denominator)
    }

    /// Precision (positive predictive value), when at least one item is ranked positive.
    #[must_use]
    pub fn precision(self) -> Option<f64> {
        let denominator = self.true_positive.saturating_add(self.false_positive);
        ratio(self.true_positive, denominator)
    }

    /// False negative rate (miss rate), when at least one relevant item is present.
    #[must_use]
    pub fn false_negative_rate(self) -> Option<f64> {
        let denominator = self.true_positive.saturating_add(self.false_negative);
        ratio(self.false_negative, denominator)
    }

    /// False positive rate (fall-out), when at least one irrelevant item is present.
    #[must_use]
    pub fn false_positive_rate(self) -> Option<f64> {
        let denominator = self.true_negative.saturating_add(self.false_positive);
        ratio(self.false_positive, denominator)
    }

    /// Total count of all evaluated records.
    #[must_use]
    pub const fn total(self) -> u64 {
        self.true_positive
            .saturating_add(self.false_positive)
            .saturating_add(self.true_negative)
            .saturating_add(self.false_negative)
    }
}

impl Validate for CalibrationCounts {
    fn validate(&self) -> Result<(), ContractError> {
        let total = self
            .true_positive
            .checked_add(self.false_positive)
            .and_then(|sum| sum.checked_add(self.true_negative))
            .and_then(|sum| sum.checked_add(self.false_negative));
        if total.is_none() {
            return Err(ContractError::Invariant(
                "calibration count sum overflowed u64".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for RankingCalibration {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            RANKING_CALIBRATION_SCHEMA_VERSION,
            "ranking_calibration.schema_version",
        )?;
        require_text(&self.review_id, "ranking_calibration.review_id")?;
        require_text(&self.ranker_version, "ranking_calibration.ranker_version")?;
        if !(0.0..=1.0).contains(&self.threshold)
            || !(0.0..=1.0).contains(&self.minimum_sensitivity)
        {
            return Err(ContractError::Invariant(
                "ranking threshold and minimum sensitivity must be between zero and one".to_owned(),
            ));
        }
        self.counts.validate()?;
        if !self.auto_exclusion_prohibited {
            return Err(ContractError::Invariant(
                "Searchright ranking calibration must prohibit automatic exclusion".to_owned(),
            ));
        }
        if self.approved_for_prioritisation {
            let sensitivity = self.counts.sensitivity().ok_or_else(|| {
                ContractError::Invariant(
                    "ranking calibration requires relevant examples".to_owned(),
                )
            })?;
            if sensitivity < self.minimum_sensitivity {
                return Err(ContractError::Invariant(format!(
                    "observed sensitivity {sensitivity:.4} is below the minimum {}",
                    self.minimum_sensitivity
                )));
            }
            let approver = self.approved_by.as_deref().ok_or_else(|| {
                ContractError::Invariant(
                    "approved ranking calibration must identify a human approver".to_owned(),
                )
            })?;
            require_text(approver, "ranking_calibration.approved_by")?;
        }
        Ok(())
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "calibration proportions intentionally project bounded integer counts into f64"
)]
fn ratio(numerator: u64, denominator: u64) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_feature() -> RankingFeature {
        RankingFeature {
            name: "title_token_overlap".to_owned(),
            value: 0.75,
            weight: 0.55,
            rationale: "fraction of query tokens in title".to_owned(),
        }
    }

    fn valid_score() -> RankingScore {
        RankingScore {
            record_id: "rec-001".to_owned(),
            score: 0.82,
            features: vec![valid_feature()],
            ranker_version: "lexical-v1".to_owned(),
            human_reviewed: false,
        }
    }

    fn valid_calibration() -> RankingCalibration {
        RankingCalibration {
            schema_version: RANKING_CALIBRATION_SCHEMA_VERSION.to_owned(),
            review_id: "review-test-1".to_owned(),
            ranker_version: "lexical-v1".to_owned(),
            threshold: 0.5,
            counts: CalibrationCounts {
                true_positive: 95,
                false_positive: 10,
                true_negative: 890,
                false_negative: 5,
            },
            minimum_sensitivity: 0.90,
            approved_for_prioritisation: true,
            auto_exclusion_prohibited: true,
            approved_by: Some("lead-reviewer".to_owned()),
        }
    }

    #[test]
    fn ranking_feature_validates_required_and_bounded_fields() {
        let feature = valid_feature();
        let Ok(()) = feature.validate() else {
            panic!("valid feature must pass validation");
        };

        let mut empty_name = valid_feature();
        empty_name.name = String::new();
        let Err(_) = empty_name.validate() else {
            panic!("empty name must fail validation");
        };

        let mut empty_rationale = valid_feature();
        empty_rationale.rationale = "   ".to_owned();
        let Err(_) = empty_rationale.validate() else {
            panic!("blank rationale must fail validation");
        };

        let mut nan_val = valid_feature();
        nan_val.value = f64::NAN;
        let Err(_) = nan_val.validate() else {
            panic!("NaN value must fail validation");
        };

        let mut inf_weight = valid_feature();
        inf_weight.weight = f64::INFINITY;
        let Err(_) = inf_weight.validate() else {
            panic!("infinite weight must fail validation");
        };
    }

    #[test]
    fn ranking_score_validates_score_bounds_and_features() {
        let score = valid_score();
        let Ok(()) = score.validate() else {
            panic!("valid score must pass validation");
        };

        let mut empty_rec = valid_score();
        empty_rec.record_id = String::new();
        let Err(_) = empty_rec.validate() else {
            panic!("empty record_id must fail validation");
        };

        let mut empty_ver = valid_score();
        empty_ver.ranker_version = String::new();
        let Err(_) = empty_ver.validate() else {
            panic!("empty ranker_version must fail validation");
        };

        let mut low_score = valid_score();
        low_score.score = -0.01;
        let Err(_) = low_score.validate() else {
            panic!("negative score must fail validation");
        };

        let mut high_score = valid_score();
        high_score.score = 1.01;
        let Err(_) = high_score.validate() else {
            panic!("score > 1.0 must fail validation");
        };

        let mut no_features = valid_score();
        no_features.features.clear();
        let Err(_) = no_features.validate() else {
            panic!("empty features must fail validation");
        };

        let mut bad_feature = valid_score();
        bad_feature.features = vec![RankingFeature {
            name: String::new(),
            value: 0.5,
            weight: 0.5,
            rationale: "test".to_owned(),
        }];
        let Err(_) = bad_feature.validate() else {
            panic!("score with invalid feature must fail validation");
        };
    }

    #[test]
    fn calibration_counts_metrics_and_ratios() {
        let counts = CalibrationCounts {
            true_positive: 90,
            false_positive: 10,
            true_negative: 80,
            false_negative: 20,
        };
        let Ok(()) = counts.validate() else {
            panic!("valid calibration counts must pass validation");
        };

        assert_eq!(counts.total(), 200);

        let Some(sensitivity) = counts.sensitivity() else {
            panic!("sensitivity must be calculated");
        };
        assert!((sensitivity - (90.0 / 110.0)).abs() < 1e-9);

        let Some(specificity) = counts.specificity() else {
            panic!("specificity must be calculated");
        };
        assert!((specificity - (80.0 / 90.0)).abs() < 1e-9);

        let Some(precision) = counts.precision() else {
            panic!("precision must be calculated");
        };
        assert!((precision - (90.0 / 100.0)).abs() < 1e-9);

        let Some(fnr) = counts.false_negative_rate() else {
            panic!("false negative rate must be calculated");
        };
        assert!((fnr - (20.0 / 110.0)).abs() < 1e-9);

        let Some(fpr) = counts.false_positive_rate() else {
            panic!("false positive rate must be calculated");
        };
        assert!((fpr - (10.0 / 90.0)).abs() < 1e-9);

        // Zero denominators return None
        let empty_counts = CalibrationCounts {
            true_positive: 0,
            false_positive: 0,
            true_negative: 0,
            false_negative: 0,
        };
        assert_eq!(empty_counts.sensitivity(), None);
        assert_eq!(empty_counts.specificity(), None);
        assert_eq!(empty_counts.precision(), None);
        assert_eq!(empty_counts.false_negative_rate(), None);
        assert_eq!(empty_counts.false_positive_rate(), None);
        assert_eq!(empty_counts.total(), 0);
    }

    #[test]
    fn ranking_calibration_validates_schema_and_advisory_rules() {
        let calibration = valid_calibration();
        let Ok(()) = calibration.validate() else {
            panic!("valid approved calibration must pass validation");
        };

        // Unapproved calibration without approver is valid
        let mut unapproved = valid_calibration();
        unapproved.approved_for_prioritisation = false;
        unapproved.approved_by = None;
        let Ok(()) = unapproved.validate() else {
            panic!("unapproved calibration without approver must pass validation");
        };

        // Invalid schema version fails
        let mut bad_schema = valid_calibration();
        bad_schema.schema_version = "org.searchright.ranking-calibration.v2".to_owned();
        let Err(_) = bad_schema.validate() else {
            panic!("invalid schema version must fail validation");
        };

        // Empty review_id fails
        let mut empty_review = valid_calibration();
        empty_review.review_id = String::new();
        let Err(_) = empty_review.validate() else {
            panic!("empty review_id must fail validation");
        };

        // Empty ranker_version fails
        let mut empty_ranker = valid_calibration();
        empty_ranker.ranker_version = String::new();
        let Err(_) = empty_ranker.validate() else {
            panic!("empty ranker_version must fail validation");
        };

        // Threshold out of bounds fails
        let mut bad_threshold = valid_calibration();
        bad_threshold.threshold = 1.05;
        let Err(_) = bad_threshold.validate() else {
            panic!("threshold > 1.0 must fail validation");
        };

        // Minimum sensitivity out of bounds fails
        let mut bad_min_sens = valid_calibration();
        bad_min_sens.minimum_sensitivity = -0.1;
        let Err(_) = bad_min_sens.validate() else {
            panic!("negative minimum sensitivity must fail validation");
        };

        // Strict advisory claim boundary: auto_exclusion_prohibited MUST be true
        let mut auto_exclude = valid_calibration();
        auto_exclude.auto_exclusion_prohibited = false;
        let Err(_) = auto_exclude.validate() else {
            panic!("auto_exclusion_prohibited: false must strictly fail validation");
        };

        // Approved without approver fails
        let mut missing_approver = valid_calibration();
        missing_approver.approved_by = None;
        let Err(_) = missing_approver.validate() else {
            panic!("approved calibration without approver must fail validation");
        };

        // Approved with blank approver fails
        let mut blank_approver = valid_calibration();
        blank_approver.approved_by = Some("   ".to_owned());
        let Err(_) = blank_approver.validate() else {
            panic!("approved calibration with blank approver must fail validation");
        };

        // Approved with zero relevant items fails
        let mut no_relevant = valid_calibration();
        no_relevant.counts = CalibrationCounts {
            true_positive: 0,
            false_positive: 10,
            true_negative: 90,
            false_negative: 0,
        };
        let Err(_) = no_relevant.validate() else {
            panic!("approved calibration with zero relevant examples must fail validation");
        };

        // Approved with sensitivity below minimum fails
        let mut low_sens = valid_calibration();
        low_sens.counts = CalibrationCounts {
            true_positive: 80,
            false_positive: 10,
            true_negative: 890,
            false_negative: 20, // sensitivity = 80/100 = 0.80 < 0.90
        };
        let Err(_) = low_sens.validate() else {
            panic!("approved calibration below minimum sensitivity must fail validation");
        };
    }
}
