//! Interoperable provenance exports.

#![forbid(unsafe_code)]

use searchright_contracts::{AuditEvent, ReviewPlan, SourceReceipt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// One serialisable provenance bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    let ro_crate = build_ro_crate(plan, receipts, events)?;
    let prov = build_prov(plan, receipts, events);
    Ok(ProvenanceBundle { ro_crate, prov })
}

/// Build an RO-Crate 1.3 metadata document.
pub fn build_ro_crate(
    plan: &ReviewPlan,
    receipts: &[SourceReceipt],
    events: &[AuditEvent],
) -> Result<Value, ProvenanceError> {
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
#[must_use]
pub fn build_prov(plan: &ReviewPlan, receipts: &[SourceReceipt], events: &[AuditEvent]) -> Value {
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

    json!({
        "prefix": {
            "prov": "http://www.w3.org/ns/prov#",
            "sr": "https://schemas.searchright.dev/prov/"
        },
        "entity": entities,
        "activity": activities,
        "agent": agents,
        "wasAssociatedWith": associations
    })
}

fn deduplicate_graph(graph: &mut Vec<Value>) {
    let mut seen = std::collections::BTreeSet::new();
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
}
