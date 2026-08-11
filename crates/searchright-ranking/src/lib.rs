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
