//! Durable, immutable lifecycle effects for the filesystem store.

use std::{fs, fs::OpenOptions, path::PathBuf};

use searchright_contracts::{DataLifecycleAction, DataLifecycleDecision, DataLifecycleRequest};
use searchright_governance::{
    LifecycleEffectReceipt, LifecycleEffectSink, lifecycle_decision_digest,
    lifecycle_resulting_head,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::FileReviewStore;

struct LifecycleLock(PathBuf);

impl Drop for LifecycleLock {
    fn drop(&mut self) {
        let _owner_result = fs::remove_file(self.0.join("owner"));
        let _result = fs::remove_dir(&self.0);
    }
}

/// Receipt for inserting a mutable managed payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedObjectReceipt {
    /// Logical object identifier.
    pub object_id: String,
    /// SHA-256 of exact payload bytes.
    pub digest: String,
}

/// Evidence from exact-token stale lifecycle lock recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleLockRecoveryReceipt {
    /// SHA-256 of the exact removed owner token.
    pub owner_token_digest: String,
}

/// Exact durable lifecycle record persisted as one immutable object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleStoreReceipt {
    /// Authorized request.
    pub request: DataLifecycleRequest,
    /// Policy decision authorizing effects.
    pub decision: DataLifecycleDecision,
    /// Head before the effect.
    pub previous_head: String,
    /// Stable target identifiers whose managed payloads were removed.
    pub tombstones: Vec<String>,
    /// Head after this receipt.
    pub resulting_head: String,
}

impl FileReviewStore {
    /// Observe the opaque lifecycle writer token for external liveness assessment.
    pub fn lifecycle_lock_token(&self) -> Result<Option<String>, String> {
        let owner = self
            .root()
            .join("lifecycle")
            .join(".write-lock")
            .join("owner");
        match fs::read_to_string(owner) {
            Ok(token) => Ok(Some(token)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    /// Remove a proven-stale lock only when its exact owner token is unchanged.
    pub fn recover_stale_lifecycle_lock(
        &self,
        expected_token: &str,
    ) -> Result<LifecycleLockRecoveryReceipt, String> {
        let lock = self.root().join("lifecycle").join(".write-lock");
        let owner = lock.join("owner");
        let observed = fs::read_to_string(&owner).map_err(|error| error.to_string())?;
        if observed != expected_token {
            return Err("lifecycle lock owner token changed".to_owned());
        }
        fs::remove_file(owner).map_err(|error| error.to_string())?;
        fs::remove_dir(lock).map_err(|error| error.to_string())?;
        Ok(LifecycleLockRecoveryReceipt {
            owner_token_digest: sha256(observed.as_bytes()),
        })
    }

    /// Insert or idempotently confirm a bounded managed payload.
    pub fn put_managed_object(
        &self,
        object_id: &str,
        bytes: &[u8],
    ) -> Result<ManagedObjectReceipt, String> {
        validate_id(object_id)?;
        let directory = self.root().join("managed");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let path = directory.join(object_id);
        if path.exists() {
            if fs::read(&path).map_err(|error| error.to_string())? != bytes {
                return Err(format!(
                    "managed object `{object_id}` already has different bytes"
                ));
            }
        } else {
            fs::write(&path, bytes).map_err(|error| error.to_string())?;
            OpenOptions::new()
                .write(true)
                .open(&path)
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
        }
        Ok(ManagedObjectReceipt {
            object_id: object_id.to_owned(),
            digest: sha256(bytes),
        })
    }

    fn lifecycle_head(&self) -> Result<String, String> {
        let path = self.root().join("lifecycle").join("HEAD");
        match fs::read_to_string(path) {
            Ok(head) => Ok(head),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("GENESIS".to_owned()),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl LifecycleEffectSink for FileReviewStore {
    fn current_head(&self) -> Result<String, String> {
        self.lifecycle_head()
    }

    fn apply(
        &mut self,
        request: &DataLifecycleRequest,
        decision: &DataLifecycleDecision,
        expected_head: &str,
    ) -> Result<LifecycleEffectReceipt, String> {
        if !decision.effects_authorized || decision.request_id != request.request_id {
            return Err("lifecycle decision does not authorize this request".to_owned());
        }
        let lifecycle_directory = self.root().join("lifecycle");
        fs::create_dir_all(&lifecycle_directory).map_err(|error| error.to_string())?;
        let lock_path = lifecycle_directory.join(".write-lock");
        fs::create_dir(&lock_path).map_err(|error| format!("lifecycle writer locked: {error}"))?;
        fs::write(
            lock_path.join("owner"),
            format!("{}:{}", std::process::id(), uuid::Uuid::now_v7()),
        )
        .map_err(|error| error.to_string())?;
        let _lock = LifecycleLock(lock_path);
        let observed = self.lifecycle_head()?;
        let directory = self.root().join("lifecycle");
        let receipts = directory.join("receipts");
        fs::create_dir_all(&receipts).map_err(|error| error.to_string())?;
        validate_id(&request.request_id)?;

        let tombstones = if matches!(request.action, DataLifecycleAction::Delete) {
            for target in &request.target_ids {
                validate_id(target)?;
            }
            request.target_ids.clone()
        } else {
            Vec::new()
        };
        let base = LifecycleStoreReceipt {
            request: request.clone(),
            decision: decision.clone(),
            previous_head: expected_head.to_owned(),
            tombstones,
            resulting_head: String::new(),
        };
        let request_digest = request.effects_digest();
        let decision_digest = lifecycle_decision_digest(decision)?;
        let resulting_head =
            lifecycle_resulting_head(expected_head, &request_digest, &decision_digest);
        let durable = LifecycleStoreReceipt {
            resulting_head: resulting_head.clone(),
            ..base
        };
        let bytes = serde_json::to_vec(&durable).map_err(|error| error.to_string())?;
        let receipt_digest = sha256(&bytes);
        let target = receipts.join(format!("{}.json", request.request_id));
        if target.exists() {
            let existing = fs::read(&target).map_err(|error| error.to_string())?;
            if existing != bytes {
                return Err("lifecycle request id was replayed with different effects".to_owned());
            }
            if observed == resulting_head {
                return Ok(effect_receipt(
                    request,
                    expected_head,
                    resulting_head,
                    receipt_digest,
                    request_digest,
                    decision_digest,
                ));
            }
        }
        if observed != expected_head {
            return Err(format!(
                "lifecycle head mismatch: expected {expected_head}, found {observed}"
            ));
        }
        let staged = directory.join("pending-effects").join(&request.request_id);
        if matches!(request.action, DataLifecycleAction::Delete) {
            fs::create_dir_all(&staged).map_err(|error| error.to_string())?;
            for target_id in &request.target_ids {
                let source = self.root().join("managed").join(target_id);
                let staged_target = staged.join(target_id);
                if source.exists() {
                    fs::hard_link(&source, &staged_target)
                        .map_err(|error| format!("stage managed target: {error}"))?;
                    fs::remove_file(&source)
                        .map_err(|error| format!("remove managed target: {error}"))?;
                } else if !staged_target.exists() && !target.exists() {
                    return Err(format!(
                        "managed lifecycle target `{target_id}` does not exist"
                    ));
                }
            }
        }
        if !target.exists() {
            commit_absent(&target, &bytes)?;
        }
        let head = directory.join("HEAD");
        if head.exists() {
            let old = fs::read_to_string(&head).map_err(|error| error.to_string())?;
            if old == resulting_head {
                return Ok(effect_receipt(
                    request,
                    expected_head,
                    resulting_head,
                    receipt_digest,
                    request_digest,
                    decision_digest,
                ));
            }
            if old != expected_head {
                return Err("lifecycle head changed before publication".to_owned());
            }
        }
        let pending = directory.join(format!(".head-{}.pending", request.request_id));
        fs::write(&pending, &resulting_head).map_err(|error| error.to_string())?;
        OpenOptions::new()
            .write(true)
            .open(&pending)
            .and_then(|file| file.sync_all())
            .map_err(|error| error.to_string())?;
        if head.exists() {
            // Windows does not replace an existing destination with `rename`. The immutable
            // receipt is already durable, so a crash after removal is repaired by exact replay.
            fs::remove_file(&head).map_err(|error| format!("remove prior head: {error}"))?;
        }
        fs::rename(&pending, &head).map_err(|error| format!("publish head: {error}"))?;
        if staged.exists() {
            fs::remove_dir_all(staged)
                .map_err(|error| format!("remove staged targets: {error}"))?;
        }
        Ok(effect_receipt(
            request,
            expected_head,
            resulting_head,
            receipt_digest,
            request_digest,
            decision_digest,
        ))
    }
}

fn effect_receipt(
    request: &DataLifecycleRequest,
    previous_head: &str,
    resulting_head: String,
    receipt_digest: String,
    request_digest: String,
    decision_digest: String,
) -> LifecycleEffectReceipt {
    LifecycleEffectReceipt {
        request_id: request.request_id.clone(),
        previous_head: previous_head.to_owned(),
        resulting_head,
        receipt_digest,
        request_digest,
        decision_digest,
    }
}

fn commit_absent(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    let pending = path.with_extension("pending");
    fs::write(&pending, bytes).map_err(|error| error.to_string())?;
    OpenOptions::new()
        .write(true)
        .open(&pending)
        .and_then(|file| file.sync_all())
        .map_err(|error| error.to_string())?;
    fs::hard_link(&pending, path).map_err(|error| format!("publish receipt: {error}"))?;
    fs::remove_file(pending).map_err(|error| format!("remove receipt pending file: {error}"))
}

fn validate_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.starts_with('.')
        || value.contains(['/', '\\'])
        || value.contains("..")
    {
        Err("invalid lifecycle storage identifier".to_owned())
    } else {
        Ok(())
    }
}

fn sha256(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        if let (Some(high), Some(low)) = (
            HEX.get(usize::from(byte >> 4)),
            HEX.get(usize::from(byte & 0x0f)),
        ) {
            output.push(char::from(*high));
            output.push(char::from(*low));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use searchright_contracts::{
        DATA_LIFECYCLE_DECISION_SCHEMA_VERSION, DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION,
        DataClassification, LifecycleExecutionMode,
    };

    fn request() -> DataLifecycleRequest {
        DataLifecycleRequest {
            schema_version: DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "delete-1".to_owned(),
            review_id: "review-1".to_owned(),
            classification: DataClassification::PublicMetadata,
            action: DataLifecycleAction::Delete,
            execution_mode: LifecycleExecutionMode::Apply,
            target_ids: vec!["record-1".to_owned()],
            retention_days: None,
            export_destination: None,
            includes_audit_log: false,
            legal_hold: false,
            approval: None,
        }
    }

    fn decision() -> DataLifecycleDecision {
        DataLifecycleDecision {
            schema_version: DATA_LIFECYCLE_DECISION_SCHEMA_VERSION.to_owned(),
            request_id: "delete-1".to_owned(),
            policy_id: "policy-1".to_owned(),
            permitted: true,
            effects_authorized: true,
            blockers: Vec::new(),
            warnings: Vec::new(),
            immutable_audit_preserved: true,
            tombstone_target_ids: vec!["record-1".to_owned()],
            receipt_required: true,
        }
    }

    #[test]
    fn lifecycle_delete_is_durable_idempotent_and_restartable() -> Result<(), String> {
        let directory =
            std::env::temp_dir().join(format!("searchright-lifecycle-{}", uuid::Uuid::now_v7()));
        let mut store = FileReviewStore::open(&directory)
            .map_err(|error| format!("open store: {error}"))?;
        let inserted = store
            .put_managed_object("record-1", b"mutable")
            .map_err(|error| format!("put managed object: {error}"))?;
        assert_eq!(inserted.digest.len(), 64);
        let first = store
            .apply(&request(), &decision(), "GENESIS")
            .map_err(|error| format!("first lifecycle apply: {error}"))?;
        assert_eq!(first.resulting_head, store.current_head()?);
        assert!(!directory.join("managed").join("record-1").exists());
        assert!(
            directory
                .join("lifecycle")
                .join("receipts")
                .join("delete-1.json")
                .is_file()
        );
        drop(store);
        let mut reopened = FileReviewStore::open(&directory)
            .map_err(|error| format!("reopen store: {error}"))?;
        assert_eq!(
            first,
            reopened
                .apply(&request(), &decision(), "GENESIS")
                .map_err(|error| format!("replay lifecycle apply: {error}"))?
        );
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }

    #[test]
    fn lifecycle_rejects_conflict_wrong_head_and_missing_target() -> Result<(), String> {
        let directory = std::env::temp_dir().join(format!(
            "searchright-lifecycle-errors-{}",
            uuid::Uuid::now_v7()
        ));
        let mut store = FileReviewStore::open(&directory).map_err(|error| error.to_string())?;
        assert!(store.apply(&request(), &decision(), "wrong").is_err());
        assert!(store.apply(&request(), &decision(), "GENESIS").is_err());
        assert!(store.put_managed_object("../escape", b"x").is_err());
        store.put_managed_object("record-1", b"one")?;
        assert!(store.put_managed_object("record-1", b"two").is_err());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }
}
