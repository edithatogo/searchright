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
}

impl Validate for RankingCalibration {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            RANKING_CALIBRATION_SCHEMA_VERSION,
            "ranking_calibration.schema_version",
        )?;
        require_text(&self.review_id, "ranking_calibration.review_id")?;
        require_text(
            &self.ranker_version,
            "ranking_calibration.ranker_version",
        )?;
        if !(0.0..=1.0).contains(&self.threshold)
            || !(0.0..=1.0).contains(&self.minimum_sensitivity)
        {
            return Err(ContractError::Invariant(
                "ranking threshold and minimum sensitivity must be between zero and one"
                    .to_owned(),
            ));
        }
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
