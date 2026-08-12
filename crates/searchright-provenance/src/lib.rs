//! Interoperable provenance exports.

#![forbid(unsafe_code)]

use schemars::JsonSchema;
use searchright_contracts::{AuditEvent, ReviewPlan, SourceReceipt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One serialisable provenance bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProvenanceBundle {
    /// RO-Crate 1.3 JSON-LD metadata.
    pub ro_crate: Value,
    /// W3C PROV-compatible JSON representation.
    pub prov: Value,
}

/// Build an RO-Crate 1.3 and PROV bundle from review, receipt and audit evidence.
pub fn build_bundle(
    plan: &ReviewPlan,
    receipts: &[SourceReceipt],
    events: &[AuditEvent],
) -> Result<ProvenanceBundle, ProvenanceError> {
    validate_inputs(plan, receipts, events)?;
    let ro_crate = build_ro_crate(plan, receipts, events)?;
    let prov = build_prov(plan, receipts, events)?;
    Ok(ProvenanceBundle { ro_crate, prov })
}

/// Build an RO-Crate 1.3 metadata document.
pub fn build_ro_crate(
    plan: &ReviewPlan,
    receipts: &[SourceReceipt],
    events: &[AuditEvent],
) -> Result<Value, ProvenanceError> {
    validate_inputs(plan, receipts, events)?;
    let mut receipts = receipts.iter().collect::<Vec<_>>();
    receipts.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
    let mut events = events.iter().collect::<Vec<_>>();
    events.sort_by(|left, right| left.event_id.cmp(&right.event_id));
    let plan_json = serde_json::to_value(plan)?;
    let receipt_entities: Vec<Value> = receipts
        .iter()
        .map(|receipt| {
            json!({
                "@id": format!("urn:searchright:receipt:{}", receipt.receipt_id),
                "@type": "Dataset",
                "name": format!("Search receipt for {}", receipt.source_label),
                "identifier": receipt.receipt_id,
                "dateCreated": receipt.executed_at,
                "contentSize": receipt.records_retrieved,
                "encodingFormat": "application/json",
                "provider": {"@id": format!("urn:searchright:provider:{}", receipt.provider_id)},
                "isPartOf": {"@id": "./"}
            })
        })
        .collect();
    let provider_entities: Vec<Value> = receipts
        .iter()
        .map(|receipt| {
            json!({
                "@id": format!("urn:searchright:provider:{}", receipt.provider_id),
                "@type": "SoftwareApplication",
                "name": receipt.provider_id
            })
        })
        .collect();
    let event_entities: Vec<Value> = events
        .iter()
        .map(|event| {
            json!({
                "@id": format!("urn:searchright:audit:{}", event.event_id),
                "@type": "CreateAction",
                "name": event.event_type,
                "startTime": event.occurred_at,
                "agent": {"@id": format!("urn:searchright:actor:{}", event.actor.actor_id)},
                "result": {"@id": format!("urn:searchright:review:{}", event.review_id)}
            })
        })
        .collect();
    let actor_entities: Vec<Value> = events
        .iter()
        .map(|event| {
            json!({
                "@id": format!("urn:searchright:actor:{}", event.actor.actor_id),
                "@type": if event.actor.actor_type == "human" { "Person" } else { "SoftwareApplication" },
                "name": event.actor.actor_id,
                "description": event.actor.provenance
            })
        })
        .collect();

    let mut graph = vec![
        json!({
            "@id": "ro-crate-metadata.json",
            "@type": "CreativeWork",
            "about": {"@id": "./"},
            "conformsTo": {"@id": "https://w3id.org/ro/crate/1.3"}
        }),
        json!({
            "@id": "./",
            "@type": "Dataset",
            "name": plan.title,
            "identifier": plan.review_id,
            "description": "Reproducible systematic-search workspace",
            "hasPart": receipts.iter().map(|receipt| json!({"@id": format!("urn:searchright:receipt:{}", receipt.receipt_id)})).collect::<Vec<_>>(),
            "mainEntity": {"@id": format!("urn:searchright:review:{}", plan.review_id)}
        }),
        json!({
            "@id": format!("urn:searchright:review:{}", plan.review_id),
            "@type": "ScholarlyArticle",
            "name": plan.title,
            "additionalProperty": plan_json
        }),
    ];
    graph.extend(receipt_entities);
    graph.extend(provider_entities);
    graph.extend(event_entities);
    graph.extend(actor_entities);
    deduplicate_graph(&mut graph);

    Ok(json!({
        "@context": "https://w3id.org/ro/crate/1.3/context",
        "@graph": graph
    }))
}

/// Build a compact W3C PROV-compatible JSON document.
pub fn build_prov(
    plan: &ReviewPlan,
    receipts: &[SourceReceipt],
    events: &[AuditEvent],
) -> Result<Value, ProvenanceError> {
    validate_inputs(plan, receipts, events)?;
    let entities = receipts
        .iter()
        .map(|receipt| {
            (
                format!("sr:receipt-{}", receipt.receipt_id),
                json!({
                    "prov:type": "sr:SourceReceipt",
                    "sr:provider": receipt.provider_id,
                    "sr:recordsRetrieved": receipt.records_retrieved,
                    "prov:generatedAtTime": receipt.executed_at
                }),
            )
        })
        .chain(std::iter::once((
            format!("sr:review-{}", plan.review_id),
            json!({"prov:type": "sr:Review", "sr:title": plan.title}),
        )))
        .collect::<serde_json::Map<String, Value>>();
    let activities = events
        .iter()
        .map(|event| {
            (
                format!("sr:event-{}", event.event_id),
                json!({
                    "prov:type": event.event_type,
                    "prov:startedAtTime": event.occurred_at
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();
    let agents = events
        .iter()
        .map(|event| {
            (
                format!("sr:actor-{}", event.actor.actor_id),
                json!({
                    "prov:type": event.actor.actor_type,
                    "sr:provenance": event.actor.provenance
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();
    let associations = events
        .iter()
        .map(|event| {
            (
                format!("sr:association-{}", event.event_id),
                json!({
                    "prov:activity": format!("sr:event-{}", event.event_id),
                    "prov:agent": format!("sr:actor-{}", event.actor.actor_id)
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();
    let event_ids_by_hash = events
        .iter()
        .map(|event| (event.event_hash.as_str(), event.event_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let predecessors = events
        .iter()
        .filter_map(|event| {
            event_ids_by_hash
                .get(event.previous_hash.as_str())
                .map(|previous_id| {
                    (
                        format!("sr:derivation-{}", event.event_id),
                        json!({
                            "prov:generatedEntity": format!("sr:event-{}", event.event_id),
                            "prov:usedEntity": format!("sr:event-{previous_id}")
                        }),
                    )
                })
        })
        .collect::<serde_json::Map<String, Value>>();

    Ok(json!({
        "prefix": {
            "prov": "http://www.w3.org/ns/prov#",
            "sr": "https://schemas.searchright.dev/prov/"
        },
        "entity": entities,
        "activity": activities,
        "agent": agents,
        "wasAssociatedWith": associations,
        "wasDerivedFrom": predecessors
    }))
}

fn validate_inputs(
    plan: &ReviewPlan,
    receipts: &[SourceReceipt],
    events: &[AuditEvent],
) -> Result<(), ProvenanceError> {
    let mut receipt_ids = BTreeSet::new();
    for receipt in receipts {
        if receipt.review_id != plan.review_id {
            return Err(ProvenanceError::ReviewMismatch {
                kind: "receipt",
                identifier: receipt.receipt_id.clone(),
            });
        }
        if !receipt_ids.insert(receipt.receipt_id.as_str()) {
            return Err(ProvenanceError::DuplicateIdentifier {
                kind: "receipt",
                identifier: receipt.receipt_id.clone(),
            });
        }
    }

    let mut event_ids = BTreeSet::new();
    let mut event_hashes = BTreeSet::new();
    for event in events {
        if event.review_id != plan.review_id {
            return Err(ProvenanceError::ReviewMismatch {
                kind: "audit event",
                identifier: event.event_id.clone(),
            });
        }
        if !event_ids.insert(event.event_id.as_str()) {
            return Err(ProvenanceError::DuplicateIdentifier {
                kind: "audit event",
                identifier: event.event_id.clone(),
            });
        }
        if !event_hashes.insert(event.event_hash.as_str()) {
            return Err(ProvenanceError::DuplicateIdentifier {
                kind: "audit event hash",
                identifier: event.event_hash.clone(),
            });
        }
    }
    for event in events {
        if event.previous_hash != "GENESIS" && !event_hashes.contains(event.previous_hash.as_str())
        {
            return Err(ProvenanceError::UnresolvedPredecessor {
                event_id: event.event_id.clone(),
            });
        }
    }
    Ok(())
}

fn deduplicate_graph(graph: &mut Vec<Value>) {
    let mut seen = BTreeSet::new();
    graph.retain(|entity| {
        entity
            .get("@id")
            .and_then(Value::as_str)
            .is_none_or(|identifier| seen.insert(identifier.to_owned()))
    });
}

/// Provenance-export error.
#[derive(Debug, thiserror::Error)]
pub enum ProvenanceError {
    /// JSON conversion failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// An input belongs to another review.
    #[error("{kind} `{identifier}` belongs to a different review")]
    ReviewMismatch {
        /// Input kind.
        kind: &'static str,
        /// Stable input identifier.
        identifier: String,
    },
    /// A stable identifier is ambiguous within an export.
    #[error("duplicate {kind} identifier `{identifier}`")]
    DuplicateIdentifier {
        /// Input kind.
        kind: &'static str,
        /// Duplicated identifier.
        identifier: String,
    },
    /// An audit predecessor cannot be resolved within this export.
    #[error("audit event `{event_id}` has an unresolved predecessor")]
    UnresolvedPredecessor {
        /// Event with the invalid predecessor.
        event_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer<'a>(document: &'a Value, path: &str) -> &'a Value {
        document
            .pointer(path)
            .unwrap_or_else(|| panic!("fixture output must contain JSON pointer {path}"))
    }

    fn fixture() -> (ReviewPlan, SourceReceipt, AuditEvent) {
        let plan =
            serde_yaml::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))
                .unwrap_or_else(|error| panic!("review-plan fixture must deserialize: {error}"));
        let mut receipt: SourceReceipt = serde_yaml::from_str(include_str!(
            "../../../contracts/examples/source-receipt.yaml"
        ))
        .unwrap_or_else(|error| panic!("source-receipt fixture must deserialize: {error}"));
        receipt.review_id = "demo-paediatric-metabolic-search".to_owned();
        let event =
            serde_json::from_str(include_str!("../../../contracts/examples/audit-event.json"))
                .unwrap_or_else(|error| panic!("audit-event fixture must deserialize: {error}"));
        (plan, receipt, event)
    }

    #[test]
    fn emits_deterministic_ro_crate_and_prov_documents() {
        let (plan, receipt, event) = fixture();
        let bundle = build_bundle(&plan, &[receipt], &[event])
            .unwrap_or_else(|error| panic!("valid provenance must export: {error}"));

        assert_eq!(
            pointer(&bundle.ro_crate, "/@context"),
            "https://w3id.org/ro/crate/1.3/context"
        );
        assert_eq!(
            pointer(&bundle.prov, "/prefix/prov"),
            "http://www.w3.org/ns/prov#"
        );
        assert!(pointer(&bundle.prov, "/entity/sr:receipt-receipt-demo-1").is_object());
    }

    #[test]
    fn preserves_resolvable_audit_predecessors() {
        let (plan, receipt, first) = fixture();
        let mut second = first.clone();
        second.event_id = "event-002".to_owned();
        second.previous_hash = first.event_hash.clone();
        second.event_hash =
            "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_owned();

        let prov = build_prov(&plan, &[receipt], &[first, second])
            .unwrap_or_else(|error| panic!("linked events must export: {error}"));
        assert_eq!(
            pointer(
                &prov,
                "/wasDerivedFrom/sr:derivation-event-002/prov:usedEntity"
            ),
            "sr:event-event-001"
        );
    }

    #[test]
    fn rejects_cross_review_and_ambiguous_inputs() {
        let (plan, mut receipt, event) = fixture();
        receipt.review_id = "another-review".to_owned();
        assert!(matches!(
            build_bundle(&plan, &[receipt], std::slice::from_ref(&event)),
            Err(ProvenanceError::ReviewMismatch {
                kind: "receipt",
                ..
            })
        ));

        assert!(matches!(
            build_bundle(&plan, &[], &[event.clone(), event]),
            Err(ProvenanceError::DuplicateIdentifier {
                kind: "audit event",
                ..
            })
        ));
    }

    #[test]
    fn rejects_unresolved_audit_predecessors() {
        let (plan, receipt, mut event) = fixture();
        event.previous_hash = "missing-hash".to_owned();
        assert!(matches!(
            build_bundle(&plan, &[receipt], &[event]),
            Err(ProvenanceError::UnresolvedPredecessor { .. })
        ));
    }
}
