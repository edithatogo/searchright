//! Single-writer append-only storage with same-filesystem, replace-style snapshots.

#![forbid(unsafe_code)]

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use evidence_search_core::{AuditLedger, AuditVerification, canonical_record_digest};
use searchright_contracts::{
    AuditEvent, BibliographicRecord, SourceReceipt, Validate, validate_registered_audit_event,
};
use serde::{Deserialize, Serialize};

/// Filesystem-backed review store.
#[derive(Debug, Clone)]
pub struct FileReviewStore {
    root: PathBuf,
}

/// Evidence emitted after an atomic derived-snapshot replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotReceipt {
    /// Snapshot name.
    pub name: String,
    /// Repository-relative or caller-visible path.
    pub path: PathBuf,
    /// BLAKE3 digest of the exact bytes written.
    pub digest: String,
    /// Number of bytes written.
    pub bytes: u64,
}

/// One immutable source-execution persistence boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCommit {
    /// Stable idempotency key for this commit.
    pub commit_id: String,
    /// Redacted source execution receipt.
    pub receipt: SourceReceipt,
    /// Normalised records introduced by the receipt.
    pub records: Vec<BibliographicRecord>,
    /// Audit event describing the commit.
    pub audit_event: AuditEvent,
}

struct ReviewLock {
    path: PathBuf,
}

struct TemporaryFile {
    path: PathBuf,
    committed: bool,
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if !self.committed {
            let _result = fs::remove_file(&self.path);
        }
    }
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _owner_result = fs::remove_file(self.path.join("owner"));
        let _directory_result = fs::remove_dir(&self.path);
    }
}

impl FileReviewStore {
    /// Open or create a review store directory.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = path.into();
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join("events"))?;
        fs::create_dir_all(root.join("commits"))?;
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
    /// Events are committed as immutable, ordered segment files. The segment is synced
    /// before an atomic hard-link commit to a previously absent destination, so a crash exposes
    /// either no new event or one complete event. A portable exclusive-create lock
    /// serialises cooperating processes.
    pub fn append_event(&self, event: &AuditEvent) -> Result<(), StoreError> {
        validate_registered_audit_event(event)
            .map_err(|error| StoreError::InvalidAuditPayload(error.to_string()))?;
        let _lock = self.acquire_write_lock("append-audit-event")?;
        let mut events = self.read_events()?;
        AuditLedger::from_events(events.clone()).verify()?;
        if let Some(existing) = events
            .iter()
            .find(|existing| existing.event_id == event.event_id)
        {
            return if existing == event {
                Ok(())
            } else {
                Err(StoreError::ConflictingEventId(event.event_id.clone()))
            };
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
        AuditLedger::from_events(events.clone()).verify()?;

        let events_dir = self.root.join("events");
        let target = events_dir.join(format!("{:020}-{}.json", events.len(), event.event_hash));
        let temporary_path = events_dir.join(format!(".{}.pending", uuid::Uuid::now_v7()));
        let mut temporary = TemporaryFile {
            path: temporary_path.clone(),
            committed: false,
        };
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, event)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        fs::hard_link(&temporary_path, &target)?;
        sync_directory(Some(&events_dir))?;
        fs::remove_file(&temporary_path)?;
        temporary.committed = true;
        Ok(())
    }

    /// Read all audit events in order.
    pub fn read_events(&self) -> Result<Vec<AuditEvent>, StoreError> {
        let legacy_path = self.root.join("audit.jsonl");
        let mut events = Vec::new();
        if legacy_path.exists() {
            read_jsonl_events(&legacy_path, &mut events)?;
        }

        let mut segments = Vec::new();
        for entry in fs::read_dir(self.root.join("events"))? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && !entry.file_name().to_string_lossy().starts_with('.')
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                segments.push(entry.path());
            }
        }
        segments.sort();
        for segment in segments {
            let event = read_segment_event(&segment)?;
            let expected_name = format!(
                "{:020}-{}.json",
                events.len().saturating_add(1),
                event.event_hash
            );
            if segment.file_name().and_then(|name| name.to_str()) != Some(expected_name.as_str()) {
                return Err(StoreError::UnexpectedSegment {
                    expected: expected_name,
                    actual: segment.file_name().map_or_else(
                        || segment.display().to_string(),
                        |name| name.to_string_lossy().into_owned(),
                    ),
                });
            }
            events.push(event);
        }
        Ok(events)
    }

    /// Verify persisted audit events.
    pub fn verify_audit(&self) -> Result<AuditVerification, StoreError> {
        let events = self.read_events()?;
        Ok(AuditLedger::from_events(events).verify()?)
    }

    /// Atomically persist receipt, records and audit event as one immutable bundle.
    pub fn append_execution_commit(&self, commit: &ExecutionCommit) -> Result<PathBuf, StoreError> {
        validate_execution_commit(commit)?;
        let _lock = self.acquire_write_lock("append-execution-commit")?;
        let directory = self.root.join("commits");
        let bytes = serde_json::to_vec(commit)?;
        let relative = PathBuf::from("commits").join(format!("{}.json", commit.commit_id));
        let target = self.root.join(&relative);
        if target.exists() {
            return if fs::read(&target)? == bytes {
                Ok(relative)
            } else {
                Err(StoreError::ConflictingCommitId(commit.commit_id.clone()))
            };
        }
        let temporary_path = directory.join(format!(".{}.pending", uuid::Uuid::now_v7()));
        let mut temporary = TemporaryFile {
            path: temporary_path.clone(),
            committed: false,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::hard_link(&temporary_path, &target)?;
        sync_directory(Some(&directory))?;
        fs::remove_file(&temporary_path)?;
        temporary.committed = true;
        Ok(relative)
    }

    /// Replace a derived JSON snapshot using a same-directory temporary file and rename.
    ///
    /// This is not a cross-filesystem transaction. Directory durability and replacement
    /// semantics remain platform-specific until the crash-consistency hardening track lands.
    pub fn write_snapshot<T: Serialize>(
        &self,
        name: &str,
        value: &T,
    ) -> Result<PathBuf, StoreError> {
        let receipt = self.write_snapshot_with_receipt(name, value)?;
        Ok(self.root.join(receipt.path))
    }

    /// Replace a derived snapshot and return a content-addressed receipt.
    pub fn write_snapshot_with_receipt<T: Serialize>(
        &self,
        name: &str,
        value: &T,
    ) -> Result<SnapshotReceipt, StoreError> {
        validate_snapshot_name(name)?;
        let _lock = self.acquire_write_lock("replace-snapshot")?;
        let snapshot_dir = self.root.join("snapshots").join(name);
        fs::create_dir_all(&snapshot_dir)?;
        let mut temporary = TemporaryFile {
            path: snapshot_dir.join(format!(".{}.tmp", uuid::Uuid::now_v7())),
            committed: false,
        };
        let mut bytes = serde_json::to_vec_pretty(value)?;
        bytes.push(b'\n');
        let digest = blake3::hash(&bytes).to_hex().to_string();
        let target = snapshot_dir.join(format!("{digest}.json"));
        if target.exists() {
            let existing = fs::read(&target)?;
            if existing != bytes {
                return Err(StoreError::SnapshotDigestCollision(digest));
            }
            return Ok(SnapshotReceipt {
                name: name.to_owned(),
                path: PathBuf::from("snapshots")
                    .join(name)
                    .join(format!("{digest}.json")),
                digest,
                bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            });
        }
        {
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary.path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&bytes)?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        fs::hard_link(&temporary.path, &target)?;
        sync_directory(Some(&snapshot_dir))?;
        fs::remove_file(&temporary.path)?;
        temporary.committed = true;
        Ok(SnapshotReceipt {
            name: name.to_owned(),
            path: PathBuf::from("snapshots")
                .join(name)
                .join(format!("{digest}.json")),
            digest,
            bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        })
    }

    fn acquire_write_lock(&self, operation: &str) -> Result<ReviewLock, StoreError> {
        let path = self.root.join(".write.lock");
        fs::create_dir(&path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::AlreadyExists {
                StoreError::WriterLocked
            } else {
                StoreError::Io(source)
            }
        })?;
        let token = format!(
            "token={} pid={} operation={operation}\n",
            uuid::Uuid::now_v7(),
            std::process::id()
        );
        let mut file = File::create(path.join("owner"))?;
        file.write_all(token.as_bytes())?;
        file.sync_all()?;
        Ok(ReviewLock { path })
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
        /// One-based line number of the malformed ledger entry.
        line: usize,
        /// JSON parser error produced for the malformed line.
        source: serde_json::Error,
    },
    /// Audit verification failed.
    #[error(transparent)]
    Audit(#[from] evidence_search_core::AuditError),
    /// Snapshot name could escape its directory.
    #[error("invalid snapshot name")]
    InvalidSnapshotName,
    /// Another process or operation currently holds the single-writer lock.
    #[error("review store is locked by another writer; inspect or explicitly clear a stale lock")]
    WriterLocked,
    /// Event identifier was already present with different content.
    #[error("audit event identifier `{0}` is already present with different content")]
    ConflictingEventId(String),
    /// The audit event failed the compiled event registry or minimisation policy.
    #[error("audit payload is not admissible: {0}")]
    InvalidAuditPayload(String),
    /// A content digest unexpectedly named different bytes.
    #[error("snapshot digest collision for `{0}`")]
    SnapshotDigestCollision(String),
    /// A commit identifier was reused for different bytes.
    #[error("execution commit identifier `{0}` is already present with different content")]
    ConflictingCommitId(String),
    /// Execution commit linkage or contract validation failed.
    #[error("invalid execution commit: {0}")]
    InvalidExecutionCommit(String),
    /// An immutable event segment was missing, reordered, renamed or injected.
    #[error("unexpected audit segment `{actual}`; expected `{expected}`")]
    UnexpectedSegment {
        /// Name implied by the ledger position and event hash.
        expected: String,
        /// Name found on disk.
        actual: String,
    },
    /// A review store was used for a different review identifier.
    #[error("review identifier mismatch: expected `{expected}`, found `{actual}`")]
    ReviewMismatch {
        /// Review identifier bound to the store.
        expected: String,
        /// Review identifier supplied by the attempted operation.
        actual: String,
    },
    /// Candidate event did not point to the current persisted head.
    #[error("audit previous-hash mismatch: expected `{expected}`, found `{actual}`")]
    PreviousHashMismatch {
        /// Hash of the current persisted ledger head.
        expected: String,
        /// Previous-event hash declared by the candidate event.
        actual: String,
    },
}

#[cfg(unix)]
fn sync_directory(path: Option<&Path>) -> Result<(), StoreError> {
    if let Some(path) = path {
        File::open(path)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
const fn sync_directory(_path: Option<&Path>) -> Result<(), StoreError> {
    // Rust's standard library does not expose a portable directory fsync primitive.
    // File contents are still synced before rename; platform-specific durability is
    // reported as a capability limitation rather than silently overclaimed.
    Ok(())
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

fn validate_execution_commit(commit: &ExecutionCommit) -> Result<(), StoreError> {
    if validate_storage_identifier(&commit.commit_id).is_err() {
        return Err(StoreError::InvalidExecutionCommit(
            "commit_id is empty".to_owned(),
        ));
    }
    commit
        .receipt
        .validate()
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    commit
        .audit_event
        .validate()
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    validate_registered_audit_event(&commit.audit_event)
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    if commit.audit_event.review_id != commit.receipt.review_id {
        return Err(StoreError::InvalidExecutionCommit(
            "audit and receipt review_id differ".to_owned(),
        ));
    }
    for record in &commit.records {
        record
            .validate()
            .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
        if record.source_receipt_id != commit.receipt.receipt_id {
            return Err(StoreError::InvalidExecutionCommit(format!(
                "record {} references another receipt",
                record.record_id
            )));
        }
    }
    let digest = canonical_record_digest(&commit.records)
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    if digest != commit.receipt.result_digest {
        return Err(StoreError::InvalidExecutionCommit(
            "receipt result_digest does not bind the committed records".to_owned(),
        ));
    }
    Ok(())
}

fn validate_storage_identifier(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value.starts_with('.')
        || value.contains('/')
        || value.contains('\\')
        || value.contains("..")
    {
        Err(StoreError::InvalidSnapshotName)
    } else {
        Ok(())
    }
}

fn read_jsonl_events(path: &Path, events: &mut Vec<AuditEvent>) -> Result<(), StoreError> {
    let reader = BufReader::new(File::open(path)?);
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
    Ok(())
}

fn read_segment_event(path: &Path) -> Result<AuditEvent, StoreError> {
    let mut events = Vec::new();
    read_jsonl_events(path, &mut events)?;
    if events.len() != 1 {
        return Err(StoreError::UnexpectedSegment {
            expected: "exactly one event".to_owned(),
            actual: format!("{} events in {}", events.len(), path.display()),
        });
    }
    events.pop().ok_or_else(|| StoreError::UnexpectedSegment {
        expected: "exactly one event".to_owned(),
        actual: format!("empty segment {}", path.display()),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        fs::OpenOptions,
        io::Write,
        process::Command,
        thread,
        time::{Duration, Instant},
    };

    use evidence_search_core::AuditLedger;
    use searchright_contracts::{Actor, AuditEventDraft};
    use serde_json::json;

    use super::*;

    fn test_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("searchright-{label}-{}", uuid::Uuid::now_v7()))
    }

    fn event(id: &str, ledger: &mut AuditLedger) -> AuditEvent {
        let appended = ledger.append(AuditEventDraft {
            schema_version: "org.searchright.audit-event.v1".to_owned(),
            event_id: id.to_owned(),
            review_id: "review-1".to_owned(),
            event_type: "review_plan_validated".to_owned(),
            occurred_at: "2026-08-05T00:00:00Z".to_owned(),
            actor: Actor {
                actor_id: "test".to_owned(),
                actor_type: "human".to_owned(),
                provenance: None,
            },
            payload: json!({"plan_id": "plan-1"}),
        });
        assert!(appended.is_ok());
        appended.cloned().unwrap_or_else(|_| unreachable!())
    }

    #[test]
    fn snapshot_receipt_matches_exact_bytes() {
        let directory = std::env::temp_dir().join(format!(
            "searchright-snapshot-test-{}",
            uuid::Uuid::now_v7()
        ));
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let receipt = store.write_snapshot_with_receipt("records", &json!({"a": 1}));
            assert!(receipt.is_ok());
            if let Ok(receipt) = receipt {
                let bytes = fs::read(directory.join(&receipt.path));
                assert!(bytes.is_ok());
                if let Ok(bytes) = bytes {
                    assert_eq!(receipt.digest, blake3::hash(&bytes).to_hex().to_string());
                }
            }
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn snapshots_are_content_addressed_idempotent_and_survive_restart() {
        let directory = test_directory("snapshot-versions-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let first = store.write_snapshot_with_receipt("records", &json!({"a": 1}));
            let retry = store.write_snapshot_with_receipt("records", &json!({"a": 1}));
            let second = store.write_snapshot_with_receipt("records", &json!({"a": 2}));
            assert!(first.is_ok() && retry.is_ok() && second.is_ok());
            if let (Ok(first), Ok(retry), Ok(second)) = (first, retry, second) {
                assert_eq!(first, retry);
                assert_ne!(first.path, second.path);
                drop(store);
                let reopened = FileReviewStore::open(&directory);
                assert!(reopened.is_ok());
                assert!(directory.join(first.path).is_file());
                assert!(directory.join(second.path).is_file());
            }
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn persisted_ledger_round_trips() {
        let directory =
            std::env::temp_dir().join(format!("searchright-test-{}", uuid::Uuid::now_v7()));
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let mut ledger = AuditLedger::new();
            let appended = ledger.append(AuditEventDraft {
                schema_version: "org.searchright.audit-event.v1".to_owned(),
                event_id: "event-1".to_owned(),
                review_id: "review-1".to_owned(),
                event_type: "review_plan_validated".to_owned(),
                occurred_at: "2026-08-05T00:00:00Z".to_owned(),
                actor: Actor {
                    actor_id: "test".to_owned(),
                    actor_type: "human".to_owned(),
                    provenance: None,
                },
                payload: json!({"plan_id": "plan-1"}),
            });
            assert!(appended.is_ok());
            if let Ok(event) = appended {
                assert!(store.append_event(event).is_ok());
            }
            assert!(store.verify_audit().is_ok());
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn restart_verifies_and_continues_from_persisted_head() {
        let directory = test_directory("restart-test");
        let mut ledger = AuditLedger::new();
        let first = event("event-1", &mut ledger);
        {
            let store = FileReviewStore::open(&directory);
            assert!(store.is_ok());
            if let Ok(store) = store {
                assert!(store.append_event(&first).is_ok());
            }
        }

        let reopened = FileReviewStore::open(&directory);
        assert!(reopened.is_ok());
        if let Ok(reopened) = reopened {
            let second = event("event-2", &mut ledger);
            assert!(reopened.append_event(&second).is_ok());
            let verification = reopened.verify_audit();
            assert!(verification.is_ok());
            if let Ok(verification) = verification {
                assert_eq!(verification.event_count, 2);
                assert_eq!(verification.head_hash, Some(second.event_hash));
            }
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn duplicate_append_is_idempotent_at_the_storage_boundary() {
        let directory = test_directory("idempotency-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let mut ledger = AuditLedger::new();
            let first = event("event-1", &mut ledger);
            assert!(store.append_event(&first).is_ok());
            let path = directory.join("events");
            let before = directory_bytes(&path);
            assert!(before.is_ok());

            assert!(store.append_event(&first).is_ok());
            assert_eq!(before.ok(), directory_bytes(&path).ok());
            assert!(store.verify_audit().is_ok());
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn conflicting_duplicate_identifier_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("conflicting-id-test");
        let store = FileReviewStore::open(&directory)?;
        let mut ledger = AuditLedger::new();
        let first = event("event-1", &mut ledger);
        store.append_event(&first)?;

        let mut conflicting = first;
        conflicting.payload = json!({"plan_id": "plan-2"});
        assert!(matches!(
            store.append_event(&conflicting),
            Err(StoreError::ConflictingEventId(event_id)) if event_id == "event-1"
        ));
        assert!(store.verify_audit().is_ok());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn incomplete_trailing_write_fails_closed_after_restart() {
        let directory = test_directory("crash-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let mut ledger = AuditLedger::new();
            let first = event("event-1", &mut ledger);
            assert!(store.append_event(&first).is_ok());
            let segment = fs::read_dir(directory.join("events"))
                .ok()
                .and_then(|mut entries| entries.next())
                .and_then(Result::ok)
                .map(|entry| entry.path());
            assert!(segment.is_some());
            let file = segment.map(|path| OpenOptions::new().append(true).open(path));
            assert!(matches!(&file, Some(Ok(_))));
            if let Some(Ok(mut file)) = file {
                assert!(file.write_all(b"{\"schema_version\":").is_ok());
                assert!(file.sync_all().is_ok());
            }
        }

        let reopened = FileReviewStore::open(&directory);
        assert!(reopened.is_ok());
        if let Ok(reopened) = reopened {
            assert!(matches!(
                reopened.verify_audit(),
                Err(StoreError::MalformedLine { line: 2, .. })
            ));
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn orphaned_pending_segment_is_ignored_after_restart() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = test_directory("orphan-pending-test");
        let store = FileReviewStore::open(&directory)?;
        let mut ledger = AuditLedger::new();
        let first = event("event-1", &mut ledger);
        store.append_event(&first)?;
        fs::write(
            directory.join("events").join(".interrupted.pending"),
            b"{\"partial\":",
        )?;

        let reopened = FileReviewStore::open(&directory)?;
        let verification = reopened.verify_audit()?;
        assert_eq!(verification.event_count, 1);
        let second = event("event-2", &mut ledger);
        reopened.append_event(&second)?;
        assert_eq!(reopened.verify_audit()?.event_count, 2);
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn active_writer_lock_blocks_all_mutations() {
        let directory = test_directory("lock-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let lock = store.acquire_write_lock("test-holder");
            assert!(lock.is_ok());
            assert!(matches!(
                store.write_snapshot("records", &json!({"a": 1})),
                Err(StoreError::WriterLocked)
            ));
            drop(lock);
            assert!(store.write_snapshot("records", &json!({"a": 1})).is_ok());
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn restart_keeps_an_abandoned_lock_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("stale-lock-test");
        let store = FileReviewStore::open(&directory)?;
        fs::create_dir(directory.join(".write.lock"))?;
        fs::write(
            directory.join(".write.lock").join("owner"),
            "token=stale pid=unavailable operation=interrupted\n",
        )?;
        drop(store);

        let reopened = FileReviewStore::open(&directory)?;
        assert!(matches!(
            reopened.write_snapshot("records", &json!({"a": 1})),
            Err(StoreError::WriterLocked)
        ));
        fs::remove_file(directory.join(".write.lock").join("owner"))?;
        fs::remove_dir(directory.join(".write.lock"))?;
        assert!(reopened.write_snapshot("records", &json!({"a": 1})).is_ok());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn multiprocess_child_lock_holder() -> Result<(), Box<dyn std::error::Error>> {
        let Some(directory) = std::env::var_os("SEARCHRIGHT_STORE_CHILD_DIR") else {
            return Ok(());
        };
        let directory = PathBuf::from(directory);
        let store = FileReviewStore::open(&directory)?;
        let _lock = store.acquire_write_lock("multiprocess-test-child")?;
        fs::write(directory.join("child.ready"), b"ready")?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while !directory.join("child.release").exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(directory.join("child.release").exists());
        Ok(())
    }

    #[test]
    fn cooperating_processes_enforce_single_writer() -> Result<(), Box<dyn std::error::Error>> {
        if std::env::var_os("SEARCHRIGHT_STORE_CHILD_DIR").is_some() {
            return Ok(());
        }
        let directory = test_directory("multiprocess-lock-test");
        let executable = std::env::current_exe()?;
        let mut child = Command::new(executable)
            .args([
                "--exact",
                "tests::multiprocess_child_lock_holder",
                "--nocapture",
            ])
            .env("SEARCHRIGHT_STORE_CHILD_DIR", &directory)
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(10);
        while !directory.join("child.ready").exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(directory.join("child.ready").exists());

        let store = FileReviewStore::open(&directory)?;
        assert!(matches!(
            store.write_snapshot("records", &json!({"a": 1})),
            Err(StoreError::WriterLocked)
        ));
        fs::write(directory.join("child.release"), b"release")?;
        assert!(child.wait()?.success());
        assert!(store.write_snapshot("records", &json!({"a": 1})).is_ok());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    fn directory_bytes(path: &Path) -> std::io::Result<Vec<Vec<u8>>> {
        let mut paths = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        paths.into_iter().map(fs::read).collect()
    }
}
