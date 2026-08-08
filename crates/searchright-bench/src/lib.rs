//! Deterministic metrics for retrieval, ranking, deduplication and regression checks.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use searchright_contracts::{BenchmarkMetric, RankingScore};
use searchright_dedup::DedupResult;

/// Binary classification or retrieval metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinaryMetrics {
    /// True positives.
    pub true_positive: u64,
    /// False positives.
    pub false_positive: u64,
    /// False negatives.
    pub false_negative: u64,
    /// Precision when at least one positive prediction exists.
    pub precision: Option<f64>,
    /// Recall when at least one expected positive exists.
    pub recall: Option<f64>,
    /// Harmonic mean when precision and recall are both defined.
    pub f1: Option<f64>,
}

/// Compare expected and observed identifier sets.
#[must_use]
pub fn binary_metrics(
    expected: &BTreeSet<String>,
    observed: &BTreeSet<String>,
) -> BinaryMetrics {
    let true_positive = usize_to_u64(expected.intersection(observed).count());
    let false_positive = usize_to_u64(observed.difference(expected).count());
    let false_negative = usize_to_u64(expected.difference(observed).count());
    let precision = ratio(true_positive, true_positive.saturating_add(false_positive));
    let recall = ratio(true_positive, true_positive.saturating_add(false_negative));
    let f1 = precision.zip(recall).and_then(|(precision, recall)| {
        let denominator = precision + recall;
        (denominator > 0.0).then_some(2.0 * precision * recall / denominator)
    });
    BinaryMetrics {
        true_positive,
        false_positive,
        false_negative,
        precision,
        recall,
        f1,
    }
}

/// Measure known-relevant recall among the first `k` advisory ranking results.
#[must_use]
pub fn recall_at_k(
    scores: &[RankingScore],
    relevant: &BTreeSet<String>,
    k: usize,
) -> Option<f64> {
    if relevant.is_empty() {
        return None;
    }
    let observed = scores
        .iter()
        .take(k)
        .map(|score| score.record_id.clone())
        .collect::<BTreeSet<_>>();
    binary_metrics(relevant, &observed).recall
}

/// Extract all unordered duplicate pairs proposed by a deduplication run.
#[must_use]
pub fn dedup_pairs(result: &DedupResult) -> BTreeSet<(String, String)> {
    let mut pairs = BTreeSet::new();
    for cluster in &result.clusters {
        for (index, left) in cluster.record_ids.iter().enumerate() {
            for right in cluster.record_ids.iter().skip(index.saturating_add(1)) {
                if left <= right {
                    pairs.insert((left.clone(), right.clone()));
                } else {
                    pairs.insert((right.clone(), left.clone()));
                }
            }
        }
    }
    pairs
}

/// Build standard proportion metrics from binary counts.
#[must_use]
pub fn metric_contracts(prefix: &str, metrics: BinaryMetrics) -> Vec<BenchmarkMetric> {
    let sample_size = metrics
        .true_positive
        .saturating_add(metrics.false_positive)
        .saturating_add(metrics.false_negative);
    [
        ("precision", metrics.precision),
        ("recall", metrics.recall),
        ("f1", metrics.f1),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        value.map(|value| BenchmarkMetric {
            name: format!("{prefix}.{name}"),
            value,
            unit: "proportion".to_owned(),
            lower_bound: None,
            upper_bound: None,
            sample_size,
        })
    })
    .collect()
}

/// Reject a metric regression beyond an explicit absolute tolerance.
pub fn regression_gate(
    baseline: f64,
    candidate: f64,
    maximum_absolute_drop: f64,
) -> Result<(), BenchmarkError> {
    if !baseline.is_finite()
        || !candidate.is_finite()
        || !maximum_absolute_drop.is_finite()
        || maximum_absolute_drop < 0.0
    {
        return Err(BenchmarkError::InvalidRegressionInput);
    }
    let drop = baseline - candidate;
    if drop > maximum_absolute_drop {
        Err(BenchmarkError::Regression {
            baseline,
            candidate,
            maximum_absolute_drop,
        })
    } else {
        Ok(())
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "benchmark proportions intentionally project bounded integer counts into f64"
)]
fn ratio(numerator: u64, denominator: u64) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Benchmark-gate failure.
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    /// Metric inputs were not finite or tolerance was negative.
    #[error("benchmark regression inputs must be finite and tolerance non-negative")]
    InvalidRegressionInput,
    /// Candidate metric dropped beyond the configured threshold.
    #[error(
        "metric regressed from {baseline} to {candidate}; maximum permitted drop is {maximum_absolute_drop}"
    )]
    Regression {
        baseline: f64,
        candidate: f64,
        maximum_absolute_drop: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "these proportions have exact binary representations and form a deterministic regression fixture"
    )]
    fn binary_metrics_are_exact_for_small_sets() {
        let expected = ["a", "b"].into_iter().map(str::to_owned).collect();
        let observed = ["b", "c"].into_iter().map(str::to_owned).collect();
        let metrics = binary_metrics(&expected, &observed);
        assert_eq!(metrics.true_positive, 1);
        assert_eq!(metrics.false_positive, 1);
        assert_eq!(metrics.false_negative, 1);
        assert_eq!(metrics.precision, Some(0.5));
        assert_eq!(metrics.recall, Some(0.5));
        assert_eq!(metrics.f1, Some(0.5));
    }

    #[test]
    fn configured_regression_tolerance_is_enforced() {
        assert!(regression_gate(0.99, 0.985, 0.01).is_ok());
        assert!(matches!(
            regression_gate(0.99, 0.90, 0.01),
            Err(BenchmarkError::Regression { .. })
        ));
    }
}
