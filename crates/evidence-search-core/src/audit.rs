use std::collections::BTreeSet;

use evidence_search_contracts::{AuditEvent, AuditEventDraft, ContractError, Validate};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Append-only in-memory audit ledger. Persistence adapters write these events as JSONL.
#[derive(Debug, Clone, Default)]
pub struct AuditLedger {
    events: Vec<AuditEvent>,
}

/// Result of verifying an audit chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditVerification {
    /// Number of verified events.
    pub event_count: usize,
    /// Final event hash, when the ledger is non-empty.
    pub head_hash: Option<String>,
}

impl AuditLedger {
    /// Create an empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load existing events without assuming they are valid.
    #[must_use]
    pub fn from_events(events: Vec<AuditEvent>) -> Self {
        Self { events }
    }

    /// Borrow all events in order.
    #[must_use]
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Append an event and assign its previous/event hashes.
    pub fn append(&mut self, draft: AuditEventDraft) -> Result<&AuditEvent, AuditError> {
        validate_draft(&draft)?;
        if self
            .events
            .iter()
            .any(|event| event.event_id == draft.event_id)
        {
            return Err(AuditError::DuplicateEventId(draft.event_id));
        }
        if let Some(first) = self.events.first()
            && first.review_id != draft.review_id
        {
            return Err(AuditError::ReviewMismatch {
                expected: first.review_id.clone(),
                actual: draft.review_id,
            });
        }
        let previous_hash = self
            .events
            .last()
            .map_or_else(|| "GENESIS".to_owned(), |event| event.event_hash.clone());
        let event_hash = hash_draft(&draft, &previous_hash)?;
        self.events.push(AuditEvent {
            schema_version: draft.schema_version,
            event_id: draft.event_id,
            review_id: draft.review_id,
            event_type: draft.event_type,
            occurred_at: draft.occurred_at,
            actor: draft.actor,
            payload: draft.payload,
            previous_hash,
            event_hash,
        });
        self.events.last().ok_or(AuditError::InternalAppend)
    }

    /// Verify the complete hash chain.
    pub fn verify(&self) -> Result<AuditVerification, AuditError> {
        let mut expected_previous = "GENESIS".to_owned();
        let mut event_ids = BTreeSet::new();
        let expected_review = self.events.first().map(|event| event.review_id.as_str());
        for (index, event) in self.events.iter().enumerate() {
            event.validate()?;
            validate_timestamp(&event.occurred_at)?;
            if !event_ids.insert(event.event_id.as_str()) {
                return Err(AuditError::DuplicateEventId(event.event_id.clone()));
            }
            if let Some(review_id) = expected_review
                && review_id != event.review_id.as_str()
            {
                return Err(AuditError::ReviewMismatch {
                    expected: review_id.to_owned(),
                    actual: event.review_id.clone(),
                });
            }
            validate_hash(&event.event_hash, "event_hash")?;
            if event.previous_hash != "GENESIS" {
                validate_hash(&event.previous_hash, "previous_hash")?;
            }
            if event.previous_hash != expected_previous {
                return Err(AuditError::PreviousHashMismatch { index });
            }
            let draft = AuditEventDraft {
                schema_version: event.schema_version.clone(),
                event_id: event.event_id.clone(),
                review_id: event.review_id.clone(),
                event_type: event.event_type.clone(),
                occurred_at: event.occurred_at.clone(),
                actor: event.actor.clone(),
                payload: event.payload.clone(),
            };
            let calculated = hash_draft(&draft, &event.previous_hash)?;
            if calculated != event.event_hash {
                return Err(AuditError::EventHashMismatch { index });
            }
            expected_previous.clone_from(&event.event_hash);
        }
        Ok(AuditVerification {
            event_count: self.events.len(),
            head_hash: self.events.last().map(|event| event.event_hash.clone()),
        })
    }
}

/// Audit-chain error.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    /// Audit contract validation failed.
    #[error(transparent)]
    Contract(#[from] ContractError),
    /// JSON serialisation failed.
    #[error("could not canonicalise audit event: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Timestamp was not valid RFC 3339.
    #[error("audit timestamp is not valid RFC 3339: {0}")]
    InvalidTimestamp(#[from] time::error::Parse),
    /// A hash was not a canonical BLAKE3 hexadecimal digest.
    #[error("audit `{field}` is not a canonical BLAKE3 digest")]
    InvalidHash { field: &'static str },
    /// An event identifier appeared more than once.
    #[error("audit event identifier `{0}` is duplicated")]
    DuplicateEventId(String),
    /// Events from different reviews were mixed in one ledger.
    #[error("audit review identifier mismatch: expected `{expected}`, found `{actual}`")]
    ReviewMismatch { expected: String, actual: String },
    /// Previous hash is inconsistent.
    #[error("audit event {index} does not point to the previous event")]
    PreviousHashMismatch { index: usize },
    /// Event content does not match its stored hash.
    #[error("audit event {index} hash does not match canonical content")]
    EventHashMismatch { index: usize },
    /// Append succeeded but the event could not be borrowed.
    #[error("internal append error")]
    InternalAppend,
}

fn validate_draft(draft: &AuditEventDraft) -> Result<(), AuditError> {
    draft.validate()?;
    validate_timestamp(&draft.occurred_at)
}

fn validate_timestamp(timestamp: &str) -> Result<(), AuditError> {
    time::OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339)?;
    Ok(())
}

fn validate_hash(hash: &str, field: &'static str) -> Result<(), AuditError> {
    if hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(AuditError::InvalidHash { field })
    }
}

fn hash_draft(draft: &AuditEventDraft, previous_hash: &str) -> Result<String, AuditError> {
    let mut value = serde_json::to_value(draft)?;
    if let Value::Object(object) = &mut value {
        object.insert(
            "previous_hash".to_owned(),
            Value::String(previous_hash.to_owned()),
        );
    }
    let canonical = canonical_json(&value);
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Convert a JSON value into recursively key-sorted canonical form.
#[must_use]
pub(crate) fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonical_json).collect()),
        Value::Object(object) => {
            let mut pairs: Vec<_> = object.iter().collect();
            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
            let mut sorted = Map::new();
            for (key, value) in pairs {
                sorted.insert(key.clone(), canonical_json(value));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use evidence_search_contracts::{Actor, AuditEventDraft};
    use serde_json::json;

    use super::*;

    fn draft(id: &str) -> AuditEventDraft {
        AuditEventDraft {
            schema_version: "org.searchright.audit-event.v1".to_owned(),
            event_id: id.to_owned(),
            review_id: "review-1".to_owned(),
            event_type: "example".to_owned(),
            occurred_at: "2026-08-05T00:00:00Z".to_owned(),
            actor: Actor {
                actor_id: "tester".to_owned(),
                actor_type: "human".to_owned(),
                provenance: None,
            },
            payload: json!({"z": 1, "a": {"d": 2, "b": 1}}),
        }
    }

    #[test]
    fn append_and_verify_round_trip() {
        let mut ledger = AuditLedger::new();
        assert!(ledger.append(draft("event-1")).is_ok());
        assert!(ledger.append(draft("event-2")).is_ok());
        let verification = ledger.verify();
        assert!(verification.is_ok());
        if let Ok(verification) = verification {
            assert_eq!(verification.event_count, 2);
            assert!(verification.head_hash.is_some());
        }
    }

    #[test]
    fn tampering_is_detected() {
        let mut ledger = AuditLedger::new();
        assert!(ledger.append(draft("event-1")).is_ok());
        let mut events = ledger.events().to_vec();
        if let Some(event) = events.first_mut() {
            event.payload = json!({"tampered": true});
        }
        let tampered = AuditLedger::from_events(events);
        assert!(matches!(
            tampered.verify(),
            Err(AuditError::EventHashMismatch { index: 0 })
        ));
    }
}
