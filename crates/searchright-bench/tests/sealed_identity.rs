//! Population and label-commitment regressions using synthetic public examples only.

use searchright_bench::{
    BenchmarkError, SealedSplitManifest, evaluate_sealed_benchmark, sealed_label_digest,
};

fn manifest() -> SealedSplitManifest {
    SealedSplitManifest {
        schema_version: "org.searchright.sealed-benchmark-manifest.v1".to_owned(),
        partition: "sealed_test".to_owned(),
        case_ids: vec!["r1".to_owned(), "r2".to_owned(), "r3".to_owned()],
        labels_present: false,
        labels_location: "synthetic test only".to_owned(),
        label_digest: None,
        release_condition: "no real sealed labels are used".to_owned(),
        claim_boundary: "population integrity only; no external performance evidence".to_owned(),
    }
}

fn labels() -> Vec<(String, bool)> {
    vec![
        ("r1".to_owned(), true),
        ("r2".to_owned(), false),
        ("r3".to_owned(), true),
    ]
}

#[test]
fn canonical_digest_matches_independent_sha256_vectors() -> Result<(), BenchmarkError> {
    // Vectors calculated independently with Python hashlib from the documented bytes.
    assert_eq!(
        sealed_label_digest(&labels())?,
        "040b66081df5355f6f01dd418b8dd96a0b24df4479bbec4d6f57bc7d184adea6"
    );
    // Embedded NUL and a two-byte UTF-8 identifier exercise framing and byte lengths.
    assert_eq!(
        sealed_label_digest(&[
            ("a\0b".to_owned(), true),
            ("a".to_owned(), false),
            ("β".to_owned(), true),
        ])?,
        "c631b1c827b1134a10d54e7e359585b8e0716ab7ae586f2e425afe661051a460"
    );
    Ok(())
}

#[test]
fn digest_rejects_empty_blank_and_duplicate_labels() {
    assert!(matches!(
        sealed_label_digest(&[]),
        Err(BenchmarkError::EmptyLabels)
    ));
    assert!(matches!(
        sealed_label_digest(&[(" \t".to_owned(), true)]),
        Err(BenchmarkError::EmptyLabelRecordId)
    ));
    for duplicate_value in [false, true] {
        assert!(matches!(
            sealed_label_digest(&[
                ("r1".to_owned(), true),
                ("r1".to_owned(), duplicate_value),
            ]),
            Err(BenchmarkError::DuplicateLabelRecord(identifier)) if identifier == "r1"
        ));
    }
}

#[test]
fn complete_evaluation_is_invariant_to_input_order() -> Result<(), BenchmarkError> {
    let mut population = manifest();
    let mut actual_labels = labels();
    let mut predictions = vec![
        ("r1".to_owned(), true),
        ("r2".to_owned(), true),
        ("r3".to_owned(), false),
    ];
    let digest = sealed_label_digest(&actual_labels)?;
    let expected = evaluate_sealed_benchmark(&population, &digest, &predictions, &actual_labels)?;
    assert_eq!(expected.true_positive, 1);
    assert_eq!(expected.false_positive, 1);
    assert_eq!(expected.false_negative, 1);

    population.label_digest = Some(digest.clone());
    population.case_ids.reverse();
    actual_labels.rotate_left(1);
    predictions.reverse();
    assert_eq!(sealed_label_digest(&actual_labels)?, digest);
    assert_eq!(
        evaluate_sealed_benchmark(&population, &digest, &predictions, &actual_labels)?,
        expected
    );
    Ok(())
}

#[test]
fn evaluation_rejects_duplicate_labels_even_when_values_agree() -> Result<(), BenchmarkError> {
    let original = labels();
    let digest = sealed_label_digest(&original)?;
    for duplicate_value in [false, true] {
        let mut duplicated = original.clone();
        duplicated.push(("r1".to_owned(), duplicate_value));
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &digest, &original, &duplicated),
            Err(BenchmarkError::DuplicateLabelRecord(identifier)) if identifier == "r1"
        ));
    }
    Ok(())
}

#[test]
fn evaluation_rejects_a_different_label_population() -> Result<(), BenchmarkError> {
    let mut substituted = labels();
    substituted.retain(|(identifier, _)| identifier != "r2");
    substituted.push(("outside".to_owned(), false));
    let digest = sealed_label_digest(&substituted)?;
    assert!(matches!(
        evaluate_sealed_benchmark(&manifest(), &digest, &substituted, &substituted),
        Err(BenchmarkError::UnknownLabelRecord(identifier)) if identifier == "outside"
    ));
    Ok(())
}

#[test]
fn evaluation_rejects_missing_positive_or_negative_labels() -> Result<(), BenchmarkError> {
    for missing in ["r1", "r2"] {
        let mut incomplete = labels();
        incomplete.retain(|(identifier, _)| identifier != missing);
        let digest = sealed_label_digest(&incomplete)?;
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &digest, &labels(), &incomplete),
            Err(BenchmarkError::MissingLabelRecord(identifier)) if identifier == missing
        ));
    }
    Ok(())
}

#[test]
fn evaluation_rejects_duplicate_predictions_even_when_values_agree() -> Result<(), BenchmarkError> {
    let actual_labels = labels();
    let digest = sealed_label_digest(&actual_labels)?;
    for duplicate_value in [false, true] {
        let mut duplicated = actual_labels.clone();
        duplicated.push(("r2".to_owned(), duplicate_value));
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &digest, &duplicated, &actual_labels),
            Err(BenchmarkError::DuplicatePredictionRecord(identifier)) if identifier == "r2"
        ));
    }
    Ok(())
}

#[test]
fn evaluation_rejects_unknown_positive_and_negative_predictions() -> Result<(), BenchmarkError> {
    let actual_labels = labels();
    let digest = sealed_label_digest(&actual_labels)?;
    for prediction in [false, true] {
        let mut extra = actual_labels.clone();
        extra.push(("outside".to_owned(), prediction));
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &digest, &extra, &actual_labels),
            Err(BenchmarkError::UnknownPredictionRecord(identifier)) if identifier == "outside"
        ));
    }
    Ok(())
}

#[test]
fn evaluation_rejects_implicit_abstention_even_for_a_negative_case() -> Result<(), BenchmarkError> {
    let actual_labels = labels();
    let digest = sealed_label_digest(&actual_labels)?;
    for missing in ["r1", "r2"] {
        let mut incomplete = actual_labels.clone();
        incomplete.retain(|(identifier, _)| identifier != missing);
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &digest, &incomplete, &actual_labels),
            Err(BenchmarkError::MissingPredictionRecord(identifier)) if identifier == missing
        ));
    }
    assert!(matches!(
        evaluate_sealed_benchmark(&manifest(), &digest, &[], &actual_labels),
        Err(BenchmarkError::MissingPredictionRecord(_))
    ));
    Ok(())
}

#[test]
fn evaluation_rejects_label_tampering_under_the_original_digest() -> Result<(), BenchmarkError> {
    let actual_labels = labels();
    let digest = sealed_label_digest(&actual_labels)?;
    let mut changed = actual_labels.clone();
    for (identifier, positive) in &mut changed {
        if identifier == "r2" {
            *positive = true;
        }
    }
    assert!(matches!(
        evaluate_sealed_benchmark(&manifest(), &digest, &actual_labels, &changed),
        Err(BenchmarkError::LabelDigestMismatch)
    ));
    Ok(())
}

#[test]
fn evaluation_checks_supplied_and_manifest_digest_commitments() -> Result<(), BenchmarkError> {
    let actual_labels = labels();
    let digest = sealed_label_digest(&actual_labels)?;
    assert!(matches!(
        evaluate_sealed_benchmark(&manifest(), &"0".repeat(64), &actual_labels, &actual_labels),
        Err(BenchmarkError::LabelDigestMismatch)
    ));
    let mut population = manifest();
    population.label_digest = Some("0".repeat(64));
    assert!(matches!(
        evaluate_sealed_benchmark(&population, &digest, &actual_labels, &actual_labels),
        Err(BenchmarkError::ManifestLabelDigestMismatch)
    ));
    for malformed in ["short".to_owned(), digest.to_uppercase()] {
        assert!(matches!(
            evaluate_sealed_benchmark(&manifest(), &malformed, &actual_labels, &actual_labels),
            Err(BenchmarkError::InvalidLabelDigest(_))
        ));
        population.label_digest = Some(malformed);
        assert!(matches!(
            population.validate(),
            Err(BenchmarkError::InvalidLabelDigest(_))
        ));
    }
    Ok(())
}
