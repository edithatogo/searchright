//! Deterministic metrics for retrieval, ranking, deduplication and regression checks.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use searchright_contracts::{BenchmarkMetric, BenchmarkReport, RankingScore, Validate};
use searchright_dedup::DedupResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

/// Report-to-study linkage quality and explicit abstention burden.
#[derive(Debug, Clone, PartialEq)]
pub struct StudyLinkageEvaluation {
    /// Pairwise report linkage precision, recall and F1.
    pub link_metrics: BinaryMetrics,
    /// Reports for which the linker made no study assignment.
    pub abstained_reports: u64,
    /// Total reports in the labelled fixture.
    pub total_reports: u64,
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

/// Evaluate a report-study partition against visible, rights-clear labels.
///
/// This is a local regression path only. It does not read sealed labels and
/// cannot establish external linkage performance.
pub fn evaluate_study_linkage(
    expected_studies: &[Vec<String>],
    observed_studies: &[Vec<String>],
    abstained_report_ids: &BTreeSet<String>,
) -> Result<StudyLinkageEvaluation, BenchmarkError> {
    let expected = expected_dedup_pairs(expected_studies)?;
    let observed = expected_dedup_pairs(observed_studies)?;
    let expected_report_ids = expected_studies
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    let observed_report_ids = observed_studies
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    if let Some(identifier) = abstained_report_ids.difference(&expected_report_ids).next() {
        return Err(BenchmarkError::UnknownAbstentionReport(identifier.clone()));
    }
    if let Some(identifier) = observed_report_ids.difference(&expected_report_ids).next() {
        return Err(BenchmarkError::UnknownObservedReport(identifier.clone()));
    }
    if let Some(identifier) = observed_report_ids
        .intersection(abstained_report_ids)
        .next()
    {
        return Err(BenchmarkError::AssignedAndAbstainedReport(
            identifier.clone(),
        ));
    }
    let accounted_report_ids = observed_report_ids
        .union(abstained_report_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    if let Some(identifier) = expected_report_ids.difference(&accounted_report_ids).next() {
        return Err(BenchmarkError::UnaccountedExpectedReport(
            identifier.clone(),
        ));
    }
    let total_reports = usize_to_u64(
        expected_studies
            .iter()
            .map(Vec::len)
            .fold(0_usize, usize::saturating_add),
    );
    if abstained_report_ids.len() > usize::try_from(total_reports).unwrap_or(usize::MAX) {
        return Err(BenchmarkError::InvalidAbstentionCount);
    }
    Ok(StudyLinkageEvaluation {
        link_metrics: binary_metrics(&expected, &observed),
        abstained_reports: usize_to_u64(abstained_report_ids.len()),
        total_reports,
    })
}

/// Build the declared study-linkage metric contracts.
#[must_use]
pub fn study_linkage_metric_contracts(evaluation: &StudyLinkageEvaluation) -> Vec<BenchmarkMetric> {
    let pair_sample = evaluation
        .link_metrics
        .true_positive
        .saturating_add(evaluation.link_metrics.false_positive)
        .saturating_add(evaluation.link_metrics.false_negative);
    let mut metrics = Vec::new();
    for (name, value) in [
        ("link_precision", evaluation.link_metrics.precision),
        ("link_recall", evaluation.link_metrics.recall),
        ("study_cluster_f1", evaluation.link_metrics.f1),
    ] {
        if let Some(value) = value {
            metrics.push(BenchmarkMetric {
                name: name.to_owned(),
                value,
                unit: "proportion".to_owned(),
                lower_bound: None,
                upper_bound: None,
                sample_size: pair_sample,
            });
        }
    }
    if let Some(value) = ratio(evaluation.abstained_reports, evaluation.total_reports) {
        metrics.push(BenchmarkMetric {
            name: "abstention_rate".to_owned(),
            value,
            unit: "proportion".to_owned(),
            lower_bound: None,
            upper_bound: None,
            sample_size: evaluation.total_reports,
        });
    }
    metrics
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

/// Manifest describing a sealed benchmark partition split and its integrity controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedSplitManifest {
    /// Schema version (e.g. `org.searchright.sealed-benchmark-manifest.v1`).
    pub schema_version: String,
    /// Partition identifier (e.g. `sealed_test`).
    pub partition: String,
    /// Unique case identifiers in the sealed partition.
    pub case_ids: Vec<String>,
    /// Whether true labels are present in the repository (must be false).
    pub labels_present: bool,
    /// Location of external evaluation labels.
    pub labels_location: String,
    /// Optional canonical SHA-256 digest from [`sealed_label_digest`] when labels are mounted.
    pub label_digest: Option<String>,
    /// Release conditions governing the sealed split.
    pub release_condition: String,
    /// Explicit claim boundary.
    pub claim_boundary: String,
}

impl SealedSplitManifest {
    /// Validate that the sealed split manifest adheres to anti-leakage policy.
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_version != "org.searchright.sealed-benchmark-manifest.v1" {
            return Err(BenchmarkError::InvalidManifest(format!(
                "expected schema version org.searchright.sealed-benchmark-manifest.v1, got `{}`",
                self.schema_version
            )));
        }
        if self.partition != "sealed_test" {
            return Err(BenchmarkError::InvalidManifest(format!(
                "expected partition sealed_test, got `{}`",
                self.partition
            )));
        }
        if self.case_ids.is_empty() {
            return Err(BenchmarkError::InvalidManifest(
                "sealed case identifiers must not be empty".to_owned(),
            ));
        }
        let mut seen = BTreeSet::new();
        for case_id in &self.case_ids {
            if case_id.trim().is_empty() {
                return Err(BenchmarkError::InvalidManifest(
                    "sealed case identifier must not be blank".to_owned(),
                ));
            }
            if !seen.insert(case_id.as_str()) {
                return Err(BenchmarkError::InvalidManifest(format!(
                    "duplicate sealed case identifier `{case_id}`"
                )));
            }
        }
        if self.labels_present {
            return Err(BenchmarkError::InvalidManifest(
                "sealed manifest labels_present must be false in repository".to_owned(),
            ));
        }
        if let Some(digest) = &self.label_digest {
            validate_label_digest(digest)?;
        }
        if self.release_condition.trim().is_empty() {
            return Err(BenchmarkError::InvalidManifest(
                "release condition must not be blank".to_owned(),
            ));
        }
        if self.claim_boundary.trim().is_empty() {
            return Err(BenchmarkError::InvalidManifest(
                "claim boundary must not be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Check if any sealed identifiers appear in a candidate text snippet or file.
#[must_use]
pub fn check_split_contamination(manifest: &SealedSplitManifest, content: &str) -> Vec<String> {
    manifest
        .case_ids
        .iter()
        .filter(|case_id| content.contains(case_id.as_str()))
        .cloned()
        .collect()
}

/// Digest external binary labels using the sealed-labels v1 encoding.
///
/// SHA-256 covers the bytes `org.searchright.sealed-labels.v1\0`, followed by the
/// label count as an unsigned 64-bit big-endian integer. Labels are sorted by the
/// exact UTF-8 bytes of their identifiers. Each entry contributes its identifier's
/// byte length as an unsigned 64-bit big-endian integer, the identifier bytes,
/// and one byte (`0` for false, `1` for true). The result is lowercase hexadecimal.
/// Identifiers are not trimmed or Unicode-normalised. Empty collections, blank
/// identifiers and duplicate identifiers are rejected.
///
/// Compute and retain this commitment in the separate evaluation environment
/// before candidate scoring. A digest supplied by the same party as mutable
/// labels cannot establish that the labels were frozen or kept from a model.
pub fn sealed_label_digest(labels: &[(String, bool)]) -> Result<String, BenchmarkError> {
    digest_label_map(&checked_label_map(labels)?)
}

fn checked_label_map(labels: &[(String, bool)]) -> Result<BTreeMap<&str, bool>, BenchmarkError> {
    if labels.is_empty() {
        return Err(BenchmarkError::EmptyLabels);
    }
    let mut label_map = BTreeMap::new();
    for (identifier, positive) in labels {
        if identifier.trim().is_empty() {
            return Err(BenchmarkError::EmptyLabelRecordId);
        }
        if label_map.insert(identifier.as_str(), *positive).is_some() {
            return Err(BenchmarkError::DuplicateLabelRecord(identifier.clone()));
        }
    }
    Ok(label_map)
}

fn digest_label_map(labels: &BTreeMap<&str, bool>) -> Result<String, BenchmarkError> {
    let mut digest = Sha256::new();
    digest.update(b"org.searchright.sealed-labels.v1\0");
    let count = u64::try_from(labels.len()).map_err(|_| BenchmarkError::LabelEncodingTooLarge)?;
    digest.update(count.to_be_bytes());
    for (identifier, positive) in labels {
        let length =
            u64::try_from(identifier.len()).map_err(|_| BenchmarkError::LabelEncodingTooLarge)?;
        digest.update(length.to_be_bytes());
        digest.update(identifier.as_bytes());
        digest.update([u8::from(*positive)]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn validate_label_digest(digest: &str) -> Result<(), BenchmarkError> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(BenchmarkError::InvalidLabelDigest(digest.to_owned()));
    }
    Ok(())
}

/// Evaluate complete binary predictions against external labels for one sealed population.
///
/// Every manifest identifier must occur exactly once in both labels and predictions;
/// unknown, missing and duplicate identifiers are errors. Missing predictions are
/// never interpreted as negative decisions or uncounted abstentions. Workflows
/// with abstentions need an explicit evaluation protocol rather than this API.
///
/// The supplied digest must equal [`sealed_label_digest`] for the actual labels,
/// as must the manifest digest when present. These checks bind the scored population
/// and values; they do not prove independent label custody or absence of leakage.
pub fn evaluate_sealed_benchmark(
    manifest: &SealedSplitManifest,
    label_digest: &str,
    predictions: &[(String, bool)],
    labels: &[(String, bool)],
) -> Result<BinaryMetrics, BenchmarkError> {
    manifest.validate()?;
    validate_label_digest(label_digest)?;
    let label_map = checked_label_map(labels)?;
    let cases = manifest
        .case_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for identifier in label_map.keys() {
        if !cases.contains(identifier) {
            return Err(BenchmarkError::UnknownLabelRecord((*identifier).to_owned()));
        }
    }
    for identifier in &cases {
        if !label_map.contains_key(identifier) {
            return Err(BenchmarkError::MissingLabelRecord((*identifier).to_owned()));
        }
    }
    let computed_digest = digest_label_map(&label_map)?;
    if computed_digest != label_digest {
        return Err(BenchmarkError::LabelDigestMismatch);
    }
    if manifest
        .label_digest
        .as_ref()
        .is_some_and(|declared| declared != &computed_digest)
    {
        return Err(BenchmarkError::ManifestLabelDigestMismatch);
    }
    let mut prediction_map = BTreeMap::new();
    for (identifier, positive) in predictions {
        if !cases.contains(identifier.as_str()) {
            return Err(BenchmarkError::UnknownPredictionRecord(identifier.clone()));
        }
        if prediction_map
            .insert(identifier.as_str(), *positive)
            .is_some()
        {
            return Err(BenchmarkError::DuplicatePredictionRecord(
                identifier.clone(),
            ));
        }
    }
    for identifier in &cases {
        if !prediction_map.contains_key(identifier) {
            return Err(BenchmarkError::MissingPredictionRecord(
                (*identifier).to_owned(),
            ));
        }
    }
    let expected_positives = label_map
        .iter()
        .filter(|(_, positive)| **positive)
        .map(|(&identifier, _)| identifier)
        .collect();
    let observed_positives = prediction_map
        .iter()
        .filter(|(_, positive)| **positive)
        .map(|(&identifier, _)| identifier)
        .collect();
    Ok(binary_metrics(&expected_positives, &observed_positives))
}

/// Validate a public benchmark report to enforce schema compliance, non-empty metrics,
/// and explicit claim boundaries preventing unverified performance assertions.
pub fn validate_public_benchmark_report(report: &BenchmarkReport) -> Result<(), BenchmarkError> {
    report
        .validate()
        .map_err(|err| BenchmarkError::ContractValidation(err.to_string()))?;
    if report.metrics.is_empty() {
        return Err(BenchmarkError::EmptyMetrics);
    }
    if report.claim_boundary.trim().is_empty() {
        return Err(BenchmarkError::MissingClaimBoundary);
    }
    Ok(())
}

/// Calculate Work Saved over Sampling (WSS) at a target recall (e.g. WSS@95%).
///
/// Formula: `WSS = (total_records - reviewed_records) / total_records - (1.0 - target_recall)`.
/// Returns `None` if `total_records == 0` or `target_recall` is outside `0.0..=1.0`.
#[must_use]
pub fn work_saved_over_sampling(
    total_records: u64,
    reviewed_records: u64,
    target_recall: f64,
) -> Option<f64> {
    if total_records == 0 || !(0.0..=1.0).contains(&target_recall) {
        return None;
    }
    let total = u64_to_f64(total_records);
    let reviewed = u64_to_f64(reviewed_records.min(total_records));
    let sampling_fraction = reviewed / total;
    Some(target_recall - sampling_fraction)
}

/// Calculate Expected Calibration Error (ECE) with equal-width binning.
///
/// `predictions` are pairs of `(confidence_prob, true_outcome_bool)`.
/// Returns `None` if predictions is empty or `num_bins == 0`.
#[must_use]
pub fn expected_calibration_error(predictions: &[(f64, bool)], num_bins: usize) -> Option<f64> {
    if predictions.is_empty() || num_bins == 0 {
        return None;
    }
    let total = usize_to_u64(predictions.len());
    let total_f = u64_to_f64(total);
    let num_bins_f = u64_to_f64(usize_to_u64(num_bins));
    let mut bin_counts = vec![0_u64; num_bins];
    let mut bin_conf_sums = vec![0.0_f64; num_bins];
    let mut bin_acc_sums = vec![0.0_f64; num_bins];

    for &(conf, outcome) in predictions {
        if !conf.is_finite() || !(0.0..=1.0).contains(&conf) {
            return None;
        }
        let mut bin_idx = num_bins.saturating_sub(1);
        for candidate_idx in 0..num_bins.saturating_sub(1) {
            let upper = u64_to_f64(usize_to_u64(candidate_idx.saturating_add(1))) / num_bins_f;
            if conf < upper {
                bin_idx = candidate_idx;
                break;
            }
        }
        if let Some(count) = bin_counts.get_mut(bin_idx) {
            *count = count.saturating_add(1);
        }
        if let Some(conf_sum) = bin_conf_sums.get_mut(bin_idx) {
            *conf_sum += conf;
        }
        if outcome && let Some(acc_sum) = bin_acc_sums.get_mut(bin_idx) {
            *acc_sum += 1.0;
        }
    }

    let mut ece = 0.0_f64;
    for ((&count, &conf_sum), &acc_sum) in bin_counts
        .iter()
        .zip(bin_conf_sums.iter())
        .zip(bin_acc_sums.iter())
    {
        if count > 0 {
            let count_f = u64_to_f64(count);
            let avg_conf = conf_sum / count_f;
            let avg_acc = acc_sum / count_f;
            let weight = count_f / total_f;
            ece = weight.mul_add((avg_acc - avg_conf).abs(), ece);
        }
    }
    Some(ece)
}

/// Calculate Brier Score (Mean Squared Error of probability forecasts).
///
/// `predictions` are pairs of `(confidence_prob, true_outcome_bool)`.
/// Returns `None` if predictions is empty or contains non-finite values outside `0.0..=1.0`.
#[must_use]
pub fn brier_score(predictions: &[(f64, bool)]) -> Option<f64> {
    if predictions.is_empty() {
        return None;
    }
    let mut sum_sq_error = 0.0_f64;
    for &(prob, outcome) in predictions {
        if !prob.is_finite() || !(0.0..=1.0).contains(&prob) {
            return None;
        }
        let y = if outcome { 1.0 } else { 0.0 };
        let diff = prob - y;
        sum_sq_error = f64::mul_add(diff, diff, sum_sq_error);
    }
    Some(sum_sq_error / u64_to_f64(usize_to_u64(predictions.len())))
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
    /// More abstentions were declared than labelled reports.
    #[error("study-linkage abstentions exceed the labelled report count")]
    InvalidAbstentionCount,
    /// An abstention named a report absent from the labelled fixture.
    #[error("study-linkage abstention references unknown report `{0}`")]
    UnknownAbstentionReport(String),
    /// An observed cluster named a report absent from the labelled fixture.
    #[error("study-linkage observation references unknown report `{0}`")]
    UnknownObservedReport(String),
    /// A report was both assigned and declared as an abstention.
    #[error("study-linkage report `{0}` is both assigned and abstained")]
    AssignedAndAbstainedReport(String),
    /// A labelled report was neither assigned nor explicitly abstained.
    #[error("study-linkage expected report `{0}` is unaccounted for")]
    UnaccountedExpectedReport(String),
    /// Invalid sealed benchmark split manifest.
    #[error("invalid sealed benchmark manifest: {0}")]
    InvalidManifest(String),
    /// Invalid label digest format.
    #[error("invalid label digest `{0}`; must be 64 lowercase hexadecimal characters")]
    InvalidLabelDigest(String),
    /// The supplied digest did not match the canonical external label bytes.
    #[error("label digest does not match the canonical sealed-labels v1 encoding")]
    LabelDigestMismatch,
    /// The manifest commitment did not match the actual external labels.
    #[error("manifest label digest does not match the evaluated labels")]
    ManifestLabelDigestMismatch,
    /// Labels collection was empty.
    #[error("sealed benchmark labels must not be empty")]
    EmptyLabels,
    /// A label had a blank record identifier.
    #[error("sealed benchmark label identifier must not be blank")]
    EmptyLabelRecordId,
    /// A label identifier appeared more than once, even with the same value.
    #[error("label identifier `{0}` appears more than once")]
    DuplicateLabelRecord(String),
    /// A label identifier was absent from the sealed population.
    #[error("label references record `{0}` outside the sealed manifest")]
    UnknownLabelRecord(String),
    /// A sealed-population identifier had no label.
    #[error("sealed manifest record `{0}` has no label")]
    MissingLabelRecord(String),
    /// A collection or identifier cannot fit the encoding's unsigned 64-bit length.
    #[error("sealed labels exceed the canonical encoding's 64-bit length limit")]
    LabelEncodingTooLarge,
    /// Prediction record ID was absent from ground truth labels.
    #[error("prediction references unknown record `{0}`")]
    UnknownPredictionRecord(String),
    /// A prediction identifier appeared more than once, even with the same value.
    #[error("prediction identifier `{0}` appears more than once")]
    DuplicatePredictionRecord(String),
    /// A sealed-population identifier had no explicit binary prediction.
    #[error("sealed manifest record `{0}` has no prediction; implicit abstentions are unsupported")]
    MissingPredictionRecord(String),
    /// Contract validation error.
    #[error("benchmark contract error: {0}")]
    ContractValidation(String),
    /// Benchmark report metrics array was empty.
    #[error("benchmark report must declare at least one metric")]
    EmptyMetrics,
    /// Benchmark report lacked a non-blank claim boundary.
    #[error("benchmark report must define a non-blank claim boundary")]
    MissingClaimBoundary,
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

    #[derive(Debug, Deserialize)]
    struct StudyLinkageFixture {
        schema_version: String,
        partition: String,
        reports: Vec<StudyLinkageFixtureReport>,
        gold_studies: Vec<Vec<String>>,
        rights_basis: String,
    }

    #[derive(Debug, Deserialize)]
    struct StudyLinkageFixtureReport {
        id: String,
        registration: Option<String>,
        kind: String,
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
    #[allow(
        clippy::float_cmp,
        reason = "the synthetic fixture produces exact zero and one proportions"
    )]
    fn rights_clear_study_linkage_fixture_executes_declared_metrics()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture: StudyLinkageFixture = serde_json::from_str(include_str!(
            "../../../benchmarks/methodology/fixtures/validation/study-linkage-cases.json"
        ))?;
        assert_eq!(
            fixture.schema_version,
            "org.searchright.study-linkage-benchmark-fixture.v1"
        );
        assert_eq!(fixture.partition, "validation");
        assert_eq!(fixture.rights_basis, "CC0 synthetic metadata");

        let mut by_registration = BTreeMap::<String, Vec<String>>::new();
        let mut abstained = BTreeSet::new();
        for report in fixture.reports {
            assert_ne!(report.kind.trim(), "");
            if let Some(registration) = report.registration.filter(|value| !value.trim().is_empty())
            {
                by_registration
                    .entry(registration)
                    .or_default()
                    .push(report.id);
            } else {
                abstained.insert(report.id);
            }
        }
        let observed = by_registration.into_values().collect::<Vec<_>>();
        let evaluation = evaluate_study_linkage(&fixture.gold_studies, &observed, &abstained)?;
        let metrics = study_linkage_metric_contracts(&evaluation);
        assert_eq!(
            metrics
                .iter()
                .map(|metric| metric.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "link_precision",
                "link_recall",
                "study_cluster_f1",
                "abstention_rate"
            ]
        );
        assert!(metrics.iter().all(|metric| {
            if metric.name == "abstention_rate" {
                metric.value == 0.0
            } else {
                metric.value == 1.0
            }
        }));
        Ok(())
    }

    #[test]
    fn study_linkage_requires_every_labelled_report_to_be_accounted_for() {
        let expected = vec![
            vec!["r1".to_owned(), "r2".to_owned()],
            vec!["r3".to_owned()],
        ];
        let missing_singleton = vec![vec!["r1".to_owned(), "r2".to_owned()]];
        assert!(matches!(
            evaluate_study_linkage(&expected, &missing_singleton, &BTreeSet::new()),
            Err(BenchmarkError::UnaccountedExpectedReport(identifier)) if identifier == "r3"
        ));

        let with_unknown = vec![
            vec!["r1".to_owned(), "r2".to_owned()],
            vec!["r3".to_owned()],
            vec!["r4".to_owned()],
        ];
        assert!(matches!(
            evaluate_study_linkage(&expected, &with_unknown, &BTreeSet::new()),
            Err(BenchmarkError::UnknownObservedReport(identifier)) if identifier == "r4"
        ));
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
                    details: BTreeMap::default(),
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

    #[test]
    fn sealed_manifest_validation_enforces_anti_leakage_invariants() {
        let valid_manifest = SealedSplitManifest {
            schema_version: "org.searchright.sealed-benchmark-manifest.v1".to_owned(),
            partition: "sealed_test".to_owned(),
            case_ids: vec!["case-001".to_owned(), "case-002".to_owned()],
            labels_present: false,
            labels_location: "external controlled evaluation mount".to_owned(),
            label_digest: None,
            release_condition: "multi-agent evaluation panel approval required".to_owned(),
            claim_boundary: "case IDs do not expose labels".to_owned(),
        };
        assert!(valid_manifest.validate().is_ok());

        let mut leaked_labels = valid_manifest.clone();
        leaked_labels.labels_present = true;
        assert!(matches!(
            leaked_labels.validate(),
            Err(BenchmarkError::InvalidManifest(_))
        ));

        let mut invalid_schema = valid_manifest.clone();
        invalid_schema.schema_version = "invalid-schema".to_owned();
        assert!(matches!(
            invalid_schema.validate(),
            Err(BenchmarkError::InvalidManifest(_))
        ));

        let mut duplicate_case = valid_manifest;
        duplicate_case.case_ids = vec!["case-001".to_owned(), "case-001".to_owned()];
        assert!(matches!(
            duplicate_case.validate(),
            Err(BenchmarkError::InvalidManifest(_))
        ));
    }

    #[test]
    fn split_contamination_detects_leaked_case_identifiers() {
        let manifest = SealedSplitManifest {
            schema_version: "org.searchright.sealed-benchmark-manifest.v1".to_owned(),
            partition: "sealed_test".to_owned(),
            case_ids: vec![
                "secret-case-alpha".to_owned(),
                "secret-case-beta".to_owned(),
            ],
            labels_present: false,
            labels_location: "external controlled evaluation mount".to_owned(),
            label_digest: None,
            release_condition: "panel review required".to_owned(),
            claim_boundary: "case IDs do not expose labels".to_owned(),
        };

        let clean_text = "This text discusses general systematic review benchmarks.";
        assert_eq!(
            check_split_contamination(&manifest, clean_text),
            Vec::<String>::new()
        );

        let contaminated_text = "Here we tune our prompts on secret-case-alpha for better scores.";
        let findings = check_split_contamination(&manifest, contaminated_text);
        assert_eq!(findings, vec!["secret-case-alpha".to_owned()]);
    }

    #[test]
    fn sealed_benchmark_evaluation_computes_exact_metrics() -> Result<(), BenchmarkError> {
        let manifest = SealedSplitManifest {
            schema_version: "org.searchright.sealed-benchmark-manifest.v1".to_owned(),
            partition: "sealed_test".to_owned(),
            case_ids: vec!["r1".to_owned(), "r2".to_owned(), "r3".to_owned()],
            labels_present: false,
            labels_location: "external controlled evaluation mount".to_owned(),
            label_digest: None,
            release_condition: "panel review required".to_owned(),
            claim_boundary: "no unverified performance claims".to_owned(),
        };
        let labels = vec![
            ("r1".to_owned(), true),
            ("r2".to_owned(), false),
            ("r3".to_owned(), true),
        ];
        let label_digest = sealed_label_digest(&labels)?;
        let predictions = vec![
            ("r1".to_owned(), true),
            ("r2".to_owned(), true),
            ("r3".to_owned(), false),
        ];

        let metrics = evaluate_sealed_benchmark(&manifest, &label_digest, &predictions, &labels)?;
        assert_eq!(metrics.true_positive, 1);
        assert_eq!(metrics.false_positive, 1);
        assert_eq!(metrics.false_negative, 1);
        assert_eq!(metrics.precision, Some(0.5));
        assert_eq!(metrics.recall, Some(0.5));

        // Invalid digest rejected
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest, "short-digest", &predictions, &labels),
            Err(BenchmarkError::InvalidLabelDigest(_))
        ));

        // Unknown prediction record rejected
        let invalid_preds = vec![("r999".to_owned(), true)];
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest, &label_digest, &invalid_preds, &labels),
            Err(BenchmarkError::UnknownPredictionRecord(_))
        ));

        Ok(())
    }

    #[test]
    fn public_benchmark_report_validation_accepts_valid_report_and_rejects_empty_metrics() {
        let valid_report = BenchmarkReport {
            schema_version: "org.searchright.benchmark-report.v1".to_owned(),
            benchmark_id: "bm-2026-11".to_owned(),
            corpus_id: "corpus-v1".to_owned(),
            corpus_version: "1.0.0".to_owned(),
            rights_basis: "CC0-1.0".to_owned(),
            implementation_version: "0.1.0-alpha.1".to_owned(),
            configuration_digest: "0".repeat(64),
            leakage_controls: vec!["sealed labels separate".to_owned()],
            metrics: vec![BenchmarkMetric {
                name: "recall_at_95".to_owned(),
                value: 0.95,
                unit: "proportion".to_owned(),
                lower_bound: None,
                upper_bound: None,
                sample_size: 100,
            }],
            environment: vec!["offline test".to_owned()],
            generated_at: "2026-11-04T00:00:00Z".to_owned(),
            claim_boundary: "synthetic metrics only".to_owned(),
        };

        assert!(validate_public_benchmark_report(&valid_report).is_ok());

        let mut empty_metrics = valid_report;
        empty_metrics.metrics.clear();
        assert!(matches!(
            validate_public_benchmark_report(&empty_metrics),
            Err(BenchmarkError::ContractValidation(_))
        ));
    }

    #[test]
    fn work_saved_over_sampling_matches_analytical_fractions() {
        let wss = work_saved_over_sampling(100, 20, 0.95);
        assert!(wss.is_some_and(|val| (val - 0.75).abs() < 1e-9));

        let wss_all = work_saved_over_sampling(100, 100, 0.95);
        assert!(wss_all.is_some_and(|val| (val - (-0.05)).abs() < 1e-9));

        assert_eq!(work_saved_over_sampling(0, 0, 0.95), None);
        assert_eq!(work_saved_over_sampling(100, 20, 1.5), None);
    }

    #[test]
    fn expected_calibration_error_and_brier_score_behave_analytically() {
        let mut perfect = Vec::new();
        for _ in 0..50 {
            perfect.push((1.0, true));
            perfect.push((0.0, false));
        }
        let ece = expected_calibration_error(&perfect, 10);
        assert_eq!(ece, Some(0.0));
        let brier = brier_score(&perfect);
        assert_eq!(brier, Some(0.0));

        let mut inverted = Vec::new();
        for _ in 0..50 {
            inverted.push((1.0, false));
            inverted.push((0.0, true));
        }
        let ece_inv = expected_calibration_error(&inverted, 10);
        assert_eq!(ece_inv, Some(1.0));
        let brier_inv = brier_score(&inverted);
        assert_eq!(brier_inv, Some(1.0));

        assert_eq!(expected_calibration_error(&[], 10), None);
        assert_eq!(brier_score(&[]), None);
    }
}
