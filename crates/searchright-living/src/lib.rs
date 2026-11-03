//! Living-review lineage, digests and deterministic change detection.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

pub use searchright_contracts::LivingUpdateRun;
use searchright_contracts::{
    BibliographicRecord, RecordChange, RecordChangeKind, UpdateRunStatus, Validate,
};

/// Create a stable BLAKE3 digest of one canonical bibliographic record.
pub fn record_digest(record: &BibliographicRecord) -> Result<String, LivingError> {
    let bytes = serde_json::to_vec(record)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Compare two canonical record sets by stable record identifier.
pub fn diff_records(
    previous: &[BibliographicRecord],
    current: &[BibliographicRecord],
) -> Result<Vec<RecordChange>, LivingError> {
    let previous_map = record_map(previous)?;
    let current_map = record_map(current)?;
    let mut identifiers: BTreeSet<&str> = previous_map.keys().map(String::as_str).collect();
    identifiers.extend(current_map.keys().map(String::as_str));

    let mut changes = Vec::new();
    for identifier in identifiers {
        match (previous_map.get(identifier), current_map.get(identifier)) {
            (None, Some(after)) => changes.push(RecordChange {
                record_id: identifier.to_owned(),
                kind: RecordChangeKind::Added,
                before_digest: None,
                after_digest: Some(record_digest(after)?),
                note: "record was not present in the parent run".to_owned(),
            }),
            (Some(before), None) => changes.push(RecordChange {
                record_id: identifier.to_owned(),
                kind: RecordChangeKind::MissingFromSource,
                before_digest: Some(record_digest(before)?),
                after_digest: None,
                note: "record was present in the parent run but not returned now".to_owned(),
            }),
            (Some(before), Some(after)) => {
                let before_digest = record_digest(before)?;
                let after_digest = record_digest(after)?;
                if before_digest != after_digest {
                    changes.push(RecordChange {
                        record_id: identifier.to_owned(),
                        kind: RecordChangeKind::Updated,
                        before_digest: Some(before_digest),
                        after_digest: Some(after_digest),
                        note: "canonical record content changed".to_owned(),
                    });
                }
            }
            (None, None) => {}
        }
    }
    Ok(changes)
}

/// Compare two canonical record sets by stable record identifier (alias for `diff_records`).
pub use diff_records as diff_record_sets;

/// Validate a set of immutable update runs as one acyclic lineage.
pub fn validate_lineage(runs: &[LivingUpdateRun]) -> Result<(), LivingError> {
    let mut by_id = BTreeMap::new();
    for run in runs {
        run.validate()?;
        if by_id.insert(run.run_id.as_str(), run).is_some() {
            return Err(LivingError::DuplicateRun(run.run_id.clone()));
        }
    }
    for run in runs {
        if let Some(parent) = run.parent_run_id.as_deref() {
            let parent_run = by_id
                .get(parent)
                .ok_or_else(|| LivingError::UnknownParent {
                    run_id: run.run_id.clone(),
                    parent_id: parent.to_owned(),
                })?;
            if parent_run.review_id != run.review_id {
                return Err(LivingError::CrossReviewLineage {
                    run_id: run.run_id.clone(),
                    related_id: parent.to_owned(),
                });
            }
        }
        if run.status == UpdateRunStatus::Completed
            && let Some(superseded) = run.supersedes_run_id.as_deref()
        {
            let superseded_run =
                by_id
                    .get(superseded)
                    .ok_or_else(|| LivingError::UnknownSuperseded {
                        run_id: run.run_id.clone(),
                        superseded_id: superseded.to_owned(),
                    })?;
            if superseded_run.review_id != run.review_id {
                return Err(LivingError::CrossReviewLineage {
                    run_id: run.run_id.clone(),
                    related_id: superseded.to_owned(),
                });
            }
            if !is_ancestor(superseded, run, &by_id) {
                return Err(LivingError::InvalidSupersession {
                    run_id: run.run_id.clone(),
                    superseded_id: superseded.to_owned(),
                });
            }
        }
        detect_cycle(run, &by_id)?;
    }
    Ok(())
}

fn is_ancestor(
    candidate: &str,
    run: &LivingUpdateRun,
    runs: &BTreeMap<&str, &LivingUpdateRun>,
) -> bool {
    let mut seen = BTreeSet::new();
    let mut current = run.parent_run_id.as_deref();
    while let Some(run_id) = current {
        if !seen.insert(run_id) {
            return false;
        }
        if run_id == candidate {
            return true;
        }
        current = runs
            .get(run_id)
            .and_then(|parent| parent.parent_run_id.as_deref());
    }
    false
}

fn record_map(
    records: &[BibliographicRecord],
) -> Result<BTreeMap<String, &BibliographicRecord>, LivingError> {
    let mut map = BTreeMap::new();
    for record in records {
        if map.insert(record.record_id.clone(), record).is_some() {
            return Err(LivingError::DuplicateRecord(record.record_id.clone()));
        }
    }
    Ok(map)
}

fn detect_cycle(
    start: &LivingUpdateRun,
    runs: &BTreeMap<&str, &LivingUpdateRun>,
) -> Result<(), LivingError> {
    let mut seen = BTreeSet::new();
    let mut current = Some(start.run_id.as_str());
    while let Some(run_id) = current {
        if !seen.insert(run_id) {
            return Err(LivingError::LineageCycle(start.run_id.clone()));
        }
        current = runs
            .get(run_id)
            .and_then(|run| run.parent_run_id.as_deref());
    }
    Ok(())
}

/// Living-review operation error.
#[derive(Debug, thiserror::Error)]
pub enum LivingError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// JSON serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Duplicate record identifier.
    #[error("record identifier `{0}` is duplicated")]
    DuplicateRecord(String),
    /// Duplicate run identifier.
    #[error("living-update run identifier `{0}` is duplicated")]
    DuplicateRun(String),
    /// Parent run was not present.
    #[error("run `{run_id}` references unknown parent `{parent_id}`")]
    UnknownParent {
        /// Identifier of the run containing the invalid parent reference.
        run_id: String,
        /// Referenced parent identifier that was not found.
        parent_id: String,
    },
    /// Superseded run was not present.
    #[error("run `{run_id}` references unknown superseded run `{superseded_id}`")]
    UnknownSuperseded {
        /// Identifier of the run containing the invalid supersession reference.
        run_id: String,
        /// Referenced superseded-run identifier that was not found.
        superseded_id: String,
    },
    /// A parent or superseded run belongs to another review.
    #[error("run `{run_id}` links to run `{related_id}` from another review")]
    CrossReviewLineage {
        /// Identifier of the run containing the invalid link.
        run_id: String,
        /// Identifier of the cross-review run.
        related_id: String,
    },
    /// A run attempted to supersede a run outside its ancestry.
    #[error("run `{run_id}` cannot supersede non-ancestor run `{superseded_id}`")]
    InvalidSupersession {
        /// Identifier of the superseding run.
        run_id: String,
        /// Identifier that is not an ancestor.
        superseded_id: String,
    },
    /// Parent links formed a cycle.
    #[error("living-update lineage contains a cycle at `{0}`")]
    LineageCycle(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use searchright_contracts::{
        BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, LIVING_UPDATE_SCHEMA_VERSION, RecordIdentifiers,
        RecordKind,
    };

    fn test_record(id: &str, title: &str) -> BibliographicRecord {
        BibliographicRecord {
            schema_version: BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: id.to_owned(),
            source_receipt_id: "receipt-1".to_owned(),
            native_id: id.to_owned(),
            kind: RecordKind::JournalArticle,
            identifiers: RecordIdentifiers::default(),
            title: title.to_owned(),
            abstract_text: None,
            authors: vec!["Author, A.".to_owned()],
            container_title: None,
            publication_year: Some(2026),
            publication_date: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            urls: Vec::new(),
            provider_metadata: serde_json::Value::Null,
        }
    }

    fn run(review: &str, id: &str, parent: Option<&str>) -> LivingUpdateRun {
        LivingUpdateRun {
            schema_version: LIVING_UPDATE_SCHEMA_VERSION.to_owned(),
            review_id: review.to_owned(),
            run_id: id.to_owned(),
            parent_run_id: parent.map(str::to_owned),
            status: UpdateRunStatus::Completed,
            started_at: "2026-08-01T00:00:00Z".to_owned(),
            completed_at: Some("2026-08-01T01:00:00Z".to_owned()),
            protocol_version: "protocol-v1".to_owned(),
            cursors_before: Vec::new(),
            cursors_after: Vec::new(),
            changes: Vec::new(),
            requires_human_release: true,
            supersedes_run_id: None,
        }
    }

    #[test]
    fn record_digest_produces_deterministic_blake3_digest() {
        let rec1 = test_record("rec-1", "Title A");
        let rec2 = test_record("rec-1", "Title A");
        let rec3 = test_record("rec-1", "Title B");

        let Ok(digest1) = record_digest(&rec1) else {
            panic!("record_digest must succeed for rec1");
        };
        let Ok(digest2) = record_digest(&rec2) else {
            panic!("record_digest must succeed for rec2");
        };
        let Ok(digest3) = record_digest(&rec3) else {
            panic!("record_digest must succeed for rec3");
        };

        assert_eq!(digest1, digest2);
        assert_eq!(digest1.len(), 64);
        assert_ne!(digest1, digest3);
    }

    #[test]
    fn diff_record_sets_detects_added_modified_missing_and_unchanged() {
        let r1 = test_record("rec-1", "Unchanged Title");
        let r2_before = test_record("rec-2", "Original Title");
        let r2_after = test_record("rec-2", "Updated Title");
        let r3_missing = test_record("rec-3", "Dropped Record");
        let r4_added = test_record("rec-4", "Newly Discovered Record");

        let previous = vec![r1.clone(), r2_before, r3_missing];
        let current = vec![r1, r2_after, r4_added];

        let Ok(changes) = diff_record_sets(&previous, &current) else {
            panic!("diff_record_sets must succeed");
        };

        assert_eq!(changes.len(), 3);

        let c2 = changes
            .iter()
            .find(|c| c.record_id == "rec-2")
            .unwrap_or_else(|| panic!("rec-2 change must be present"));
        assert_eq!(c2.kind, RecordChangeKind::Updated);
        assert!(c2.before_digest.is_some());
        assert!(c2.after_digest.is_some());
        assert_ne!(c2.before_digest, c2.after_digest);

        let c3 = changes
            .iter()
            .find(|c| c.record_id == "rec-3")
            .unwrap_or_else(|| panic!("rec-3 change must be present"));
        assert_eq!(c3.kind, RecordChangeKind::MissingFromSource);
        assert!(c3.before_digest.is_some());
        assert!(c3.after_digest.is_none());

        let c4 = changes
            .iter()
            .find(|c| c.record_id == "rec-4")
            .unwrap_or_else(|| panic!("rec-4 change must be present"));
        assert_eq!(c4.kind, RecordChangeKind::Added);
        assert!(c4.before_digest.is_none());
        assert!(c4.after_digest.is_some());
    }

    #[test]
    fn diff_record_sets_rejects_duplicate_records() {
        let rec = test_record("rec-dup", "Duplicate Title");
        let previous = vec![rec.clone(), rec];
        let current = vec![];

        assert!(matches!(
            diff_record_sets(&previous, &current),
            Err(LivingError::DuplicateRecord(id)) if id == "rec-dup"
        ));
    }

    #[test]
    fn validate_lineage_detects_cycles() {
        let first = run("review-1", "run-1", Some("run-2"));
        let second = run("review-1", "run-2", Some("run-1"));

        assert!(matches!(
            validate_lineage(&[first, second]),
            Err(LivingError::LineageCycle(id)) if id == "run-1" || id == "run-2"
        ));
    }

    #[test]
    fn validate_lineage_detects_duplicate_runs() {
        let first = run("review-1", "run-1", None);
        let duplicate = run("review-1", "run-1", None);

        assert!(matches!(
            validate_lineage(&[first, duplicate]),
            Err(LivingError::DuplicateRun(id)) if id == "run-1"
        ));
    }

    #[test]
    fn validate_lineage_detects_unknown_parent() {
        let run_with_missing_parent = run("review-1", "run-2", Some("nonexistent-parent"));

        assert!(matches!(
            validate_lineage(&[run_with_missing_parent]),
            Err(LivingError::UnknownParent { run_id, parent_id }) if run_id == "run-2" && parent_id == "nonexistent-parent"
        ));
    }

    #[test]
    fn validate_lineage_detects_unknown_superseded() {
        let mut completed = run("review-1", "run-1", None);
        completed.supersedes_run_id = Some("nonexistent-superseded".to_owned());

        assert!(matches!(
            validate_lineage(&[completed]),
            Err(LivingError::UnknownSuperseded { run_id, superseded_id }) if run_id == "run-1" && superseded_id == "nonexistent-superseded"
        ));
    }

    #[test]
    fn accepts_multi_cycle_lineage_and_ancestor_supersession() {
        let first = run("review-1", "run-1", None);
        let second = run("review-1", "run-2", Some("run-1"));
        let mut third = run("review-1", "run-3", Some("run-2"));
        third.supersedes_run_id = Some("run-1".to_owned());

        assert!(validate_lineage(&[third, first, second]).is_ok());
    }

    #[test]
    fn rejects_cross_review_parent_links() {
        let first = run("review-1", "run-1", None);
        let second = run("review-2", "run-2", Some("run-1"));

        assert!(matches!(
            validate_lineage(&[first, second]),
            Err(LivingError::CrossReviewLineage { .. })
        ));
    }

    #[test]
    fn rejects_supersession_outside_run_ancestry() {
        let first = run("review-1", "run-1", None);
        let sibling = run("review-1", "run-2", Some("run-1"));
        let mut other_sibling = run("review-1", "run-3", Some("run-1"));
        other_sibling.supersedes_run_id = Some("run-2".to_owned());

        assert!(matches!(
            validate_lineage(&[first, sibling, other_sibling]),
            Err(LivingError::InvalidSupersession { .. })
        ));
    }
}
