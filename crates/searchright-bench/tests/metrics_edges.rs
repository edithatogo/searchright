//! Benchmark metric edge cases.

use std::collections::BTreeSet;

use searchright_bench::{
    BenchmarkError, SealedSplitManifest, binary_metrics, brier_score, check_split_contamination,
    evaluate_sealed_benchmark, expected_calibration_error, regression_gate, sealed_label_digest,
    work_saved_over_sampling,
};

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(ToString::to_string).collect()
}

#[test]
fn binary_metrics_perfect_overlap_has_f1_one() {
    let expected = set(&["a", "b"]);
    let metrics = binary_metrics(&expected, &expected);
    assert_eq!(metrics.true_positive, 2);
    assert_eq!(metrics.precision, Some(1.0));
    assert_eq!(metrics.recall, Some(1.0));
    assert_eq!(metrics.f1, Some(1.0));
}

#[test]
fn binary_metrics_disjoint_has_f1_none() {
    let metrics = binary_metrics(&set(&["a"]), &set(&["b"]));
    assert_eq!(metrics.true_positive, 0);
    assert_eq!(metrics.precision, Some(0.0));
    assert_eq!(metrics.recall, Some(0.0));
    assert_eq!(metrics.f1, None);
}

#[test]
fn binary_metrics_empty_expected_has_none_recall() {
    let metrics = binary_metrics(&BTreeSet::new(), &set(&["a"]));
    assert_eq!(metrics.recall, None);
    assert_eq!(metrics.precision, Some(0.0));
}

#[test]
fn regression_gate_allows_exact_maximum_drop() {
    assert!(regression_gate(1.0, 0.9, 0.1).is_ok());
}

#[test]
fn regression_gate_rejects_over_maximum_drop() {
    assert!(matches!(
        regression_gate(0.9, 0.80, 0.05),
        Err(BenchmarkError::Regression { .. })
    ));
}

#[test]
fn regression_gate_rejects_nonfinite_inputs() {
    assert!(matches!(
        regression_gate(f64::NAN, 0.9, 0.05),
        Err(BenchmarkError::InvalidRegressionInput)
    ));
    assert!(matches!(
        regression_gate(0.9, 0.9, -0.1),
        Err(BenchmarkError::InvalidRegressionInput)
    ));
    assert!(matches!(
        regression_gate(0.9, 0.9, f64::INFINITY),
        Err(BenchmarkError::InvalidRegressionInput)
    ));
}

#[test]
fn work_saved_zero_records_is_none() {
    assert_eq!(work_saved_over_sampling(0, 0, 0.95), None);
}

#[test]
fn work_saved_invalid_target_recall_is_none() {
    assert_eq!(work_saved_over_sampling(100, 10, 1.5), None);
    assert_eq!(work_saved_over_sampling(100, 10, -0.1), None);
}

#[test]
fn work_saved_caps_reviewed_at_total() {
    let result = work_saved_over_sampling(100, 200, 0.95).unwrap_or_else(|| panic!("expected wss"));
    assert!((result - -0.05).abs() < 1e-12);
}

#[test]
fn calibration_error_empty_or_zero_bins_is_none() {
    assert_eq!(expected_calibration_error(&[], 10), None);
    assert_eq!(expected_calibration_error(&[(0.9, true)], 0), None);
}

#[test]
fn brier_score_perfect_predictions_is_zero() {
    let result = brier_score(&[(1.0, true), (0.0, false)]);
    assert_eq!(result, Some(0.0));
}

#[test]
fn brier_score_empty_is_none() {
    assert_eq!(brier_score(&[]), None);
}

fn manifest() -> SealedSplitManifest {
    SealedSplitManifest {
        schema_version: "org.searchright.sealed-benchmark-manifest.v1".to_string(),
        partition: "sealed_test".to_string(),
        case_ids: vec!["case-1".to_string(), "case-2".to_string()],
        labels_present: false,
        labels_location: "external-vault".to_string(),
        label_digest: None,
        release_condition: "all gates clear".to_string(),
        claim_boundary: "no performance claim".to_string(),
    }
}

#[test]
fn sealed_manifest_happy_path_passes() {
    assert!(manifest().validate().is_ok());
}

#[test]
fn sealed_manifest_rejects_wrong_schema_or_partition() {
    let mut m = manifest();
    m.schema_version = "wrong".to_string();
    assert!(m.validate().is_err());
    let mut m2 = manifest();
    m2.partition = "visible".to_string();
    assert!(m2.validate().is_err());
}

#[test]
fn sealed_manifest_rejects_empty_duplicate_and_blank_cases() {
    let mut m = manifest();
    m.case_ids.clear();
    assert!(m.validate().is_err());
    let mut m2 = manifest();
    m2.case_ids.push("case-1".to_string());
    assert!(m2.validate().is_err());
    let mut m3 = manifest();
    m3.case_ids.push("  ".to_string());
    assert!(m3.validate().is_err());
}

#[test]
fn sealed_manifest_rejects_labels_in_repo() {
    let mut m = manifest();
    m.labels_present = true;
    assert!(m.validate().is_err());
}

#[test]
fn contamination_check_detects_literal_id() {
    let hits = check_split_contamination(&manifest(), "text with case-2 inside");
    assert_eq!(hits, vec!["case-2"]);
}

#[test]
fn sealed_evaluation_rejects_bad_digest_and_unknown_prediction() -> Result<(), BenchmarkError> {
    let m = manifest();
    let labels = vec![("case-1".to_string(), true), ("case-2".to_string(), false)];
    let digest = sealed_label_digest(&labels)?;
    assert!(matches!(
        evaluate_sealed_benchmark(&m, "short", &[("case-1".to_string(), true)], &labels),
        Err(BenchmarkError::InvalidLabelDigest(_))
    ));
    assert!(matches!(
        evaluate_sealed_benchmark(&m, &digest, &[("ghost".to_string(), true)], &labels),
        Err(BenchmarkError::UnknownPredictionRecord(_))
    ));
    Ok(())
}

#[test]
fn sealed_evaluation_computes_positive_metrics() -> Result<(), BenchmarkError> {
    let m = manifest();
    let labels = vec![("case-1".to_string(), true), ("case-2".to_string(), false)];
    let digest = sealed_label_digest(&labels)?;
    let predictions = vec![("case-1".to_string(), true), ("case-2".to_string(), true)];
    let metrics = evaluate_sealed_benchmark(&m, &digest, &predictions, &labels)?;
    assert_eq!(metrics.true_positive, 1);
    assert_eq!(metrics.false_positive, 1);
    assert_eq!(metrics.recall, Some(1.0));
    Ok(())
}
