//! Living-review diff and digest edge cases.

use searchright_contracts::{
    BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, BibliographicRecord, RecordIdentifiers, RecordKind,
};
use searchright_living::{LivingError, diff_record_sets, record_digest};

fn record(id: &str, title: &str, year: Option<i32>) -> BibliographicRecord {
    BibliographicRecord {
        schema_version: BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_string(),
        record_id: id.to_string(),
        source_receipt_id: "receipt-1".to_string(),
        native_id: id.to_string(),
        kind: RecordKind::JournalArticle,
        identifiers: RecordIdentifiers::default(),
        title: title.to_string(),
        abstract_text: None,
        authors: vec![],
        container_title: None,
        publication_year: year,
        publication_date: None,
        languages: vec![],
        subjects: vec![],
        urls: vec![],
        provider_metadata: serde_json::Value::Null,
    }
}

#[test]
fn digest_changes_with_publication_year() {
    let first = record_digest(&record("r1", "Title", Some(2024)))
        .unwrap_or_else(|_| panic!("expected digest"));
    let second = record_digest(&record("r1", "Title", Some(2025)))
        .unwrap_or_else(|_| panic!("expected digest"));
    assert_ne!(first, second);
    assert_eq!(first.len(), 64);
}

#[test]
fn diff_empty_to_empty_has_no_changes() {
    let changes = diff_record_sets(&[], &[]).unwrap_or_else(|_| panic!("expected diff"));
    assert!(changes.is_empty());
}

#[test]
fn diff_empty_to_populated_reports_all_added() {
    let current = vec![record("r1", "Title", None)];
    let changes = diff_record_sets(&[], &current).unwrap_or_else(|_| panic!("expected diff"));
    assert_eq!(changes.len(), 1);
    let first = changes.first().unwrap_or_else(|| panic!("expected change"));
    assert!(first.before_digest.is_none());
    assert!(first.after_digest.is_some());
    assert!(first.note.contains("not present in the parent run"));
}

#[test]
fn diff_populated_to_empty_reports_all_missing() {
    let previous = vec![record("r1", "Title", None)];
    let changes = diff_record_sets(&previous, &[]).unwrap_or_else(|_| panic!("expected diff"));
    assert_eq!(changes.len(), 1);
    let first = changes.first().unwrap_or_else(|| panic!("expected change"));
    assert!(first.before_digest.is_some());
    assert!(first.after_digest.is_none());
}

#[test]
fn diff_identical_sets_has_no_changes() {
    let set = vec![record("r1", "Title", None), record("r2", "Other", None)];
    let changes = diff_record_sets(&set, &set).unwrap_or_else(|_| panic!("expected diff"));
    assert!(changes.is_empty());
}

#[test]
fn diff_duplicate_in_current_is_rejected() {
    let rec = record("r1", "Title", None);
    assert!(matches!(
        diff_record_sets(&[], &[rec.clone(), rec]),
        Err(LivingError::DuplicateRecord(_))
    ));
}

#[test]
fn diff_output_is_sorted_by_record_id() {
    let previous = vec![];
    let current = vec![record("b", "Title B", None), record("a", "Title A", None)];
    let changes = diff_record_sets(&previous, &current).unwrap_or_else(|_| panic!("expected diff"));
    assert_eq!(changes.len(), 2);
    let first = changes.first().unwrap_or_else(|| panic!("expected change"));
    assert_eq!(first.record_id, "a");
}
