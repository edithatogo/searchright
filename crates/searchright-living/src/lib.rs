//! Living-review lineage, digests and deterministic change detection.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use searchright_contracts::{
    BibliographicRecord, LivingUpdateRun, RecordChange, RecordChangeKind, UpdateRunStatus, Validate,
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
        if let Some(parent) = run.parent_run_id.as_deref()
            && !by_id.contains_key(parent)
        {
            return Err(LivingError::UnknownParent {
                run_id: run.run_id.clone(),
                parent_id: parent.to_owned(),
            });
        }
        if run.status == UpdateRunStatus::Completed
            && let Some(superseded) = run.supersedes_run_id.as_deref()
            && !by_id.contains_key(superseded)
        {
            return Err(LivingError::UnknownSuperseded {
                run_id: run.run_id.clone(),
                superseded_id: superseded.to_owned(),
            });
        }
        detect_cycle(run, &by_id)?;
    }
    Ok(())
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
    UnknownParent { run_id: String, parent_id: String },
    /// Superseded run was not present.
    #[error("run `{run_id}` references unknown superseded run `{superseded_id}`")]
    UnknownSuperseded {
        run_id: String,
        superseded_id: String,
    },
    /// Parent links formed a cycle.
    #[error("living-update lineage contains a cycle at `{0}`")]
    LineageCycle(String),
}
