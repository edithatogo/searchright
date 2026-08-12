//! Bounded, fail-closed verification of immutable lifecycle approval records.

use std::collections::BTreeMap;

use searchright_contracts::LifecycleApproval;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::LifecycleApprovalVerifier;

/// Clock injected into approval verification for deterministic expiry decisions.
pub trait ApprovalClock: Send + Sync {
    /// Current canonical UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`).
    fn now_utc(&self) -> String;
}

/// Immutable approval record admitted only after adapter-owned identity/signature checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLifecycleApprovalRecord {
    /// Exact approval fields presented during verification.
    pub approval: LifecycleApproval,
    /// Durable reference to the adapter-owned identity/signature evidence.
    pub authority_evidence_ref: String,
    /// Whether the ingesting adapter positively verified that evidence.
    pub authority_evidence_verified: bool,
}

/// Bounded registry that verifies exact records and consumes approval nonces once.
pub struct BoundedLifecycleApprovalRegistry<C> {
    records: BTreeMap<String, VerifiedLifecycleApprovalRecord>,
    clock: C,
}

impl<C: ApprovalClock> BoundedLifecycleApprovalRegistry<C> {
    /// Build a registry, rejecting unverified, duplicate, malformed or oversized input.
    pub fn new(
        records: impl IntoIterator<Item = VerifiedLifecycleApprovalRecord>,
        maximum_records: usize,
        clock: C,
    ) -> Result<Self, String> {
        if maximum_records == 0 {
            return Err("approval registry bound must be positive".to_owned());
        }
        let mut indexed = BTreeMap::new();
        for record in records {
            if indexed.len() >= maximum_records {
                return Err("approval registry exceeds configured bound".to_owned());
            }
            if !record.authority_evidence_verified
                || record.authority_evidence_ref.trim().is_empty()
                || !canonical_utc(&record.approval.approved_at)
                || !canonical_utc(&record.approval.expires_at)
                || record.approval.approved_at >= record.approval.expires_at
            {
                return Err("approval registry record is not verified and canonical".to_owned());
            }
            let approval_id = record.approval.approval_id.clone();
            if indexed.insert(approval_id, record).is_some() {
                return Err("approval registry contains duplicate approval identifiers".to_owned());
            }
        }
        Ok(Self {
            records: indexed,
            clock,
        })
    }
}

impl<C: ApprovalClock> LifecycleApprovalVerifier for BoundedLifecycleApprovalRegistry<C> {
    fn verify(
        &self,
        approval: &LifecycleApproval,
        request_digest: &str,
        policy_id: &str,
    ) -> Result<(), String> {
        let record = self
            .records
            .get(&approval.approval_id)
            .ok_or_else(|| "approval record is not registered".to_owned())?;
        if &record.approval != approval
            || approval.approved_by.trim().is_empty()
            || approval.policy_id != policy_id
            || approval.request_digest != request_digest
        {
            return Err(
                "approval record does not exactly bind principal, policy and request".to_owned(),
            );
        }
        let now = parse_utc(&self.clock.now_utc())?;
        let approved_at = parse_utc(&approval.approved_at)?;
        let expires_at = parse_utc(&approval.expires_at)?;
        if now < approved_at || now >= expires_at {
            return Err("approval is not currently valid".to_owned());
        }
        // Exact approval replay is safe: the request digest, request identifier and immutable
        // store receipt make the effect idempotent. Consuming authority before durable apply
        // would make crash recovery impossible; broadened reuse fails the equality checks above.
        Ok(())
    }
}

fn canonical_utc(value: &str) -> bool {
    parse_utc(value).is_ok()
}

fn parse_utc(value: &str) -> Result<OffsetDateTime, String> {
    if !value.ends_with('Z') {
        return Err("approval timestamp must use canonical UTC".to_owned());
    }
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| "approval timestamp is not valid RFC 3339".to_owned())
}

#[cfg(test)]
mod tests {
    use searchright_contracts::DataLifecycleAction;

    use super::*;

    #[derive(Clone)]
    struct FixedClock(&'static str);

    impl ApprovalClock for FixedClock {
        fn now_utc(&self) -> String {
            self.0.to_owned()
        }
    }

    fn approval() -> LifecycleApproval {
        LifecycleApproval {
            approval_id: "approval-1".to_owned(),
            approved_by: "principal-1".to_owned(),
            review_id: "review-1".to_owned(),
            action: DataLifecycleAction::Delete,
            request_digest: "a".repeat(64),
            policy_id: "policy-1".to_owned(),
            nonce: "nonce-1".to_owned(),
            approved_at: "2026-08-13T00:00:00Z".to_owned(),
            expires_at: "2026-08-14T00:00:00Z".to_owned(),
        }
    }

    fn record() -> VerifiedLifecycleApprovalRecord {
        VerifiedLifecycleApprovalRecord {
            approval: approval(),
            authority_evidence_ref: "receipt:identity-signature-1".to_owned(),
            authority_evidence_verified: true,
        }
    }

    #[test]
    fn exact_record_allows_only_idempotent_exact_replay() -> Result<(), String> {
        let registry = BoundedLifecycleApprovalRegistry::new(
            [record()],
            1,
            FixedClock("2026-08-13T12:00:00Z"),
        )?;
        registry.verify(&approval(), &"a".repeat(64), "policy-1")?;
        registry.verify(&approval(), &"a".repeat(64), "policy-1")?;
        Ok(())
    }

    #[test]
    fn exact_binding_rejects_principal_policy_digest_nonce_and_timestamp_changes()
    -> Result<(), String> {
        for changed in [
            LifecycleApproval {
                approved_by: "principal-2".to_owned(),
                ..approval()
            },
            LifecycleApproval {
                policy_id: "policy-2".to_owned(),
                ..approval()
            },
            LifecycleApproval {
                request_digest: "b".repeat(64),
                ..approval()
            },
            LifecycleApproval {
                nonce: "nonce-2".to_owned(),
                ..approval()
            },
            LifecycleApproval {
                expires_at: "2026-08-15T00:00:00Z".to_owned(),
                ..approval()
            },
        ] {
            let registry = BoundedLifecycleApprovalRegistry::new(
                [record()],
                1,
                FixedClock("2026-08-13T12:00:00Z"),
            )?;
            assert!(
                registry
                    .verify(&changed, &changed.request_digest, &changed.policy_id)
                    .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn expiry_and_future_approval_fail_closed() -> Result<(), String> {
        for now in ["2026-08-12T23:59:59Z", "2026-08-14T00:00:00Z"] {
            let registry = BoundedLifecycleApprovalRegistry::new([record()], 1, FixedClock(now))?;
            assert!(
                registry
                    .verify(&approval(), &"a".repeat(64), "policy-1")
                    .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn registry_rejects_unverified_duplicate_malformed_and_unbounded_input() {
        let unverified = VerifiedLifecycleApprovalRecord {
            authority_evidence_verified: false,
            ..record()
        };
        assert!(
            BoundedLifecycleApprovalRegistry::new(
                [unverified],
                1,
                FixedClock("2026-08-13T12:00:00Z")
            )
            .is_err()
        );
        assert!(
            BoundedLifecycleApprovalRegistry::new(
                [record(), record()],
                2,
                FixedClock("2026-08-13T12:00:00Z")
            )
            .is_err()
        );
        assert!(
            BoundedLifecycleApprovalRegistry::new(
                [record()],
                0,
                FixedClock("2026-08-13T12:00:00Z")
            )
            .is_err()
        );
        assert!(
            BoundedLifecycleApprovalRegistry::new(
                [record(), record()],
                1,
                FixedClock("2026-08-13T12:00:00Z")
            )
            .is_err()
        );
    }
}
