//! Single-writer append-only storage with same-filesystem, replace-style snapshots.

#![forbid(unsafe_code)]

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use evidence_search_core::{
    AuditLedger, AuditVerification, canonical_record_digest, verify_event_integrity,
};
use searchright_contracts::{
    AuditEvent, BibliographicRecord, DecisionValue, ReviewerKind, ScreeningDecision,
    ScreeningPolicy, SourceReceipt, Validate, validate_registered_audit_event,
};
use searchright_screening::{ScreeningBoard, is_exclusion_decision};
use serde::{Deserialize, Serialize};

mod lifecycle;
pub use lifecycle::{LifecycleStoreReceipt, ManagedObjectReceipt};

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
    /// SHA-256 digest binding the exact approved execution inputs and authority.
    pub binding_digest: String,
    /// Redacted source execution receipt.
    pub receipt: SourceReceipt,
    /// Normalised records introduced by the receipt.
    pub records: Vec<BibliographicRecord>,
    /// Audit event describing the commit.
    pub audit_event: AuditEvent,
}

/// One immutable, complete screening-decision persistence boundary.
///
/// The policy is retained with the full decision so a restart can re-evaluate the
/// same authority boundary. Derived snapshots are deliberately not consulted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreeningDecisionCommit {
    /// Policy under which the decision was admitted.
    pub policy: ScreeningPolicy,
    /// Complete canonical screening decision.
    pub decision: ScreeningDecision,
    /// Whether the recorded reviewer holds final decision authority.
    pub final_authority: bool,
}

/// Externally verified evidence authorising recovery of one abandoned writer lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbandonedLockEvidence {
    /// Exact unpredictable owner token read from the stranded lock.
    pub expected_token: String,
    /// Process identifier recorded with the lock.
    pub expected_pid: u32,
    /// Accountable principal or service that verified the process is no longer alive.
    pub liveness_verified_by: String,
    /// Durable evidence reference for the liveness check.
    pub evidence_reference: String,
}

/// Exact persisted owner identity presented to an abandoned-lock verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockOwnerObservation {
    /// Unpredictable writer instance token.
    pub token: String,
    /// Process identifier recorded by the writer.
    pub pid: u32,
    /// Per-acquisition process-instance identifier used to detect PID reuse.
    pub process_instance_id: String,
}

/// Independent authority that can establish that an observed writer is no longer alive.
pub trait AbandonedLockVerifier {
    /// Verify liveness evidence for the exact persisted owner identity.
    fn verify(
        &self,
        owner: &LockOwnerObservation,
        evidence: &AbandonedLockEvidence,
    ) -> Result<(), String>;
}

/// Receipt emitted after an exact-owner abandoned-lock recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockRecoveryReceipt {
    /// Recovered token, retained for audit correlation.
    pub recovered_token: String,
    /// Process identifier that had owned the lock.
    pub recovered_pid: u32,
    /// Per-acquisition process identity that was independently found abandoned.
    pub recovered_process_instance_id: String,
    /// Accountable liveness verifier.
    pub liveness_verified_by: String,
    /// Durable evidence reference supplied by the verifier.
    pub evidence_reference: String,
}

pub(crate) struct ReviewLock {
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
        fs::create_dir_all(root.join("screening-decisions"))?;
        fs::create_dir_all(root.join("snapshots"))?;
        Ok(Self { root })
    }

    /// Root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Recover a lock only when exact owner and independently verified liveness evidence match.
    ///
    /// The store never guesses that a lock is stale from elapsed time. Callers must first
    /// establish that the recorded process is no longer alive and preserve that evidence.
    pub fn recover_abandoned_lock(
        &self,
        evidence: &AbandonedLockEvidence,
        verifier: &dyn AbandonedLockVerifier,
    ) -> Result<LockRecoveryReceipt, StoreError> {
        if evidence.expected_token.trim().is_empty()
            || evidence.liveness_verified_by.trim().is_empty()
            || evidence.evidence_reference.trim().is_empty()
        {
            return Err(StoreError::InvalidLockRecoveryEvidence);
        }
        let lock = self.root.join(".write.lock");
        let owner_path = lock.join("owner");
        let owner = fs::read_to_string(&owner_path)?;
        let observed = parse_lock_owner(&owner)?;
        if observed.token != evidence.expected_token || observed.pid != evidence.expected_pid {
            return Err(StoreError::LockOwnerMismatch);
        }
        verifier
            .verify(&observed, evidence)
            .map_err(StoreError::LockRecoveryNotVerified)?;
        // Re-read immediately before removal so recovery cannot act on a replaced owner record.
        if parse_lock_owner(&fs::read_to_string(&owner_path)?)? != observed {
            return Err(StoreError::LockOwnerMismatch);
        }
        fs::remove_file(&owner_path)?;
        fs::remove_dir(&lock)?;
        sync_directory(Some(&self.root))?;
        Ok(LockRecoveryReceipt {
            recovered_token: observed.token,
            recovered_pid: observed.pid,
            recovered_process_instance_id: observed.process_instance_id,
            liveness_verified_by: evidence.liveness_verified_by.clone(),
            evidence_reference: evidence.evidence_reference.clone(),
        })
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

    /// Read and fully validate one immutable execution commit by idempotency key.
    pub fn read_execution_commit(
        &self,
        commit_id: &str,
    ) -> Result<Option<ExecutionCommit>, StoreError> {
        validate_storage_identifier(commit_id)
            .map_err(|_| StoreError::InvalidExecutionCommit("commit_id is invalid".to_owned()))?;
        let path = self.root.join("commits").join(format!("{commit_id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let commit: ExecutionCommit = serde_json::from_slice(&fs::read(path)?)?;
        if commit.commit_id != commit_id {
            return Err(StoreError::InvalidExecutionCommit(
                "execution commit filename and commit_id differ".to_owned(),
            ));
        }
        validate_execution_commit(&commit)?;
        Ok(Some(commit))
    }

    /// Persist a complete screening decision as an immutable, idempotent commit.
    ///
    /// Existing commits are replayed through [`ScreeningBoard`] on every write,
    /// including after restart. Agent exclusions are not admitted here because
    /// this primitive has no atomic human-confirmation linkage.
    pub fn append_screening_decision(
        &self,
        policy: &ScreeningPolicy,
        decision: &ScreeningDecision,
    ) -> Result<PathBuf, StoreError> {
        let _lock = self.acquire_write_lock("append-screening-decision")?;
        let existing = self.read_screening_decisions_unlocked()?;
        let mut board = validate_screening_commits(&existing, Some(policy))?;
        validate_screening_authority(decision)?;

        let final_authority = decision.reviewer_kind == ReviewerKind::Human;
        let commit = ScreeningDecisionCommit {
            policy: policy.clone(),
            decision: decision.clone(),
            final_authority,
        };
        let bytes = serde_json::to_vec(&commit)?;
        let relative =
            PathBuf::from("screening-decisions").join(format!("{}.json", decision.decision_id));
        let target = self.root.join(&relative);
        if target.exists() {
            return if fs::read(&target)? == bytes {
                Ok(relative)
            } else {
                Err(StoreError::ConflictingScreeningDecisionId(
                    decision.decision_id.clone(),
                ))
            };
        }

        board
            .submit(decision.clone())
            .map_err(|error| StoreError::InvalidScreeningDecision(error.to_string()))?;
        let directory = self.root.join("screening-decisions");
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

    /// Read and revalidate all canonical screening-decision commits.
    ///
    /// This reads immutable commits only; replaceable snapshots are noncanonical.
    pub fn read_screening_decisions(&self) -> Result<Vec<ScreeningDecisionCommit>, StoreError> {
        let commits = self.read_screening_decisions_unlocked()?;
        if !commits.is_empty() {
            validate_screening_commits(&commits, None)?;
        }
        Ok(commits)
    }

    fn read_screening_decisions_unlocked(
        &self,
    ) -> Result<Vec<ScreeningDecisionCommit>, StoreError> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(self.root.join("screening-decisions"))? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && !entry.file_name().to_string_lossy().starts_with('.')
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                paths.push(entry.path());
            }
        }
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let commit: ScreeningDecisionCommit = serde_json::from_slice(&fs::read(&path)?)?;
                let expected = format!("{}.json", commit.decision.decision_id);
                if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
                    return Err(StoreError::UnexpectedScreeningDecisionFile {
                        expected,
                        actual: path.file_name().map_or_else(
                            || path.display().to_string(),
                            |name| name.to_string_lossy().into_owned(),
                        ),
                    });
                }
                Ok(commit)
            })
            .collect()
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

    pub(crate) fn acquire_write_lock(&self, operation: &str) -> Result<ReviewLock, StoreError> {
        let path = self.root.join(".write.lock");
        fs::create_dir(&path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::AlreadyExists {
                StoreError::WriterLocked
            } else {
                StoreError::Io(source)
            }
        })?;
        let token = uuid::Uuid::now_v7();
        let owner = format!(
            "token={token} pid={} process_instance_id={token} operation={operation}\n",
            std::process::id()
        );
        let mut file = File::create(path.join("owner"))?;
        file.write_all(owner.as_bytes())?;
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
    /// Abandoned-lock recovery evidence was incomplete.
    #[error("abandoned-lock recovery requires exact owner and liveness evidence")]
    InvalidLockRecoveryEvidence,
    /// Independent liveness verification denied the recovery.
    #[error("abandoned-lock recovery was not independently verified: {0}")]
    LockRecoveryNotVerified(String),
    /// The observed lock owner changed or did not match the recovery request.
    #[error("abandoned-lock recovery owner did not match the current lock")]
    LockOwnerMismatch,
    /// The persisted lock owner record was malformed.
    #[error("persisted writer-lock owner record is malformed")]
    MalformedLockOwner,
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
    /// A screening decision identifier was reused for different complete content.
    #[error("screening decision identifier `{0}` is already present with different content")]
    ConflictingScreeningDecisionId(String),
    /// A screening decision or its authority context was invalid.
    #[error("invalid screening decision commit: {0}")]
    InvalidScreeningDecision(String),
    /// An immutable screening-decision file was renamed or injected.
    #[error("unexpected screening decision file `{actual}`; expected `{expected}`")]
    UnexpectedScreeningDecisionFile {
        /// Name implied by the complete decision identifier.
        expected: String,
        /// Name found on disk.
        actual: String,
    },
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
pub(crate) fn sync_directory(path: Option<&Path>) -> Result<(), StoreError> {
    if let Some(path) = path {
        File::open(path)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) const fn sync_directory(_path: Option<&Path>) -> Result<(), StoreError> {
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
    if commit.binding_digest.len() != 64
        || !commit
            .binding_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(StoreError::InvalidExecutionCommit(
            "binding_digest must be a lowercase SHA-256 digest".to_owned(),
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
    verify_event_integrity(&commit.audit_event)
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    validate_registered_audit_event(&commit.audit_event)
        .map_err(|error| StoreError::InvalidExecutionCommit(error.to_string()))?;
    if commit.audit_event.review_id != commit.receipt.review_id {
        return Err(StoreError::InvalidExecutionCommit(
            "audit and receipt review_id differ".to_owned(),
        ));
    }
    if commit.audit_event.event_type != "execution_committed"
        || commit
            .audit_event
            .payload
            .get("commit_id")
            .and_then(serde_json::Value::as_str)
            != Some(commit.commit_id.as_str())
        || commit
            .audit_event
            .payload
            .get("binding_digest")
            .and_then(serde_json::Value::as_str)
            != Some(commit.binding_digest.as_str())
        || commit
            .audit_event
            .payload
            .get("receipt_id")
            .and_then(serde_json::Value::as_str)
            != Some(commit.receipt.receipt_id.as_str())
        || commit
            .audit_event
            .payload
            .get("record_count")
            .and_then(serde_json::Value::as_u64)
            != u64::try_from(commit.records.len()).ok()
        || commit
            .audit_event
            .payload
            .get("run_id")
            .and_then(serde_json::Value::as_str)
            != Some(commit.receipt.run_id.as_str())
        || commit.receipt.records_retrieved
            != u64::try_from(commit.records.len()).unwrap_or(u64::MAX)
    {
        return Err(StoreError::InvalidExecutionCommit(
            "execution_committed audit payload does not bind the commit, receipt and records"
                .to_owned(),
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

fn validate_screening_authority(decision: &ScreeningDecision) -> Result<(), StoreError> {
    if decision.reviewer_kind == ReviewerKind::Agent && is_exclusion_decision(decision.decision) {
        return Err(StoreError::InvalidScreeningDecision(
            "agent exclusion requires an atomically linked human confirmation".to_owned(),
        ));
    }
    Ok(())
}

fn validate_screening_commits(
    commits: &[ScreeningDecisionCommit],
    expected_policy: Option<&ScreeningPolicy>,
) -> Result<ScreeningBoard, StoreError> {
    let policy = match (commits.first(), expected_policy) {
        (Some(first), Some(expected)) if &first.policy != expected => {
            return Err(StoreError::InvalidScreeningDecision(
                "screening policy differs from the immutable persisted policy".to_owned(),
            ));
        }
        (Some(first), _) => first.policy.clone(),
        (None, Some(expected)) => expected.clone(),
        (None, None) => {
            return Err(StoreError::InvalidScreeningDecision(
                "cannot construct a screening board without a policy".to_owned(),
            ));
        }
    };
    let mut board = ScreeningBoard::new(policy.clone())
        .map_err(|error| StoreError::InvalidScreeningDecision(error.to_string()))?;
    let mut review_id: Option<&str> = None;
    for commit in commits {
        if commit.policy != policy {
            return Err(StoreError::InvalidScreeningDecision(
                "screening policy changed within immutable decision commits".to_owned(),
            ));
        }
        validate_storage_identifier(&commit.decision.decision_id).map_err(|_| {
            StoreError::InvalidScreeningDecision("decision_id is not storage-safe".to_owned())
        })?;
        validate_screening_authority(&commit.decision)?;
        let expected_authority = commit.decision.reviewer_kind == ReviewerKind::Human;
        if commit.final_authority != expected_authority {
            return Err(StoreError::InvalidScreeningDecision(
                "final authority does not match the reviewer role".to_owned(),
            ));
        }
        if let Some(expected) = review_id {
            if commit.decision.review_id != expected {
                return Err(StoreError::ReviewMismatch {
                    expected: expected.to_owned(),
                    actual: commit.decision.review_id.clone(),
                });
            }
        } else {
            review_id = Some(&commit.decision.review_id);
        }
        board
            .submit(commit.decision.clone())
            .map_err(|error| StoreError::InvalidScreeningDecision(error.to_string()))?;
    }
    Ok(board)
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

fn parse_lock_owner(owner: &str) -> Result<LockOwnerObservation, StoreError> {
    let mut token = None;
    let mut pid = None;
    let mut process_instance_id = None;
    for part in owner.split_whitespace() {
        if let Some(value) = part.strip_prefix("token=") {
            token = Some(value.to_owned());
        } else if let Some(value) = part.strip_prefix("pid=") {
            pid = value.parse::<u32>().ok();
        } else if let Some(value) = part.strip_prefix("process_instance_id=") {
            process_instance_id = Some(value.to_owned());
        }
    }
    match (token, pid, process_instance_id) {
        (Some(token), Some(pid), Some(process_instance_id))
            if !token.is_empty() && !process_instance_id.is_empty() =>
        {
            Ok(LockOwnerObservation {
                token,
                pid,
                process_instance_id,
            })
        }
        _ => Err(StoreError::MalformedLockOwner),
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
    use searchright_contracts::{
        Actor, AgentAuthority, AuditEventDraft, DecisionValue as Dv, ExclusionReason,
        SCREENING_POLICY_SCHEMA_VERSION, ScreeningRound,
    };
    use serde_json::json;

    use super::*;

    struct AcceptAbandonedLock;

    impl AbandonedLockVerifier for AcceptAbandonedLock {
        fn verify(
            &self,
            owner: &LockOwnerObservation,
            evidence: &AbandonedLockEvidence,
        ) -> Result<(), String> {
            if owner.process_instance_id == "instance-stale"
                && evidence.evidence_reference == "incident-1"
            {
                Ok(())
            } else {
                Err("liveness evidence does not cover this process instance".to_owned())
            }
        }
    }

    struct RejectActiveLock;

    impl AbandonedLockVerifier for RejectActiveLock {
        fn verify(
            &self,
            _owner: &LockOwnerObservation,
            _evidence: &AbandonedLockEvidence,
        ) -> Result<(), String> {
            Err("writer is still active".to_owned())
        }
    }

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

    fn screening_policy() -> ScreeningPolicy {
        ScreeningPolicy {
            schema_version: SCREENING_POLICY_SCHEMA_VERSION.to_owned(),
            title_abstract_reviewers: 2,
            full_text_reviewers: 2,
            agent_authority: AgentAuthority::AdvisoryOnly,
            minimum_agent_sensitivity: Some(0.99),
            independent_blinding: true,
            adjudication_rule: "independent human adjudication".to_owned(),
        }
    }

    fn screening_decision(
        decision_id: &str,
        reviewer_id: &str,
        reviewer_kind: ReviewerKind,
        decision: DecisionValue,
    ) -> ScreeningDecision {
        ScreeningDecision {
            decision_id: decision_id.to_owned(),
            review_id: "review-1".to_owned(),
            subject_id: "record-1".to_owned(),
            round: ScreeningRound::TitleAbstract,
            reviewer_id: reviewer_id.to_owned(),
            reviewer_kind: reviewer_kind.clone(),
            decision,
            exclusion_reason: is_exclusion_decision(decision).then(|| ExclusionReason {
                reason_id: "wrong-population".to_owned(),
                criterion_id: "population".to_owned(),
                label: "Wrong population".to_owned(),
                evidence: None,
            }),
            confidence: (reviewer_kind == ReviewerKind::Agent).then_some(0.995),
            decided_at: "2026-08-29T00:00:00Z".to_owned(),
            rationale: "Evidence-bearing screening rationale".to_owned(),
            eligibility_version: "1".to_owned(),
            agent_provenance: (reviewer_kind == ReviewerKind::Agent)
                .then(|| "model=fixture;version=1;prompt=sha256:test".to_owned()),
        }
    }

    #[test]
    fn complete_screening_decision_is_idempotent_and_survives_restart()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("screening-decision-restart-test");
        let policy = screening_policy();
        let decision = screening_decision(
            "decision-1",
            "reviewer-1",
            ReviewerKind::Human,
            DecisionValue::Include,
        );
        let store = FileReviewStore::open(&directory)?;
        let first = store.append_screening_decision(&policy, &decision)?;
        assert_eq!(store.append_screening_decision(&policy, &decision)?, first);
        drop(store);

        let reopened = FileReviewStore::open(&directory)?;
        assert_eq!(
            reopened.append_screening_decision(&policy, &decision)?,
            first
        );
        let commits = reopened.read_screening_decisions()?;
        assert_eq!(commits.len(), 1);
        let Some(commit) = commits.first() else {
            return Err("screening commit disappeared after restart".into());
        };
        assert_eq!(commit.decision, decision);
        assert!(commit.final_authority);
        assert_eq!(commit.policy, policy);
        assert!(!directory.join("snapshots").join("screening").exists());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn screening_role_policy_agent_exclusion_and_conflicting_id_fail_closed()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("screening-decision-authority-test");
        let policy = screening_policy();
        let store = FileReviewStore::open(&directory)?;

        let agent_exclusion = screening_decision(
            "agent-exclusion",
            "agent-1",
            ReviewerKind::Agent,
            Dv::Exclude,
        );
        assert!(matches!(
            store.append_screening_decision(&policy, &agent_exclusion),
            Err(StoreError::InvalidScreeningDecision(_))
        ));
        assert!(store.read_screening_decisions()?.is_empty());

        let first = screening_decision(
            "decision-1",
            "reviewer-1",
            ReviewerKind::Human,
            DecisionValue::Include,
        );
        store.append_screening_decision(&policy, &first)?;
        let conflict =
            screening_decision("decision-1", "reviewer-2", ReviewerKind::Human, Dv::Exclude);
        assert!(matches!(
            store.append_screening_decision(&policy, &conflict),
            Err(StoreError::ConflictingScreeningDecisionId(id)) if id == "decision-1"
        ));

        let duplicate_reviewer =
            screening_decision("decision-2", "reviewer-1", ReviewerKind::Human, Dv::Exclude);
        assert!(matches!(
            store.append_screening_decision(&policy, &duplicate_reviewer),
            Err(StoreError::InvalidScreeningDecision(_))
        ));
        assert_eq!(store.read_screening_decisions()?.len(), 1);
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn screening_policy_change_and_tampered_role_authority_fail_after_restart()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("screening-decision-policy-test");
        let policy = screening_policy();
        let store = FileReviewStore::open(&directory)?;
        let decision = screening_decision(
            "decision-1",
            "reviewer-1",
            ReviewerKind::Human,
            DecisionValue::Include,
        );
        let relative = store.append_screening_decision(&policy, &decision)?;

        let mut changed_policy = policy;
        changed_policy.title_abstract_reviewers = 1;
        let second = screening_decision(
            "decision-2",
            "reviewer-2",
            ReviewerKind::Human,
            DecisionValue::Include,
        );
        assert!(matches!(
            store.append_screening_decision(&changed_policy, &second),
            Err(StoreError::InvalidScreeningDecision(_))
        ));

        let bytes = fs::read(directory.join(&relative))?;
        let mut persisted: ScreeningDecisionCommit = serde_json::from_slice(&bytes)?;
        persisted.final_authority = false;
        fs::write(directory.join(relative), serde_json::to_vec(&persisted)?)?;
        drop(store);
        let reopened = FileReviewStore::open(&directory)?;
        assert!(matches!(
            reopened.read_screening_decisions(),
            Err(StoreError::InvalidScreeningDecision(_))
        ));
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
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
    fn invalid_snapshot_names_fail_closed() {
        let directory = test_directory("snapshot-name-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            for name in ["", ".hidden", "../escape", "a/b", "a\\b"] {
                assert!(matches!(
                    store.write_snapshot(name, &json!({})),
                    Err(StoreError::InvalidSnapshotName)
                ));
            }
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn review_previous_hash_and_payload_mismatches_fail_closed() {
        let directory = test_directory("event-validation-test");
        let store = FileReviewStore::open(&directory);
        assert!(store.is_ok());
        if let Ok(store) = store {
            let mut ledger = AuditLedger::new();
            let first = event("event-1", &mut ledger);
            assert!(store.append_event(&first).is_ok());

            let mut wrong_review = event("event-2", &mut ledger);
            wrong_review.review_id = "other-review".to_owned();
            assert!(matches!(
                store.append_event(&wrong_review),
                Err(StoreError::ReviewMismatch { .. })
            ));

            let mut wrong_previous = event("event-3", &mut ledger);
            wrong_previous.previous_hash = "wrong".to_owned();
            assert!(matches!(
                store.append_event(&wrong_previous),
                Err(StoreError::PreviousHashMismatch { .. })
            ));

            let mut prohibited = event("event-4", &mut ledger);
            prohibited.payload = json!({"plan_id": "p", "token": "secret"});
            assert!(matches!(
                store.append_event(&prohibited),
                Err(StoreError::InvalidAuditPayload(_))
            ));
        }
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn renamed_and_multi_event_segments_fail_closed() {
        let directory = test_directory("segment-validation-test");
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
            if let Some(segment) = segment {
                let renamed = directory
                    .join("events")
                    .join("00000000000000000099-wrong.json");
                assert!(fs::rename(&segment, &renamed).is_ok());
                assert!(matches!(
                    store.read_events(),
                    Err(StoreError::UnexpectedSegment { .. })
                ));
                assert!(fs::rename(&renamed, &segment).is_ok());
                let bytes = fs::read(&segment);
                assert!(bytes.is_ok());
                if let Ok(bytes) = bytes {
                    let mut doubled = bytes.clone();
                    doubled.extend(bytes);
                    assert!(fs::write(&segment, doubled).is_ok());
                    assert!(matches!(
                        store.read_events(),
                        Err(StoreError::UnexpectedSegment { .. })
                    ));
                }
            }
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
            "token=stale pid=424242 process_instance_id=instance-stale operation=interrupted\n",
        )?;
        drop(store);

        let reopened = FileReviewStore::open(&directory)?;
        assert!(matches!(
            reopened.write_snapshot("records", &json!({"a": 1})),
            Err(StoreError::WriterLocked)
        ));
        assert!(matches!(
            reopened.recover_abandoned_lock(
                &AbandonedLockEvidence {
                    expected_token: "wrong".to_owned(),
                    expected_pid: 424_242,
                    liveness_verified_by: "operator-1".to_owned(),
                    evidence_reference: "incident-1".to_owned(),
                },
                &AcceptAbandonedLock,
            ),
            Err(StoreError::LockOwnerMismatch)
        ));
        let evidence = AbandonedLockEvidence {
            expected_token: "stale".to_owned(),
            expected_pid: 424_242,
            liveness_verified_by: "operator-1".to_owned(),
            evidence_reference: "incident-1".to_owned(),
        };
        assert!(matches!(
            reopened.recover_abandoned_lock(&evidence, &RejectActiveLock),
            Err(StoreError::LockRecoveryNotVerified(_))
        ));
        assert!(directory.join(".write.lock").exists());
        let receipt = reopened.recover_abandoned_lock(&evidence, &AcceptAbandonedLock)?;
        assert_eq!(receipt.recovered_token, "stale");
        assert_eq!(receipt.recovered_pid, 424_242);
        assert_eq!(receipt.recovered_process_instance_id, "instance-stale");
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

    fn execution_commit(commit_id: &str, receipt_id: &str, run_id: &str) -> ExecutionCommit {
        let records = vec![BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: "record-1".to_owned(),
            source_receipt_id: receipt_id.to_owned(),
            native_id: "native-1".to_owned(),
            kind: searchright_contracts::RecordKind::JournalArticle,
            identifiers: searchright_contracts::RecordIdentifiers::default(),
            title: "Example record".to_owned(),
            abstract_text: None,
            authors: Vec::new(),
            container_title: None,
            publication_year: Some(2026),
            publication_date: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            urls: Vec::new(),
            provider_metadata: serde_json::Value::Null,
        }];
        let result_digest = canonical_record_digest(&records).unwrap_or_else(|_| unreachable!());
        let receipt = SourceReceipt {
            schema_version: searchright_contracts::SOURCE_RECEIPT_SCHEMA_VERSION.to_owned(),
            receipt_id: receipt_id.to_owned(),
            review_id: "review-1".to_owned(),
            run_id: run_id.to_owned(),
            provider_id: "fixture".to_owned(),
            source_label: "Fixture".to_owned(),
            strategy_id: "strategy-1".to_owned(),
            query_hash: "query-hash".to_owned(),
            executed_at: "2026-08-13T00:00:00Z".to_owned(),
            records_retrieved: 1,
            pages_retrieved: 1,
            execution_mode: "fixture".to_owned(),
            endpoint: None,
            policy: searchright_contracts::ExecutionPolicy {
                live_enabled: false,
                max_records: 10,
                max_pages: 1,
                timeout_seconds: 10,
                total_timeout_seconds: Some(10),
                max_retries: 0,
                min_interval_ms: 0,
                retry_base_delay_ms: None,
                retry_max_delay_ms: None,
                max_response_bytes: Some(1024),
                replay_enabled: true,
                cache_write_enabled: false,
            },
            provider_version: "1".to_owned(),
            compiler_version: "1".to_owned(),
            result_digest,
            cache_hits: 0,
            cache_writes: 0,
            warnings: Vec::new(),
        };
        let mut ledger = AuditLedger::new();
        let audit_event = ledger
            .append(AuditEventDraft {
                schema_version: searchright_contracts::AUDIT_EVENT_SCHEMA_VERSION.to_owned(),
                event_id: format!("execution-{commit_id}"),
                review_id: receipt.review_id.clone(),
                event_type: "execution_committed".to_owned(),
                occurred_at: "2026-08-13T00:00:01Z".to_owned(),
                actor: Actor {
                    actor_id: "runtime".to_owned(),
                    actor_type: "service".to_owned(),
                    provenance: None,
                },
                payload: json!({
                    "_schema_version": 1,
                    "commit_id": commit_id,
                    "binding_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "receipt_id": receipt_id,
                    "record_count": records.len(),
                    "run_id": run_id,
                }),
            })
            .cloned()
            .unwrap_or_else(|_| unreachable!());
        ExecutionCommit {
            commit_id: commit_id.to_owned(),
            binding_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_owned(),
            receipt,
            records,
            audit_event,
        }
    }

    #[test]
    fn execution_commit_is_idempotent_and_survives_restart()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("execution-commit-restart-test");
        let commit = execution_commit("commit-1", "receipt-1", "run-1");
        let store = FileReviewStore::open(&directory)?;
        let first = store.append_execution_commit(&commit)?;
        let retry = store.append_execution_commit(&commit)?;
        assert_eq!(first, retry);
        drop(store);

        let reopened = FileReviewStore::open(&directory)?;
        assert_eq!(reopened.append_execution_commit(&commit)?, first);
        let persisted: ExecutionCommit = serde_json::from_slice(&fs::read(directory.join(first))?)?;
        assert_eq!(persisted, commit);
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn execution_commit_conflict_tamper_and_binding_fail_closed()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("execution-commit-validation-test");
        let store = FileReviewStore::open(&directory)?;
        let commit = execution_commit("commit-1", "receipt-1", "run-1");
        store.append_execution_commit(&commit)?;

        let conflict = execution_commit("commit-1", "receipt-2", "run-2");
        assert!(matches!(
            store.append_execution_commit(&conflict),
            Err(StoreError::ConflictingCommitId(id)) if id == "commit-1"
        ));

        let mut tampered = execution_commit("commit-2", "receipt-2", "run-2");
        if let Some(payload) = tampered.audit_event.payload.as_object_mut() {
            payload.insert("record_count".to_owned(), json!(2));
        }
        assert!(matches!(
            store.append_execution_commit(&tampered),
            Err(StoreError::InvalidExecutionCommit(_))
        ));

        let mut wrong_run = execution_commit("commit-3", "receipt-3", "run-3");
        wrong_run.receipt.run_id = "other-run".to_owned();
        assert!(matches!(
            store.append_execution_commit(&wrong_run),
            Err(StoreError::InvalidExecutionCommit(_))
        ));

        let mut wrong_count = execution_commit("commit-4", "receipt-4", "run-4");
        wrong_count.receipt.records_retrieved = 2;
        assert!(matches!(
            store.append_execution_commit(&wrong_count),
            Err(StoreError::InvalidExecutionCommit(_))
        ));

        let mut wrong_commit = execution_commit("commit-5", "receipt-5", "run-5");
        wrong_commit.commit_id = "other-commit".to_owned();
        assert!(matches!(
            store.append_execution_commit(&wrong_commit),
            Err(StoreError::InvalidExecutionCommit(_))
        ));
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn orphaned_pending_execution_commit_is_ignored() -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("execution-commit-orphan-test");
        let store = FileReviewStore::open(&directory)?;
        fs::write(
            directory.join("commits").join(".interrupted.pending"),
            b"partial",
        )?;
        let commit = execution_commit("commit-1", "receipt-1", "run-1");
        let relative = store.append_execution_commit(&commit)?;
        assert!(directory.join(relative).is_file());
        assert_eq!(
            fs::read(directory.join("commits").join(".interrupted.pending"))?,
            b"partial"
        );
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
