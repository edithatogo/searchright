//! Fail-closed audit-event ingestion policy mirrored by the event registry.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::{AuditEvent, ContractError};

const MAXIMUM_PAYLOAD_BYTES: usize = 16_384;
const PROHIBITED_KEYS: [&str; 5] = ["credential", "password", "secret", "token", "full_text"];

/// Validate an audit event against the compiled ingestion registry before persistence.
pub fn validate_registered_audit_event(event: &AuditEvent) -> Result<(), ContractError> {
    let payload = event.payload.as_object().ok_or_else(|| {
        ContractError::Invariant("audit payload must be a JSON object".to_owned())
    })?;
    let bytes = serde_json::to_vec(&event.payload).map_err(|error| {
        ContractError::Invariant(format!("audit payload is not serializable: {error}"))
    })?;
    if bytes.len() > MAXIMUM_PAYLOAD_BYTES {
        return Err(ContractError::Invariant(
            "audit payload exceeds the 16384-byte ingestion limit".to_owned(),
        ));
    }
    reject_prohibited_keys(&event.payload)?;

    let (allowed, versions): (&[&str], &[u64]) = match event.event_type.as_str() {
        "protocol_amended" => (&["_schema_version", "amendment_id"], &[1]),
        "review_plan_validated" => (&["_schema_version", "contract_version", "plan_id"], &[1]),
        "review_status_changed" => (&["_schema_version", "status"], &[1]),
        "screening_decision_recorded" => (
            &[
                "_schema_version",
                "decision",
                "final_authority",
                "record_id",
                "reviewer_id",
                "stage",
            ],
            &[1],
        ),
        "search_run_completed" => (
            &[
                "_schema_version",
                "provider",
                "record_count",
                "run_id",
                "source_id",
            ],
            &[0, 1],
        ),
        "execution_committed" => (
            &["_schema_version", "commit_id", "receipt_id", "record_count"],
            &[1],
        ),
        other => {
            return Err(ContractError::Invariant(format!(
                "unregistered audit event type `{other}`"
            )));
        }
    };
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if let Some(key) = payload.keys().find(|key| !allowed.contains(key.as_str())) {
        return Err(ContractError::Invariant(format!(
            "audit payload key `{key}` is not registered for {}",
            event.event_type
        )));
    }
    let version = payload
        .get("_schema_version")
        .map_or(Some(1), Value::as_u64)
        .ok_or_else(|| {
            ContractError::Invariant("audit payload version must be an integer".to_owned())
        })?;
    if !versions.contains(&version) {
        return Err(ContractError::Invariant(format!(
            "unsupported audit payload version {version} for {}",
            event.event_type
        )));
    }
    if event.event_type == "execution_committed" {
        for key in ["commit_id", "receipt_id", "record_count"] {
            if !payload.contains_key(key) {
                return Err(ContractError::Invariant(format!(
                    "audit payload key `{key}` is required"
                )));
            }
        }
    }
    for (key, value) in payload {
        if key == "_schema_version" {
            continue;
        }
        let valid = if key == "record_count" {
            value.as_u64().is_some()
        } else {
            value.as_str().is_some_and(|text| {
                !text.trim().is_empty() && text.len() <= 512 && !text.chars().any(char::is_control)
            })
        };
        if !valid {
            return Err(ContractError::Invariant(format!(
                "audit payload key `{key}` has an invalid type or value"
            )));
        }
    }
    Ok(())
}

fn reject_prohibited_keys(value: &Value) -> Result<(), ContractError> {
    match value {
        Value::Object(values) => {
            for (key, nested) in values {
                if PROHIBITED_KEYS.contains(&key.to_ascii_lowercase().as_str()) {
                    return Err(ContractError::Invariant(format!(
                        "audit payload key `{key}` is prohibited"
                    )));
                }
                reject_prohibited_keys(nested)?;
            }
        }
        Value::Array(values) => {
            for nested in values {
                reject_prohibited_keys(nested)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{AUDIT_EVENT_SCHEMA_VERSION, Actor};

    fn event(event_type: &str, payload: Value) -> AuditEvent {
        AuditEvent {
            schema_version: AUDIT_EVENT_SCHEMA_VERSION.to_owned(),
            event_id: "event-1".to_owned(),
            review_id: "review-1".to_owned(),
            event_type: event_type.to_owned(),
            occurred_at: "2026-08-13T00:00:00Z".to_owned(),
            actor: Actor {
                actor_id: "actor-1".to_owned(),
                actor_type: "human".to_owned(),
                provenance: None,
            },
            payload,
            previous_hash: "GENESIS".to_owned(),
            event_hash: "0".repeat(64),
        }
    }

    #[test]
    fn registered_minimal_payload_is_admitted() {
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"plan_id": "plan-1"}),
            ))
            .is_ok()
        );
    }

    #[test]
    fn unknown_types_fields_and_nested_secrets_fail_closed() {
        assert!(validate_registered_audit_event(&event("unknown", json!({}))).is_err());
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"plan_id": "plan-1", "extra": true}),
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"plan_id": "plan-1", "nested": {"token": "secret"}}),
            ))
            .is_err()
        );
    }

    #[test]
    fn every_registered_type_and_version_is_exercised() {
        let cases = [
            (
                "protocol_amended",
                json!({"_schema_version": 1, "amendment_id": "a"}),
            ),
            (
                "review_plan_validated",
                json!({"_schema_version": 1, "contract_version": "1", "plan_id": "p"}),
            ),
            (
                "review_status_changed",
                json!({"_schema_version": 1, "status": "active"}),
            ),
            (
                "screening_decision_recorded",
                json!({"_schema_version": 1, "decision": "include", "final_authority": "human", "record_id": "r", "reviewer_id": "u", "stage": "title"}),
            ),
            (
                "search_run_completed",
                json!({"_schema_version": 0, "provider": "fixture", "record_count": 0, "run_id": "run", "source_id": "source"}),
            ),
            (
                "execution_committed",
                json!({"_schema_version": 1, "commit_id": "c", "receipt_id": "receipt", "record_count": 0}),
            ),
        ];
        for (kind, payload) in cases {
            assert!(validate_registered_audit_event(&event(kind, payload)).is_ok());
        }
    }

    #[test]
    fn malformed_versions_sizes_and_prohibited_arrays_fail_closed() {
        assert!(
            validate_registered_audit_event(&event("review_plan_validated", json!([]))).is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"_schema_version": "one", "plan_id": "p"})
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "execution_committed",
                json!({"commit_id": "c", "receipt_id": "r"})
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "execution_committed",
                json!({"commit_id": "c", "receipt_id": "r", "record_count": "zero"})
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"_schema_version": 2, "plan_id": "p"})
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"plan_id": [ {"Password": "x"} ]})
            ))
            .is_err()
        );
        assert!(
            validate_registered_audit_event(&event(
                "review_plan_validated",
                json!({"plan_id": "x".repeat(MAXIMUM_PAYLOAD_BYTES)})
            ))
            .is_err()
        );
    }

    #[test]
    fn compiled_registry_matches_canonical_json_registry() {
        let registry: Value =
            serde_json::from_str(include_str!("../../../contracts/events/registry.json"))
                .unwrap_or(Value::Null);
        let observed = registry
            .get("event_types")
            .and_then(Value::as_array)
            .map(|events| {
                events
                    .iter()
                    .filter_map(|entry| {
                        Some((
                            entry.get("event_type")?.as_str()?.to_owned(),
                            entry
                                .get("allowed_payload_keys")?
                                .as_array()?
                                .iter()
                                .filter_map(Value::as_str)
                                .map(ToOwned::to_owned)
                                .collect::<BTreeSet<_>>(),
                            entry
                                .get("versions")?
                                .as_array()?
                                .iter()
                                .filter_map(|version| version.get("version")?.as_u64())
                                .collect::<BTreeSet<_>>(),
                        ))
                    })
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let expected = [
            (
                "execution_committed",
                &["_schema_version", "commit_id", "receipt_id", "record_count"][..],
                &[1][..],
            ),
            (
                "protocol_amended",
                &["_schema_version", "amendment_id"][..],
                &[1][..],
            ),
            (
                "review_plan_validated",
                &["_schema_version", "contract_version", "plan_id"][..],
                &[1][..],
            ),
            (
                "review_status_changed",
                &["_schema_version", "status"][..],
                &[1][..],
            ),
            (
                "screening_decision_recorded",
                &[
                    "_schema_version",
                    "decision",
                    "final_authority",
                    "record_id",
                    "reviewer_id",
                    "stage",
                ][..],
                &[1][..],
            ),
            (
                "search_run_completed",
                &[
                    "_schema_version",
                    "provider",
                    "record_count",
                    "run_id",
                    "source_id",
                ][..],
                &[0, 1][..],
            ),
        ]
        .into_iter()
        .map(|(kind, keys, versions)| {
            (
                kind.to_owned(),
                keys.iter().map(|value| (*value).to_owned()).collect(),
                versions.iter().copied().collect(),
            )
        })
        .collect::<BTreeSet<_>>();
        assert_eq!(observed, expected);
    }
}
