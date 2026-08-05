//! Single-writer append-only storage with same-filesystem, replace-style snapshots.

#![forbid(unsafe_code)]

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use evidence_search_core::{AuditLedger, AuditVerification};
use searchright_contracts::AuditEvent;
use serde::Serialize;

/// Filesystem-backed review store.
#[derive(Debug, Clone)]
pub struct FileReviewStore {
    root: PathBuf,
}

impl FileReviewStore {
    /// Open or create a review store directory.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = path.into();
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join("snapshots"))?;
        Ok(Self { root })
    }

    /// Root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Append one audit event after verifying the persisted head and candidate event.
    ///
    /// This method is safe against accidental chain divergence in a single writer.
    /// Multi-process writers require the file-locking track before they are supported.
    pub fn append_event(&self, event: &AuditEvent) -> Result<(), StoreError> {
        let mut events = self.read_events()?;
        AuditLedger::from_events(events.clone()).verify()?;
        if events.iter().any(|existing| existing.event_id == event.event_id) {
            return Err(StoreError::DuplicateEventId(event.event_id.clone()));
        }
        if let Some(first) = events.first()
            && first.review_id != event.review_id
        {
            return Err(StoreError::ReviewMismatch {
                expected: first.review_id.clone(),
                actual: event.review_id.clone(),
            });
        }
        let expected_previous = events
            .last()
            .map_or("GENESIS", |existing| existing.event_hash.as_str());
        if event.previous_hash != expected_previous {
            return Err(StoreError::PreviousHashMismatch {
                expected: expected_previous.to_owned(),
                actual: event.previous_hash.clone(),
            });
        }
        events.push(event.clone());
        AuditLedger::from_events(events).verify()?;

        let path = self.root.join("audit.jsonl");
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, event)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        Ok(())
    }

    /// Read all audit events in order.
    pub fn read_events(&self) -> Result<Vec<AuditEvent>, StoreError> {
        let path = self.root.join("audit.jsonl");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let reader = BufReader::new(File::open(path)?);
        let mut events = Vec::new();
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let event = serde_json::from_str(&line).map_err(|source| StoreError::MalformedLine {
                line: line_number + 1,
                source,
            })?;
            events.push(event);
        }
        Ok(events)
    }

    /// Verify persisted audit events.
    pub fn verify_audit(&self) -> Result<AuditVerification, StoreError> {
        let events = self.read_events()?;
        Ok(AuditLedger::from_events(events).verify()?)
    }

    /// Replace a derived JSON snapshot using a same-directory temporary file and rename.
    ///
    /// This is not a cross-filesystem transaction. Directory durability and replacement
    /// semantics remain platform-specific until the crash-consistency hardening track lands.
    pub fn write_snapshot<T: Serialize>(&self, name: &str, value: &T) -> Result<PathBuf, StoreError> {
        validate_snapshot_name(name)?;
        let target = self.root.join("snapshots").join(format!("{name}.json"));
        let temporary = self
            .root
            .join("snapshots")
            .join(format!(".{name}.{}.tmp", uuid::Uuid::now_v7()));
        {
            let file = File::create(&temporary)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, value)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        fs::rename(&temporary, &target)?;
        Ok(target)
    }
}

/// Storage error.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Filesystem error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// One JSONL line was malformed.
    #[error("audit JSONL line {line} is malformed: {source}")]
    MalformedLine {
        line: usize,
        source: serde_json::Error,
    },
    /// Audit verification failed.
    #[error(transparent)]
    Audit(#[from] evidence_search_core::AuditError),
    /// Snapshot name could escape its directory.
    #[error("invalid snapshot name")]
    InvalidSnapshotName,
    /// Event identifier was already present in the ledger.
    #[error("audit event identifier `{0}` is already present")]
    DuplicateEventId(String),
    /// A review store was used for a different review identifier.
    #[error("review identifier mismatch: expected `{expected}`, found `{actual}`")]
    ReviewMismatch { expected: String, actual: String },
    /// Candidate event did not point to the current persisted head.
    #[error("audit previous-hash mismatch: expected `{expected}`, found `{actual}`")]
    PreviousHashMismatch { expected: String, actual: String },
}

fn validate_snapshot_name(name: &str) -> Result<(), StoreError> {
    if name.is_empty()
        || name.starts_with('.')
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
    {
        Err(StoreError::InvalidSnapshotName)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use evidence_search_core::AuditLedger;
    use searchright_contracts::{Actor, AuditEventDraft};
    use serde_json::json;

    use super::*;

    #[test]
    fn persisted_ledger_round_trips() {
        let directory = std::env::temp_dir().join(format!("searchright-test-{}", uuid::Uuid::now_v7()));
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let mut ledger = AuditLedger::new();
            let appended = ledger.append(AuditEventDraft {
                schema_version: "org.searchright.audit-event.v1".to_owned(),
                event_id: "event-1".to_owned(),
                review_id: "review-1".to_owned(),
                event_type: "created".to_owned(),
                occurred_at: "2026-08-05T00:00:00Z".to_owned(),
                actor: Actor { actor_id: "test".to_owned(), actor_type: "human".to_owned(), provenance: None },
                payload: json!({"ok": true}),
            });
            assert!(appended.is_ok());
            if let Ok(event) = appended {
                assert!(store.append_event(event).is_ok());
            }
            assert!(store.verify_audit().is_ok());
        }
        let _cleanup = fs::remove_dir_all(directory);
    }
}
