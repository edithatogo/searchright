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

/// Pair accuracy and the amount of proposed clustering that still requires review.
#[derive(Debug, Clone, PartialEq)]
pub struct DedupEvaluation {
    /// Pairwise precision, recall and F1 against the supplied visible labels.
    pub pair_metrics: BinaryMetrics,
    /// Number of proposed duplicate clusters.
    pub proposed_clusters: u64,
    /// Number of clusters containing at least one review-required match.
    pub review_required_clusters: u64,
    /// Unique records appearing in a review-required cluster.
    pub review_required_records: u64,
}

/// Compare expected and observed sets of identifiers or identifier pairs.
#[must_use]
pub fn binary_metrics<T: Ord>(expected: &BTreeSet<T>, observed: &BTreeSet<T>) -> BinaryMetrics {
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
pub fn recall_at_k(scores: &[RankingScore], relevant: &BTreeSet<String>, k: usize) -> Option<f64> {
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

/// Convert labelled clusters into unordered expected duplicate pairs.
///
/// Singleton clusters contribute no pair but retain their identifier in the
/// duplicate-label integrity check. An identifier appearing in more than one
/// cluster is rejected because that would make the gold partition ambiguous.
pub fn expected_dedup_pairs(
    expected_clusters: &[Vec<String>],
) -> Result<BTreeSet<(String, String)>, BenchmarkError> {
    let mut seen = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for cluster in expected_clusters {
        if cluster.is_empty() {
            return Err(BenchmarkError::EmptyExpectedCluster);
        }
        for identifier in cluster {
            if identifier.trim().is_empty() {
                return Err(BenchmarkError::EmptyExpectedRecordId);
            }
            if !seen.insert(identifier.as_str()) {
                return Err(BenchmarkError::DuplicateExpectedRecordId(
                    identifier.clone(),
                ));
            }
        }
        for (index, left) in cluster.iter().enumerate() {
            for right in cluster.iter().skip(index.saturating_add(1)) {
                if left <= right {
                    pairs.insert((left.clone(), right.clone()));
                } else {
                    pairs.insert((right.clone(), left.clone()));
                }
            }
        }
    }
    Ok(pairs)
}

/// Evaluate one deduplication result against visible, rights-clear labels.
///
/// This function deliberately accepts labels from its caller and never reads
/// the sealed benchmark partition. Its result supports fixture regression
/// testing only; it does not establish external benchmark performance.
pub fn evaluate_dedup(
    expected_clusters: &[Vec<String>],
    observed: &DedupResult,
) -> Result<DedupEvaluation, BenchmarkError> {
    let expected = expected_dedup_pairs(expected_clusters)?;
    let observed_pairs = dedup_pairs(observed);
    let mut review_required_records = BTreeSet::new();
    let review_required_clusters = observed
        .clusters
        .iter()
        .filter(|cluster| {
            let requires_review = cluster.evidence.iter().any(|item| item.review_required);
            if requires_review {
                review_required_records.extend(cluster.record_ids.iter().cloned());
            }
            requires_review
        })
        .count();
    Ok(DedupEvaluation {
        pair_metrics: binary_metrics(&expected, &observed_pairs),
        proposed_clusters: usize_to_u64(observed.clusters.len()),
        review_required_clusters: usize_to_u64(review_required_clusters),
        review_required_records: usize_to_u64(review_required_records.len()),
    })
}

/// Build stable benchmark metrics for a deduplication evaluation.
#[must_use]
pub fn dedup_metric_contracts(evaluation: &DedupEvaluation) -> Vec<BenchmarkMetric> {
    let mut metrics = metric_contracts("dedup.pair", evaluation.pair_metrics);
    let proposed_clusters = evaluation.proposed_clusters;
    if let Some(value) = ratio(evaluation.review_required_clusters, proposed_clusters) {
        metrics.push(BenchmarkMetric {
            name: "dedup.review_required_cluster_rate".to_owned(),
            value,
            unit: "proportion".to_owned(),
            lower_bound: None,
            upper_bound: None,
            sample_size: proposed_clusters,
        });
    }
    metrics.push(BenchmarkMetric {
        name: "dedup.review_required_records".to_owned(),
        value: u64_to_f64(evaluation.review_required_records),
        unit: "records".to_owned(),
        lower_bound: None,
        upper_bound: None,
        sample_size: evaluation.review_required_records,
    });
    metrics
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

#[allow(
    clippy::cast_precision_loss,
    reason = "benchmark counts are bounded fixture values represented in the metric contract as f64"
)]
const fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

/// Benchmark-gate failure.
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    /// Metric inputs were not finite or tolerance was negative.
    #[error("benchmark regression inputs must be finite and tolerance non-negative")]
    InvalidRegressionInput,
    /// A labelled expected cluster contained no records.
    #[error("expected duplicate cluster must contain at least one record")]
    EmptyExpectedCluster,
    /// A labelled expected record identifier was blank.
    #[error("expected duplicate record identifier must not be blank")]
    EmptyExpectedRecordId,
    /// One record appeared in more than one labelled expected cluster.
    #[error("expected duplicate record identifier `{0}` appears in multiple clusters")]
    DuplicateExpectedRecordId(String),
    /// Candidate metric dropped beyond the configured threshold.
    #[error(
        "metric regressed from {baseline} to {candidate}; maximum permitted drop is {maximum_absolute_drop}"
    )]
    Regression {
        /// Previously accepted metric value used as the comparison baseline.
        baseline: f64,
        /// Newly measured metric value being evaluated by the gate.
        candidate: f64,
        /// Largest absolute decrease allowed before the gate reports a regression.
        maximum_absolute_drop: f64,
    },
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{BibliographicRecord, RecordIdentifiers, RecordKind};
    use searchright_dedup::{DedupConfig, Deduplicator, DuplicateCluster, MatchEvidence};
    use serde::Deserialize;
    use serde_json::Value;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct DedupFixture {
        schema_version: String,
        partition: String,
        records: Vec<FixtureRecord>,
        gold_clusters: Vec<Vec<String>>,
        rights_basis: String,
    }

    #[derive(Debug, Deserialize)]
    struct FixtureRecord {
        id: String,
        title: String,
        doi: Option<String>,
    }

    fn benchmark_record(record: FixtureRecord) -> BibliographicRecord {
        BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: record.id.clone(),
            source_receipt_id: "rights-clear-validation-fixture".to_owned(),
            native_id: record.id,
            kind: RecordKind::JournalArticle,
            identifiers: RecordIdentifiers {
                doi: record.doi,
                ..RecordIdentifiers::default()
            },
            title: record.title,
            abstract_text: None,
            authors: Vec::new(),
            container_title: None,
            publication_year: None,
            publication_date: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            urls: Vec::new(),
            provider_metadata: Value::Null,
        }
    }

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

    #[test]
    fn rights_clear_validation_fixture_executes_compiled_dedup_evaluation()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture: DedupFixture = serde_json::from_str(include_str!(
            "../../../benchmarks/methodology/fixtures/validation/dedup-cases.json"
        ))?;
        assert_eq!(
            fixture.schema_version,
            "org.searchright.dedup-benchmark-fixture.v1"
        );
        assert_eq!(fixture.partition, "validation");
        assert_eq!(fixture.rights_basis, "CC0 synthetic metadata");
        let records = fixture
            .records
            .into_iter()
            .map(benchmark_record)
            .collect::<Vec<_>>();
        let observed = Deduplicator::new(DedupConfig::default())?.cluster(&records)?;
        let evaluation = evaluate_dedup(&fixture.gold_clusters, &observed)?;

        assert_eq!(evaluation.pair_metrics.true_positive, 1);
        assert_eq!(evaluation.pair_metrics.false_positive, 0);
        assert_eq!(evaluation.pair_metrics.false_negative, 0);
        assert_eq!(evaluation.proposed_clusters, 1);
        assert_eq!(evaluation.review_required_clusters, 0);
        assert_eq!(evaluation.review_required_records, 0);
        let metrics = dedup_metric_contracts(&evaluation);
        assert_eq!(
            metrics
                .iter()
                .map(|metric| metric.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "dedup.pair.precision",
                "dedup.pair.recall",
                "dedup.pair.f1",
                "dedup.review_required_cluster_rate",
                "dedup.review_required_records"
            ]
        );
        Ok(())
    }

    #[test]
    fn review_burden_counts_unique_records_in_review_required_clusters()
    -> Result<(), BenchmarkError> {
        let observed = DedupResult {
            clusters: vec![DuplicateCluster {
                cluster_id: "fixture-cluster".to_owned(),
                representative_record_id: "a".to_owned(),
                record_ids: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
                evidence: vec![MatchEvidence {
                    left_record_id: "a".to_owned(),
                    right_record_id: "b".to_owned(),
                    reason: "fuzzy_title_author_year".to_owned(),
                    score: 0.95,
                    details: std::collections::BTreeMap::default(),
                    review_required: true,
                }],
            }],
            retained_record_ids: vec!["a".to_owned()],
            comparisons: 3,
            proposed_duplicate_count: 2,
        };
        let expected = vec![vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]];
        let evaluation = evaluate_dedup(&expected, &observed)?;
        assert_eq!(evaluation.review_required_clusters, 1);
        assert_eq!(evaluation.review_required_records, 3);
        Ok(())
    }

    #[test]
    fn ambiguous_expected_clusters_are_rejected() {
        let duplicate = vec![vec!["a".to_owned()], vec!["a".to_owned()]];
        assert!(matches!(
            expected_dedup_pairs(&duplicate),
            Err(BenchmarkError::DuplicateExpectedRecordId(identifier)) if identifier == "a"
        ));
        assert!(matches!(
            expected_dedup_pairs(&[Vec::new()]),
            Err(BenchmarkError::EmptyExpectedCluster)
        ));
        assert!(matches!(
            expected_dedup_pairs(&[vec!["  ".to_owned()]]),
            Err(BenchmarkError::EmptyExpectedRecordId)
        ));
    }
}
