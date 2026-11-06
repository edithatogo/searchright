//! Tokenisation, ranking tie-breaks and calibration edge cases.

use std::collections::BTreeSet;

use searchright_contracts::{
    BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, BibliographicRecord, RankingFeature, RankingScore,
    RecordIdentifiers, RecordKind,
};
use searchright_ranking::{
    LexicalRanker, RankingError, calibrate, calibration_counts, validate_calibration,
};

fn record(
    id: &str,
    title: &str,
    abstract_text: Option<&str>,
    subjects: Vec<&str>,
) -> BibliographicRecord {
    BibliographicRecord {
        schema_version: BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_string(),
        record_id: id.to_string(),
        source_receipt_id: "receipt-1".to_string(),
        native_id: format!("native-{id}"),
        kind: RecordKind::JournalArticle,
        identifiers: RecordIdentifiers::default(),
        title: title.to_string(),
        abstract_text: abstract_text.map(str::to_string),
        authors: vec![],
        container_title: None,
        publication_year: None,
        publication_date: None,
        languages: vec![],
        subjects: subjects.iter().map(ToString::to_string).collect(),
        urls: vec![],
        provider_metadata: serde_json::Value::Null,
    }
}

fn score(id: &str, value: f64) -> RankingScore {
    RankingScore {
        record_id: id.to_string(),
        score: value,
        features: vec![RankingFeature {
            name: "x".to_string(),
            value,
            weight: 1.0,
            rationale: "synthetic".to_string(),
        }],
        ranker_version: "test".to_string(),
        human_reviewed: false,
    }
}

#[test]
fn tokenisation_is_case_and_punctuation_insensitive() {
    let ranker = LexicalRanker::default();
    let records = vec![record("r1", "Machine, LEARNING!", None, vec![])];
    let scores = ranker
        .score(&records, &["MACHINE".to_string(), "learning".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    assert_eq!(first.features.len(), 3);
    // title overlap should be 1.0
    let title_feature = first
        .features
        .iter()
        .find(|f| f.name == "title_token_overlap")
        .unwrap_or_else(|| panic!("expected title feature"));
    assert!((title_feature.value - 1.0).abs() < 1e-12);
}

#[test]
fn single_character_tokens_are_ignored() {
    let ranker = LexicalRanker::default();
    let records = vec![record("r1", "a b cancer", None, vec![])];
    let scores = ranker
        .score(&records, &["a".to_string(), "cancer".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    let title_feature = first
        .features
        .iter()
        .find(|f| f.name == "title_token_overlap")
        .unwrap_or_else(|| panic!("expected title feature"));
    // query reduces to {cancer}; title contains cancer => overlap 1.0
    assert!((title_feature.value - 1.0).abs() < 1e-12);
}

#[test]
fn empty_records_returns_empty_scores() {
    let ranker = LexicalRanker::default();
    let scores = ranker
        .score(&[], &["cancer".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    assert!(scores.is_empty());
}

#[test]
#[allow(clippy::float_cmp, reason = "exact zero abstract overlap is analytic")]
fn none_abstract_scores_zero_for_abstract_feature() {
    let ranker = LexicalRanker::default();
    let records = vec![record("r1", "cancer", None, vec![])];
    let scores = ranker
        .score(&records, &["cancer".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    let abstract_feature = first
        .features
        .iter()
        .find(|f| f.name == "abstract_token_overlap")
        .unwrap_or_else(|| panic!("expected abstract feature"));
    assert_eq!(abstract_feature.value, 0.0);
}

#[test]
fn equal_scores_tie_break_by_record_id() {
    let ranker = LexicalRanker::default();
    let records = vec![
        record("b-record", "cancer", None, vec![]),
        record("a-record", "cancer", None, vec![]),
    ];
    let scores = ranker
        .score(&records, &["cancer".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    assert_eq!(scores.len(), 2);
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    assert_eq!(first.record_id, "a-record");
}

#[test]
fn subject_only_match_is_counted() {
    let ranker = LexicalRanker::default();
    let records = vec![record("r1", "unrelated title", None, vec!["cancer"])];
    let scores = ranker
        .score(&records, &["cancer".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    let subject_feature = first
        .features
        .iter()
        .find(|f| f.name == "subject_token_overlap")
        .unwrap_or_else(|| panic!("expected subject feature"));
    assert!((subject_feature.value - 1.0).abs() < 1e-12);
    assert!(first.score > 0.0);
}

#[test]
fn calibration_counts_threshold_is_inclusive() {
    let scores = vec![score("r1", 0.5), score("r2", 0.5), score("r3", 0.4)];
    let mut relevant = BTreeSet::new();
    relevant.insert("r1".to_string());
    relevant.insert("r3".to_string());
    let counts = calibration_counts(&scores, &relevant, 0.5);
    // r1 TP (>= threshold), r2 FP, r3 TN (relevant below threshold is FN)
    assert_eq!(counts.true_positive, 1);
    assert_eq!(counts.false_positive, 1);
    assert_eq!(counts.true_negative, 0);
    assert_eq!(counts.false_negative, 1);
}

#[test]
fn calibrate_default_thresholds_when_empty() {
    let scores = vec![score("r1", 0.5)];
    let relevant = BTreeSet::new();
    let curve = calibrate(&scores, &relevant, &[]);
    assert_eq!(curve.points.len(), 9);
    assert_eq!(curve.total_records, 1);
}

#[test]
fn calibrate_counts_total_relevant_only_from_scores() {
    let scores = vec![score("r1", 0.9), score("r2", 0.1)];
    let mut relevant = BTreeSet::new();
    relevant.insert("r1".to_string());
    relevant.insert("ghost".to_string());
    let curve = calibrate(&scores, &relevant, &[0.5]);
    assert_eq!(curve.total_relevant, 1);
}

#[test]
fn to_ranking_calibration_missing_threshold_fails() {
    let scores = vec![score("r1", 0.9)];
    let mut relevant = BTreeSet::new();
    relevant.insert("r1".to_string());
    let curve = calibrate(&scores, &relevant, &[0.5]);
    assert!(matches!(
        curve.to_ranking_calibration("rev", "ranker", 0.3, 0.5, Some("approver")),
        Err(RankingError::Contract(_))
    ));
}

#[test]
fn negative_weights_fail_with_invalid_weights() {
    let ranker = LexicalRanker {
        version: "neg".to_string(),
        title_weight: -2.0,
        abstract_weight: 1.0,
        subject_weight: 1.0,
    };
    let records = vec![record("r1", "cancer", None, vec![])];
    assert!(matches!(
        ranker.score(&records, &["cancer".to_string()]),
        Err(RankingError::InvalidWeights)
    ));
}

#[test]
fn infinite_weights_fail_with_invalid_weights() {
    let ranker = LexicalRanker {
        version: "inf".to_string(),
        title_weight: f64::INFINITY,
        abstract_weight: 0.1,
        subject_weight: 0.1,
    };
    let records = vec![record("r1", "cancer", None, vec![])];
    assert!(matches!(
        ranker.score(&records, &["cancer".to_string()]),
        Err(RankingError::InvalidWeights)
    ));
}

#[test]
fn score_features_are_all_present_and_bounded() {
    let ranker = LexicalRanker::default();
    let records = vec![record(
        "r1",
        "cancer screening",
        Some("cancer in abstracts"),
        vec!["cancer"],
    )];
    let scores = ranker
        .score(&records, &["cancer".to_string(), "screening".to_string()])
        .unwrap_or_else(|_| panic!("expected scoring"));
    let first = scores.first().unwrap_or_else(|| panic!("expected score"));
    assert_eq!(first.features.len(), 3);
    for feature in &first.features {
        assert!((0.0..=1.0).contains(&feature.value));
    }
}

#[test]
fn validate_calibration_rejects_tampered_auto_exclusion() {
    let scores = vec![score("r1", 0.9)];
    let mut relevant = BTreeSet::new();
    relevant.insert("r1".to_string());
    let curve = calibrate(&scores, &relevant, &[0.5]);
    let mut calibration = curve
        .to_ranking_calibration("rev-1", "ranker", 0.5, 0.5, Some("approver"))
        .unwrap_or_else(|_| panic!("expected calibration"));
    calibration.auto_exclusion_prohibited = false;
    assert!(validate_calibration(&calibration).is_err());
}
