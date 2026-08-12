//! Durable, immutable lifecycle effects for the filesystem store.

use std::{fs, fs::OpenOptions, path::PathBuf};

use searchright_contracts::{DataLifecycleAction, DataLifecycleDecision, DataLifecycleRequest};
use searchright_governance::{
    LifecycleAuthorization, LifecycleEffectReceipt, LifecycleEffectSink, lifecycle_decision_digest,
    lifecycle_resulting_head,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::FileReviewStore;

/// Receipt for inserting a mutable managed payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedObjectReceipt {
    /// Logical object identifier.
    pub object_id: String,
    /// SHA-256 of exact payload bytes.
    pub digest: String,
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
        let receipts = self.root().join("lifecycle").join("receipts");
        if !receipts.exists() {
            return Ok("GENESIS".to_owned());
        }
        let mut pending = fs::read_dir(receipts)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == "json")
            })
            .map(|entry| {
                fs::read(entry.path())
                    .map_err(|error| error.to_string())
                    .and_then(|bytes| {
                        serde_json::from_slice::<LifecycleStoreReceipt>(&bytes)
                            .map_err(|error| error.to_string())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut head = "GENESIS".to_owned();
        while let Some(index) = pending
            .iter()
            .position(|receipt| receipt.previous_head == head)
        {
            head = pending.swap_remove(index).resulting_head;
        }
        if pending.is_empty() {
            Ok(head)
        } else {
            Err("lifecycle receipt chain is disconnected or forks".to_owned())
        }
    }
}

impl LifecycleEffectSink for FileReviewStore {
    fn current_head(&self) -> Result<String, String> {
        self.lifecycle_head()
    }

    fn apply(
        &mut self,
        authorization: &LifecycleAuthorization,
        expected_head: &str,
    ) -> Result<LifecycleEffectReceipt, String> {
        let request = authorization.request();
        let decision = authorization.decision();
        if !decision.effects_authorized || decision.request_id != request.request_id {
            return Err("lifecycle decision does not authorize this request".to_owned());
        }
        let lifecycle_directory = self.root().join("lifecycle");
        fs::create_dir_all(&lifecycle_directory).map_err(|error| error.to_string())?;
        let _lock = self
            .acquire_write_lock("apply-lifecycle")
            .map_err(|error| error.to_string())?;
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
        let receipt_exists = target.exists();
        if receipt_exists {
            let existing = fs::read(&target).map_err(|error| error.to_string())?;
            if existing != bytes {
                return Err("lifecycle request id was replayed with different effects".to_owned());
            }
        }
        if observed != expected_head && !(receipt_exists && observed == resulting_head) {
            return Err(format!(
                "lifecycle head mismatch: expected {expected_head}, found {observed}"
            ));
        }
        if !receipt_exists {
            for target_id in &request.target_ids {
                if matches!(request.action, DataLifecycleAction::Delete)
                    && !self.root().join("managed").join(target_id).exists()
                {
                    return Err(format!(
                        "managed lifecycle target `{target_id}` does not exist"
                    ));
                }
            }
            commit_absent(&target, &bytes)?;
        }
        // The immutable authorization receipt is durable before any mutable effect. Exact replay
        // completes an interrupted deletion and treats already-removed targets as idempotent.
        if matches!(request.action, DataLifecycleAction::Delete) {
            for target_id in &request.target_ids {
                let source = self.root().join("managed").join(target_id);
                if source.exists() {
                    fs::remove_file(&source)
                        .map_err(|error| format!("remove managed target: {error}"))?;
                }
            }
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
        DATA_LIFECYCLE_REQUEST_SCHEMA_VERSION, DataClassification, DeploymentMode,
        INSTITUTIONAL_POLICY_SCHEMA_VERSION, InstitutionalPolicy, LifecycleApproval,
        LifecycleExecutionMode,
    };
    use searchright_governance::{LifecycleApprovalVerifier, authorize_lifecycle};

    struct AcceptVerifier;

    impl LifecycleApprovalVerifier for AcceptVerifier {
        fn verify(
            &self,
            _approval: &LifecycleApproval,
            _request_digest: &str,
            _policy_id: &str,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    fn policy() -> InstitutionalPolicy {
        InstitutionalPolicy {
            schema_version: INSTITUTIONAL_POLICY_SCHEMA_VERSION.to_owned(),
            policy_id: "policy-1".to_owned(),
            institution: "Test institution".to_owned(),
            deployment_modes: vec![DeploymentMode::LocalOnly],
            allowed_classifications: vec![DataClassification::PublicMetadata],
            permitted_regions: vec!["AU".to_owned()],
            maximum_retention_days: 30,
            telemetry_allowed: false,
            full_text_persistence_allowed: false,
            external_model_processing_allowed: false,
            cross_border_transfer_allowed: false,
            approved_by: "governance-owner".to_owned(),
            effective_from: "2026-08-13".to_owned(),
            review_by: Some("2027-08-13".to_owned()),
        }
    }

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
            approval: Some(LifecycleApproval {
                approval_id: "approval-1".to_owned(),
                approved_by: "accountable-owner".to_owned(),
                review_id: "review-1".to_owned(),
                action: DataLifecycleAction::Delete,
                request_digest: String::new(),
                policy_id: "policy-1".to_owned(),
                nonce: "nonce-1".to_owned(),
                approved_at: "2026-08-13T00:00:00Z".to_owned(),
                expires_at: "2026-08-14T00:00:00Z".to_owned(),
            }),
        }
    }

    fn authorization() -> Result<LifecycleAuthorization, String> {
        let mut request = request();
        let digest = request.effects_digest();
        if let Some(approval) = request.approval.as_mut() {
            approval.request_digest = digest;
        }
        authorize_lifecycle(&policy(), &request, &AcceptVerifier).map_err(|error| error.to_string())
    }

    #[test]
    fn lifecycle_delete_is_durable_idempotent_and_restartable() -> Result<(), String> {
        let directory =
            std::env::temp_dir().join(format!("searchright-lifecycle-{}", uuid::Uuid::now_v7()));
        let mut store =
            FileReviewStore::open(&directory).map_err(|error| format!("open store: {error}"))?;
        let inserted = store
            .put_managed_object("record-1", b"mutable")
            .map_err(|error| format!("put managed object: {error}"))?;
        assert_eq!(inserted.digest.len(), 64);
        let authorization = authorization()?;
        let first = store
            .apply(&authorization, "GENESIS")
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
        let mut reopened =
            FileReviewStore::open(&directory).map_err(|error| format!("reopen store: {error}"))?;
        assert_eq!(
            first,
            reopened
                .apply(&authorization, "GENESIS")
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
        let authorization = authorization()?;
        assert!(store.apply(&authorization, "wrong").is_err());
        assert!(store.apply(&authorization, "GENESIS").is_err());
        assert!(store.put_managed_object("../escape", b"x").is_err());
        store.put_managed_object("record-1", b"one")?;
        assert!(store.put_managed_object("record-1", b"two").is_err());
        let _cleanup = fs::remove_dir_all(directory);
        Ok(())
    }
}
