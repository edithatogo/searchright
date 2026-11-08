//! Edge-case tests for RO-Crate / PROV export.

use searchright_contracts::{
    AUDIT_EVENT_SCHEMA_VERSION, Actor, AuditEvent, ExecutionPolicy, ReviewPlan,
    SOURCE_RECEIPT_SCHEMA_VERSION, SourceReceipt,
};
use searchright_provenance::{ProvenanceError, build_bundle, build_prov, build_ro_crate};

fn plan() -> ReviewPlan {
    noyalib::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))
        .unwrap_or_else(|error| panic!("review-plan fixture must deserialize: {error}"))
}

const fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        live_enabled: true,
        max_records: 100,
        max_pages: 5,
        timeout_seconds: 30,
        total_timeout_seconds: Some(300),
        max_retries: 2,
        min_interval_ms: 350,
        retry_base_delay_ms: Some(350),
        retry_max_delay_ms: Some(5600),
        max_response_bytes: Some(16_777_216),
        replay_enabled: true,
        cache_write_enabled: false,
    }
}

fn receipt(id: &str, review_id: &str, provider: &str) -> SourceReceipt {
    SourceReceipt {
        schema_version: SOURCE_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: id.to_string(),
        review_id: review_id.to_string(),
        run_id: "run-1".to_string(),
        provider_id: provider.to_string(),
        source_label: "Synthetic source".to_string(),
        strategy_id: "strategy-1".to_string(),
        query_hash: "a".repeat(64),
        executed_at: "2026-08-06T00:00:00Z".to_string(),
        records_retrieved: 10,
        pages_retrieved: 1,
        execution_mode: "fixture".to_string(),
        endpoint: Some("example.test".to_string()),
        policy: policy(),
        provider_version: "fixture/1".to_string(),
        compiler_version: "compiler/1".to_string(),
        result_digest: "b".repeat(64),
        cache_hits: 0,
        cache_writes: 0,
        warnings: vec![],
    }
}

fn event(
    id: &str,
    review_id: &str,
    previous_hash: &str,
    hash: &str,
    actor_type: &str,
) -> AuditEvent {
    AuditEvent {
        schema_version: AUDIT_EVENT_SCHEMA_VERSION.to_string(),
        event_id: id.to_string(),
        review_id: review_id.to_string(),
        event_type: "review.validated".to_string(),
        occurred_at: "2026-08-06T00:00:00Z".to_string(),
        actor: Actor {
            actor_id: "actor-1".to_string(),
            actor_type: actor_type.to_string(),
            provenance: Some("test".to_string()),
        },
        payload: serde_json::Value::Null,
        previous_hash: previous_hash.to_string(),
        event_hash: hash.to_string(),
    }
}

#[test]
fn empty_receipts_and_events_still_build_bundle() {
    let p = plan();
    let bundle = build_bundle(&p, &[], &[]).unwrap_or_else(|_| panic!("expected bundle"));
    let graph = bundle
        .ro_crate
        .pointer("/@graph")
        .unwrap_or_else(|| panic!("expected @graph"));
    assert!(graph.is_array());
    // Base graph has at least the 3 fixed entities.
    assert!(
        graph
            .as_array()
            .unwrap_or_else(|| panic!("expected array"))
            .len()
            >= 3
    );
}

#[test]
fn output_is_deterministic_regardless_of_input_order() {
    let p = plan();
    let r1 = receipt("receipt-a", &p.review_id, "pubmed");
    let r2 = receipt("receipt-b", &p.review_id, "pubmed");
    let e1 = event(
        "event-001",
        &p.review_id,
        "GENESIS",
        &"a".repeat(64),
        "human",
    );
    let mut e2 = event(
        "event-002",
        &p.review_id,
        &"a".repeat(64),
        &"b".repeat(64),
        "human",
    );
    e2.event_id = "event-002".to_string();
    let first = build_bundle(&p, &[r1.clone(), r2.clone()], &[e1.clone(), e2.clone()])
        .unwrap_or_else(|_| panic!("expected bundle"));
    let second =
        build_bundle(&p, &[r2, r1], &[e2, e1]).unwrap_or_else(|_| panic!("expected bundle"));
    assert_eq!(first, second);
}

#[test]
fn duplicate_receipt_id_is_rejected() {
    let p = plan();
    let r = receipt("receipt-1", &p.review_id, "pubmed");
    assert!(matches!(
        build_bundle(&p, &[r.clone(), r], &[]),
        Err(ProvenanceError::DuplicateIdentifier {
            kind: "receipt",
            ..
        })
    ));
}

#[test]
fn duplicate_event_hash_is_rejected_even_with_distinct_ids() {
    let p = plan();
    let hash = "a".repeat(64);
    let e1 = event("event-1", &p.review_id, "GENESIS", &hash, "human");
    let e2 = event("event-2", &p.review_id, "GENESIS", &hash, "human");
    assert!(matches!(
        build_bundle(&p, &[], &[e1, e2]),
        Err(ProvenanceError::DuplicateIdentifier {
            kind: "audit event hash",
            ..
        })
    ));
}

#[test]
fn cross_review_event_is_rejected() {
    let p = plan();
    let e = event(
        "event-1",
        "another-review",
        "GENESIS",
        &"a".repeat(64),
        "human",
    );
    assert!(matches!(
        build_bundle(&p, &[], &[e]),
        Err(ProvenanceError::ReviewMismatch {
            kind: "audit event",
            ..
        })
    ));
}

#[test]
fn human_actor_exports_as_person() {
    let p = plan();
    let e = event("event-1", &p.review_id, "GENESIS", &"a".repeat(64), "human");
    let ro_crate = build_ro_crate(&p, &[], &[e]).unwrap_or_else(|_| panic!("expected ro_crate"));
    let graph = ro_crate
        .pointer("/@graph")
        .unwrap_or_else(|| panic!("expected @graph"))
        .as_array()
        .unwrap_or_else(|| panic!("expected array"));
    let actor_entity = graph
        .iter()
        .find(|v| {
            v.pointer("/@id")
                .and_then(|x| x.as_str())
                .is_some_and(|id| id.contains("actor-1"))
        })
        .unwrap_or_else(|| panic!("expected actor entity"));
    assert_eq!(
        actor_entity
            .pointer("/@type")
            .unwrap_or_else(|| panic!("expected @type")),
        "Person"
    );
}

#[test]
fn agent_actor_exports_as_software_application() {
    let p = plan();
    let e = event("event-1", &p.review_id, "GENESIS", &"a".repeat(64), "agent");
    let ro_crate = build_ro_crate(&p, &[], &[e]).unwrap_or_else(|_| panic!("expected ro_crate"));
    let graph = ro_crate
        .pointer("/@graph")
        .unwrap_or_else(|| panic!("expected @graph"))
        .as_array()
        .unwrap_or_else(|| panic!("expected array"));
    let actor_entity = graph
        .iter()
        .find(|v| {
            v.pointer("/@id")
                .and_then(|x| x.as_str())
                .is_some_and(|id| id.contains("actor-1"))
        })
        .unwrap_or_else(|| panic!("expected actor entity"));
    assert_eq!(
        actor_entity
            .pointer("/@type")
            .unwrap_or_else(|| panic!("expected @type")),
        "SoftwareApplication"
    );
}

#[test]
fn same_provider_receipts_deduplicate_provider_entity() {
    let p = plan();
    let receipts = vec![
        receipt("receipt-a", &p.review_id, "pubmed"),
        receipt("receipt-b", &p.review_id, "pubmed"),
    ];
    let ro_crate =
        build_ro_crate(&p, &receipts, &[]).unwrap_or_else(|_| panic!("expected ro_crate"));
    let graph = ro_crate
        .pointer("/@graph")
        .unwrap_or_else(|| panic!("expected @graph"))
        .as_array()
        .unwrap_or_else(|| panic!("expected array"));
    let provider_count = graph
        .iter()
        .filter(|v| {
            v.pointer("/@id")
                .and_then(|x| x.as_str())
                .is_some_and(|id| id == "urn:searchright:provider:pubmed")
        })
        .count();
    assert_eq!(provider_count, 1);
    // Two distinct receipts remain two entities.
    let receipt_count = graph
        .iter()
        .filter(|v| {
            v.pointer("/@id")
                .and_then(|x| x.as_str())
                .is_some_and(|id| id.starts_with("urn:searchright:receipt:"))
        })
        .count();
    assert_eq!(receipt_count, 2);
}

#[test]
fn three_event_chain_derives_stepwise() {
    let p = plan();
    let h1 = "a".repeat(64);
    let h2 = "b".repeat(64);
    let h3 = "c".repeat(64);
    let e1 = event("event-001", &p.review_id, "GENESIS", &h1, "human");
    let e2 = event("event-002", &p.review_id, &h1, &h2, "human");
    let e3 = event("event-003", &p.review_id, &h2, &h3, "human");
    let prov = build_prov(&p, &[], &[e1, e2, e3]).unwrap_or_else(|_| panic!("expected prov"));
    assert_eq!(
        prov.pointer("/wasDerivedFrom/sr:derivation-event-002/prov:usedEntity")
            .unwrap_or_else(|| panic!("expected derivation")),
        "sr:event-event-001"
    );
    assert_eq!(
        prov.pointer("/wasDerivedFrom/sr:derivation-event-003/prov:usedEntity")
            .unwrap_or_else(|| panic!("expected derivation")),
        "sr:event-event-002"
    );
}

#[test]
fn receipt_mismatch_takes_precedence_over_event_mismatch() {
    let p = plan();
    let bad_receipt = receipt("receipt-1", "another-review", "pubmed");
    let bad_event = event(
        "event-1",
        "another-review",
        "GENESIS",
        &"a".repeat(64),
        "human",
    );
    assert!(matches!(
        build_bundle(&p, &[bad_receipt], &[bad_event]),
        Err(ProvenanceError::ReviewMismatch {
            kind: "receipt",
            ..
        })
    ));
}
