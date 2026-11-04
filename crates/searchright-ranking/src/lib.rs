//! Transparent advisory ranking with explicit calibration and no automatic exclusion.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use searchright_contracts::{
    BibliographicRecord, CalibrationCounts, RankingCalibration, RankingFeature, RankingScore,
    Validate,
};

/// Deterministic lexical ranker configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalRanker {
    /// Version label included in every score.
    pub version: String,
    /// Weight for title overlap.
    pub title_weight: f64,
    /// Weight for abstract overlap.
    pub abstract_weight: f64,
    /// Weight for subject overlap.
    pub subject_weight: f64,
}

impl Default for LexicalRanker {
    fn default() -> Self {
        Self {
            version: "lexical-v1".to_owned(),
            title_weight: 0.55,
            abstract_weight: 0.30,
            subject_weight: 0.15,
        }
    }
}

impl LexicalRanker {
    /// Score records against transparent query terms.
    pub fn score(
        &self,
        records: &[BibliographicRecord],
        query_terms: &[String],
    ) -> Result<Vec<RankingScore>, RankingError> {
        let query = token_set(&query_terms.join(" "));
        if query.is_empty() {
            return Err(RankingError::EmptyQuery);
        }
        let mut scores = Vec::with_capacity(records.len());
        for record in records {
            let title = overlap(&query, &token_set(&record.title));
            let abstract_value = record
                .abstract_text
                .as_deref()
                .map_or(0.0, |text| overlap(&query, &token_set(text)));
            let subject_value = overlap(&query, &token_set(&record.subjects.join(" ")));
            let weighted = title.mul_add(
                self.title_weight,
                abstract_value.mul_add(self.abstract_weight, subject_value * self.subject_weight),
            );
            let total_weight = self.title_weight + self.abstract_weight + self.subject_weight;
            if total_weight <= 0.0 || !total_weight.is_finite() {
                return Err(RankingError::InvalidWeights);
            }
            let score = (weighted / total_weight).clamp(0.0, 1.0);
            let result = RankingScore {
                record_id: record.record_id.clone(),
                score,
                features: vec![
                    RankingFeature {
                        name: "title_token_overlap".to_owned(),
                        value: title,
                        weight: self.title_weight,
                        rationale: "fraction of query tokens found in the title".to_owned(),
                    },
                    RankingFeature {
                        name: "abstract_token_overlap".to_owned(),
                        value: abstract_value,
                        weight: self.abstract_weight,
                        rationale: "fraction of query tokens found in the abstract".to_owned(),
                    },
                    RankingFeature {
                        name: "subject_token_overlap".to_owned(),
                        value: subject_value,
                        weight: self.subject_weight,
                        rationale: "fraction of query tokens found in subject terms".to_owned(),
                    },
                ],
                ranker_version: self.version.clone(),
                human_reviewed: false,
            };
            result.validate()?;
            scores.push(result);
        }
        scores.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.record_id.cmp(&right.record_id))
        });
        Ok(scores)
    }
}

/// Build confusion counts from labelled scores at one threshold.
#[must_use]
pub fn calibration_counts(
    scores: &[RankingScore],
    relevant_record_ids: &BTreeSet<String>,
    threshold: f64,
) -> CalibrationCounts {
    let mut counts = CalibrationCounts {
        true_positive: 0,
        false_positive: 0,
        true_negative: 0,
        false_negative: 0,
    };
    for score in scores {
        let predicted_relevant = score.score >= threshold;
        let relevant = relevant_record_ids.contains(&score.record_id);
        match (predicted_relevant, relevant) {
            (true, true) => counts.true_positive += 1,
            (true, false) => counts.false_positive += 1,
            (false, false) => counts.true_negative += 1,
            (false, true) => counts.false_negative += 1,
        }
    }
    counts
}

/// A single evaluation point on an empirical calibration curve.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationPoint {
    /// Decision threshold evaluated.
    pub threshold: f64,
    /// Confusion counts at this threshold.
    pub counts: CalibrationCounts,
    /// Empirical sensitivity / recall (TP / (TP + FN)), 0.0 if no relevant items.
    pub recall: f64,
    /// Empirical precision (TP / (TP + FP)), 0.0 if no predicted positives.
    pub precision: f64,
    /// Empirical false negative rate (FN / (TP + FN)), 0.0 if no relevant items.
    pub false_negative_rate: f64,
    /// Empirical false positive rate (FP / (TN + FP)), 0.0 if no irrelevant items.
    pub false_positive_rate: f64,
    /// Work Saved over Sampling at this threshold: `recall - (TP + FP) / total`.
    pub work_saved_over_sampling: f64,
}

/// Empirical calibration curve evaluated across multiple thresholds.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationCurve {
    /// Total records evaluated.
    pub total_records: usize,
    /// Total gold relevant records.
    pub total_relevant: usize,
    /// Threshold points evaluated.
    pub points: Vec<CalibrationPoint>,
}

impl CalibrationCurve {
    /// Build an advisory `RankingCalibration` contract at a selected threshold.
    ///
    /// Always enforces `auto_exclusion_prohibited: true` to prevent autonomous screening exclusion.
    pub fn to_ranking_calibration(
        &self,
        review_id: &str,
        ranker_version: &str,
        threshold: f64,
        minimum_sensitivity: f64,
        approved_by: Option<&str>,
    ) -> Result<RankingCalibration, RankingError> {
        let point = self
            .points
            .iter()
            .find(|p| (p.threshold - threshold).abs() < 1e-6)
            .ok_or_else(|| {
                searchright_contracts::ContractError::Invariant(format!(
                    "threshold {threshold} not found in evaluated calibration curve"
                ))
            })?;

        let calibration = RankingCalibration {
            schema_version: searchright_contracts::RANKING_CALIBRATION_SCHEMA_VERSION.to_owned(),
            review_id: review_id.to_owned(),
            ranker_version: ranker_version.to_owned(),
            threshold,
            counts: point.counts,
            minimum_sensitivity,
            approved_for_prioritisation: approved_by.is_some(),
            auto_exclusion_prohibited: true,
            approved_by: approved_by.map(str::to_owned),
        };
        calibration.validate()?;
        Ok(calibration)
    }
}

/// Calibrate ranking scores against gold human decisions across evaluated thresholds.
///
/// Computes confusion counts, recall, precision, FNR, FPR, and WSS for each threshold.
/// Protects against zero division when counts are zero.
#[must_use]
pub fn calibrate(
    scores: &[RankingScore],
    relevant_record_ids: &BTreeSet<String>,
    thresholds: &[f64],
) -> CalibrationCurve {
    let total_records = scores.len();
    let total_relevant = scores
        .iter()
        .filter(|score| relevant_record_ids.contains(&score.record_id))
        .count();

    let default_thresholds = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
    let eval_thresholds: &[f64] = if thresholds.is_empty() {
        &default_thresholds
    } else {
        thresholds
    };

    let total_f = usize_to_f64(total_records);
    let mut points = Vec::with_capacity(eval_thresholds.len());

    for &threshold in eval_thresholds {
        let counts = calibration_counts(scores, relevant_record_ids, threshold);
        let true_pos = u64_to_f64(counts.true_positive);
        let false_pos = u64_to_f64(counts.false_positive);
        let true_neg = u64_to_f64(counts.true_negative);
        let false_neg = u64_to_f64(counts.false_negative);

        let relevant_sum = true_pos + false_neg;
        let recall = if relevant_sum > 0.0 {
            true_pos / relevant_sum
        } else {
            0.0
        };

        let predicted_positive_sum = true_pos + false_pos;
        let precision = if predicted_positive_sum > 0.0 {
            true_pos / predicted_positive_sum
        } else {
            0.0
        };

        let false_negative_rate = if relevant_sum > 0.0 {
            false_neg / relevant_sum
        } else {
            0.0
        };

        let irrelevant_sum = true_neg + false_pos;
        let false_positive_rate = if irrelevant_sum > 0.0 {
            false_pos / irrelevant_sum
        } else {
            0.0
        };

        let work_saved_over_sampling = if total_records > 0 {
            let sampling_fraction = predicted_positive_sum / total_f;
            recall - sampling_fraction
        } else {
            0.0
        };

        points.push(CalibrationPoint {
            threshold,
            counts,
            recall,
            precision,
            false_negative_rate,
            false_positive_rate,
            work_saved_over_sampling,
        });
    }

    CalibrationCurve {
        total_records,
        total_relevant,
        points,
    }
}

/// Validate that a calibration report permits prioritisation only.
pub fn validate_calibration(calibration: &RankingCalibration) -> Result<(), RankingError> {
    calibration.validate()?;
    Ok(())
}

fn overlap(query: &BTreeSet<String>, text: &BTreeSet<String>) -> f64 {
    let matches = query.intersection(text).count();
    if query.is_empty() {
        0.0
    } else {
        usize_to_f64(matches) / usize_to_f64(query.len())
    }
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "calibration proportions intentionally project bounded integer counts into f64"
)]
const fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

fn token_set(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(str::to_lowercase)
        .collect()
}

/// Ranking error.
#[derive(Debug, thiserror::Error)]
pub enum RankingError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Query terms were empty after tokenisation.
    #[error("ranking query contains no usable terms")]
    EmptyQuery,
    /// Configured weights were invalid.
    #[error("ranking weights must be finite and sum to a positive value")]
    InvalidWeights,
}

#[cfg(test)]
mod tests {
    use super::*;
    use searchright_contracts::BibliographicRecord;
    use serde_json::Value;

    fn sample_records() -> Vec<BibliographicRecord> {
        vec![
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: "rec-1".to_owned(),
                source_receipt_id: "receipt-test".to_owned(),
                native_id: "native-rec-1".to_owned(),
                kind: searchright_contracts::RecordKind::JournalArticle,
                identifiers: searchright_contracts::RecordIdentifiers::default(),
                title: "Machine learning for screening".to_owned(),
                abstract_text: Some("We explore active learning in reviews.".to_owned()),
                authors: vec!["Author One".to_owned()],
                container_title: None,
                publication_year: Some(2023),
                publication_date: None,
                languages: Vec::new(),
                subjects: vec!["information retrieval".to_owned()],
                urls: Vec::new(),
                provider_metadata: Value::Null,
            },
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: "rec-2".to_owned(),
                source_receipt_id: "receipt-test".to_owned(),
                native_id: "native-rec-2".to_owned(),
                kind: searchright_contracts::RecordKind::JournalArticle,
                identifiers: searchright_contracts::RecordIdentifiers::default(),
                title: "Deep neural networks".to_owned(),
                abstract_text: Some(
                    "Machine learning and screening techniques applied to clinical datasets."
                        .to_owned(),
                ),
                authors: vec!["Author Two".to_owned()],
                container_title: None,
                publication_year: Some(2024),
                publication_date: None,
                languages: Vec::new(),
                subjects: vec!["machine learning".to_owned()],
                urls: Vec::new(),
                provider_metadata: Value::Null,
            },
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: "rec-3".to_owned(),
                source_receipt_id: "receipt-test".to_owned(),
                native_id: "native-rec-3".to_owned(),
                kind: searchright_contracts::RecordKind::JournalArticle,
                identifiers: searchright_contracts::RecordIdentifiers::default(),
                title: "Randomized controlled trial of statins".to_owned(),
                abstract_text: Some("Cardiovascular outcomes in elderly patients.".to_owned()),
                authors: vec!["Author Three".to_owned()],
                container_title: None,
                publication_year: Some(2022),
                publication_date: None,
                languages: Vec::new(),
                subjects: vec!["cardiology".to_owned()],
                urls: Vec::new(),
                provider_metadata: Value::Null,
            },
        ]
    }

    #[test]
    fn lexical_ranker_title_and_abstract_weighting() {
        let records = sample_records();
        let query = vec!["machine".to_owned(), "learning".to_owned()];

        // Title-heavy ranker: rec-1 has both query terms in title, rec-2 has them only in abstract
        let title_heavy = LexicalRanker {
            version: "title-heavy-v1".to_owned(),
            title_weight: 0.80,
            abstract_weight: 0.15,
            subject_weight: 0.05,
        };
        let Ok(scores_title) = title_heavy.score(&records, &query) else {
            panic!("scoring must succeed");
        };
        assert_eq!(scores_title.len(), 3);
        let Some(first_title) = scores_title.first() else {
            panic!("first score must exist");
        };
        let Some(second_title) = scores_title.get(1) else {
            panic!("second score must exist");
        };
        assert_eq!(first_title.record_id, "rec-1");
        assert!(first_title.score > second_title.score);

        // Abstract-heavy ranker: abstract match outweighs title match
        let abstract_heavy = LexicalRanker {
            version: "abstract-heavy-v1".to_owned(),
            title_weight: 0.10,
            abstract_weight: 0.85,
            subject_weight: 0.05,
        };
        let Ok(scores_abstract) = abstract_heavy.score(&records, &query) else {
            panic!("scoring must succeed");
        };
        assert_eq!(scores_abstract.len(), 3);
        let Some(first_abstract) = scores_abstract.first() else {
            panic!("first score must exist");
        };
        let Some(second_abstract) = scores_abstract.get(1) else {
            panic!("second score must exist");
        };
        assert_eq!(first_abstract.record_id, "rec-2");
        assert!(first_abstract.score > second_abstract.score);
    }

    #[test]
    fn lexical_ranker_deterministic_scoring_and_bounds() {
        let records = sample_records();
        let query = vec!["screening".to_owned(), "review".to_owned()];
        let ranker = LexicalRanker::default();

        let Ok(first_run) = ranker.score(&records, &query) else {
            panic!("first score run must succeed");
        };
        let Ok(second_run) = ranker.score(&records, &query) else {
            panic!("second score run must succeed");
        };

        // Deterministic equality across multiple evaluations
        assert_eq!(first_run, second_run);

        // All scores are non-negative and bounded in 0.0..=1.0
        for score in &first_run {
            assert!(score.score >= 0.0);
            assert!(score.score <= 1.0);
            assert_eq!(score.ranker_version, "lexical-v1");
            assert!(!score.human_reviewed);
            assert_ne!(score.features.len(), 0);
            for feature in &score.features {
                assert!(feature.value >= 0.0);
                assert!(feature.weight > 0.0);
                assert_ne!(feature.name, String::new());
                assert_ne!(feature.rationale, String::new());
            }
        }
    }

    #[test]
    fn lexical_ranker_error_cases() {
        let records = sample_records();
        let ranker = LexicalRanker::default();

        // Empty query terms
        let empty_query: Vec<String> = Vec::new();
        let Err(RankingError::EmptyQuery) = ranker.score(&records, &empty_query) else {
            panic!("empty query terms must yield RankingError::EmptyQuery");
        };

        // Query with only punctuation / single chars (trimmed out)
        let punctuation_query = vec![".", "!", "a"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let Err(RankingError::EmptyQuery) = ranker.score(&records, &punctuation_query) else {
            panic!("query with no tokenizable terms must yield RankingError::EmptyQuery");
        };

        // Invalid weights: zero sum
        let zero_weights = LexicalRanker {
            version: "zero-weights".to_owned(),
            title_weight: 0.0,
            abstract_weight: 0.0,
            subject_weight: 0.0,
        };
        let query = vec!["machine".to_owned()];
        let Err(RankingError::InvalidWeights) = zero_weights.score(&records, &query) else {
            panic!("zero total weight must yield RankingError::InvalidWeights");
        };

        // Invalid weights: NaN
        let nan_weights = LexicalRanker {
            version: "nan-weights".to_owned(),
            title_weight: f64::NAN,
            abstract_weight: 0.3,
            subject_weight: 0.1,
        };
        let Err(RankingError::InvalidWeights) = nan_weights.score(&records, &query) else {
            panic!("NaN weights must yield RankingError::InvalidWeights");
        };
    }

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "analytical test cases yield exact binary floating point values"
    )]
    fn calibrate_computes_curve_metrics_and_wss() {
        let records = sample_records();
        let query = vec!["machine".to_owned(), "learning".to_owned()];
        let ranker = LexicalRanker::default();

        let Ok(scores) = ranker.score(&records, &query) else {
            panic!("scoring must succeed");
        };

        let mut relevant = BTreeSet::new();
        relevant.insert("rec-1".to_owned());
        relevant.insert("rec-2".to_owned());

        let thresholds = vec![0.1, 0.4, 0.9];
        let curve = calibrate(&scores, &relevant, &thresholds);

        assert_eq!(curve.total_records, 3);
        assert_eq!(curve.total_relevant, 2);
        assert_eq!(curve.points.len(), 3);

        // At low threshold (0.1), both relevant items are above threshold: TP=2, FP=0, TN=1, FN=0 (since rec-3 score is 0.0)
        let Some(low) = curve.points.first() else {
            panic!("low threshold point must exist");
        };
        assert_eq!(low.threshold, 0.1);
        assert_eq!(low.counts.true_positive, 2);
        assert_eq!(low.counts.false_negative, 0);
        assert!((low.recall - 1.0).abs() < 1e-9);
        assert!((low.precision - 1.0).abs() < 1e-9);
        assert!((low.false_negative_rate - 0.0).abs() < 1e-9);
        // WSS = recall - (TP + FP) / total = 1.0 - 2.0/3.0 = 1/3
        assert!((low.work_saved_over_sampling - (1.0 / 3.0)).abs() < 1e-9);

        // At high threshold (0.9), no item exceeds threshold: TP=0, FP=0, TN=1, FN=2
        let Some(high) = curve.points.get(2) else {
            panic!("high threshold point must exist");
        };
        assert_eq!(high.threshold, 0.9);
        assert_eq!(high.counts.true_positive, 0);
        assert_eq!(high.counts.false_negative, 2);
        assert!((high.recall - 0.0).abs() < 1e-9);
        assert!((high.false_negative_rate - 1.0).abs() < 1e-9);
        // Zero division protection: precision is 0.0 when predicted positive is 0
        assert_eq!(high.precision, 0.0);
        // WSS = 0.0 - 0.0 / 3.0 = 0.0
        assert_eq!(high.work_saved_over_sampling, 0.0);
    }

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "zero division protection explicitly asserts exact 0.0 float proportions"
    )]
    fn calibrate_zero_division_protection() {
        let empty_scores: Vec<RankingScore> = Vec::new();
        let relevant = BTreeSet::new();

        let curve = calibrate(&empty_scores, &relevant, &[0.5]);
        assert_eq!(curve.total_records, 0);
        assert_eq!(curve.total_relevant, 0);
        assert_eq!(curve.points.len(), 1);

        let Some(point) = curve.points.first() else {
            panic!("empty curve must contain evaluated threshold point");
        };
        assert_eq!(point.recall, 0.0);
        assert_eq!(point.precision, 0.0);
        assert_eq!(point.false_negative_rate, 0.0);
        assert_eq!(point.false_positive_rate, 0.0);
        assert_eq!(point.work_saved_over_sampling, 0.0);

        // Non-empty scores with NO relevant records
        let records = sample_records();
        let query = vec!["machine".to_owned()];
        let ranker = LexicalRanker::default();
        let Ok(scores) = ranker.score(&records, &query) else {
            panic!("scoring must succeed");
        };
        let no_relevant = BTreeSet::new();
        let curve_no_rel = calibrate(&scores, &no_relevant, &[0.1]);
        let Some(no_rel_point) = curve_no_rel.points.first() else {
            panic!("curve_no_rel must contain point");
        };
        assert_eq!(no_rel_point.recall, 0.0);
        assert_eq!(no_rel_point.false_negative_rate, 0.0);

        // Non-empty scores with ALL relevant records (no irrelevant)
        let mut all_relevant = BTreeSet::new();
        for r in &records {
            all_relevant.insert(r.record_id.clone());
        }
        let curve_all_rel = calibrate(&scores, &all_relevant, &[0.1]);
        let Some(all_rel_point) = curve_all_rel.points.first() else {
            panic!("curve_all_rel must contain point");
        };
        assert_eq!(all_rel_point.false_positive_rate, 0.0);
    }

    #[test]
    fn calibrate_advisory_contract_enforces_no_auto_exclusion() {
        let records = sample_records();
        let query = vec!["machine".to_owned(), "learning".to_owned()];
        let ranker = LexicalRanker::default();
        let Ok(scores) = ranker.score(&records, &query) else {
            panic!("scoring must succeed");
        };

        let mut relevant = BTreeSet::new();
        relevant.insert("rec-1".to_owned());
        relevant.insert("rec-2".to_owned());

        let thresholds = vec![0.1, 0.5];
        let curve = calibrate(&scores, &relevant, &thresholds);

        // Build valid approved calibration at threshold 0.1 (where recall = 1.0 >= 0.95)
        let Ok(calibration) = curve.to_ranking_calibration(
            "rev-test-1",
            "lexical-v1",
            0.1,
            0.95,
            Some("human-reviewer-1"),
        ) else {
            panic!("valid calibration conversion must succeed");
        };
        assert!(calibration.auto_exclusion_prohibited);
        assert!(calibration.approved_for_prioritisation);
        assert_eq!(calibration.approved_by.as_deref(), Some("human-reviewer-1"));

        // Validate via validate_calibration
        let Ok(()) = validate_calibration(&calibration) else {
            panic!("validate_calibration must succeed on valid calibration");
        };

        // Sensitivity below minimum sensitivity must fail conversion/validation
        let low_sens_result = curve.to_ranking_calibration(
            "rev-test-2",
            "lexical-v1",
            0.5,
            0.95, // threshold 0.5 may have lower recall than 0.95
            Some("human-reviewer-1"),
        );
        // At threshold 0.5 only rec-1 is >= 0.5 (recall = 0.5 < 0.95), so it fails
        let Err(_) = low_sens_result else {
            panic!("threshold with sensitivity < minimum must fail validation");
        };

        // Tampering to disable auto_exclusion_prohibited fails validate_calibration
        let mut tampered = calibration;
        tampered.auto_exclusion_prohibited = false;
        let Err(_) = validate_calibration(&tampered) else {
            panic!("tampered calibration with auto_exclusion_prohibited=false must be rejected");
        };
    }
}
