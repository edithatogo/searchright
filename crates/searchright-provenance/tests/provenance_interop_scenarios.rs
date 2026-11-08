//! Comprehensive provenance, RO-Crate 1.3, and W3C PROV interoperability integration scenarios.

#![forbid(unsafe_code)]

use searchright_contracts::{
    AUDIT_EVENT_SCHEMA_VERSION, Actor, AuditEvent, ExecutionPolicy, ReviewPlan,
    SOURCE_RECEIPT_SCHEMA_VERSION, SourceReceipt,
};
use searchright_provenance::{ProvenanceError, build_bundle, build_prov, build_ro_crate};

fn sample_plan(review_id: &str, title: &str) -> ReviewPlan {
    let mut plan: ReviewPlan =
        noyalib::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))
            .unwrap_or_else(|error| panic!("review-plan fixture must deserialize: {error}"));
    plan.review_id = review_id.to_string();
    plan.title = title.to_string();
    plan
}

const fn test_policy() -> ExecutionPolicy {
    ExecutionPolicy {
        live_enabled: false,
        max_records: 50,
        max_pages: 2,
        timeout_seconds: 15,
        total_timeout_seconds: Some(60),
        max_retries: 1,
        min_interval_ms: 100,
        retry_base_delay_ms: Some(100),
        retry_max_delay_ms: Some(500),
        max_response_bytes: Some(1_048_576),
        replay_enabled: true,
        cache_write_enabled: false,
    }
}

fn sample_receipt(
    id: &str,
    review_id: &str,
    provider: &str,
    records: u64,
    executed_at: &str,
) -> SourceReceipt {
    SourceReceipt {
        schema_version: SOURCE_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: id.to_string(),
        review_id: review_id.to_string(),
        run_id: format!("run-{id}"),
        provider_id: provider.to_string(),
        source_label: format!("Source {provider}"),
        strategy_id: "strategy-alpha".to_string(),
        query_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        executed_at: executed_at.to_string(),
        records_retrieved: records,
        pages_retrieved: 1,
        execution_mode: "fixture".to_string(),
        endpoint: Some(format!("https://api.{provider}.example.org/search")),
        policy: test_policy(),
        provider_version: "1.0.0".to_string(),
        compiler_version: "searchright-compiler/1.0.0".to_string(),
        result_digest: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
            .to_string(),
        cache_hits: 0,
        cache_writes: 0,
        warnings: vec![],
    }
}

fn sample_audit_event(
    id: &str,
    review_id: &str,
    event_type: &str,
    occurred_at: &str,
    actor_id: &str,
    actor_type: &str,
    previous_hash: &str,
    event_hash: &str,
) -> AuditEvent {
    AuditEvent {
        schema_version: AUDIT_EVENT_SCHEMA_VERSION.to_string(),
        event_id: id.to_string(),
        review_id: review_id.to_string(),
        event_type: event_type.to_string(),
        occurred_at: occurred_at.to_string(),
        actor: Actor {
            actor_id: actor_id.to_string(),
            actor_type: actor_type.to_string(),
            provenance: Some(format!("{actor_type} execution context")),
        },
        payload: serde_json::json!({
            "step": event_type,
            "status": "completed"
        }),
        previous_hash: previous_hash.to_string(),
        event_hash: event_hash.to_string(),
    }
}

#[test]
fn test_ro_crate_1_3_conformance_and_graph_topology() {
    let plan = sample_plan("rev-top-001", "Topology and Conformance Study");
    let receipts = vec![
        sample_receipt(
            "rcpt-01",
            "rev-top-001",
            "pubmed",
            25,
            "2026-11-01T10:00:00Z",
        ),
        sample_receipt(
            "rcpt-02",
            "rev-top-001",
            "cochrane",
            14,
            "2026-11-01T10:05:00Z",
        ),
        sample_receipt(
            "rcpt-03",
            "rev-top-001",
            "pubmed",
            8,
            "2026-11-01T10:10:00Z",
        ),
    ];
    let events = vec![
        sample_audit_event(
            "ev-01",
            "rev-top-001",
            "review.initialized",
            "2026-11-01T09:50:00Z",
            "investigator-alice",
            "human",
            "GENESIS",
            "hash-genesis-01",
        ),
        sample_audit_event(
            "ev-02",
            "rev-top-001",
            "search.executed",
            "2026-11-01T10:12:00Z",
            "searchright-agent-runner",
            "agent",
            "hash-genesis-01",
            "hash-step-02",
        ),
    ];

    let ro_crate = build_ro_crate(&plan, &receipts, &events)
        .unwrap_or_else(|err| panic!("build_ro_crate failed: {err}"));

    // Verify RO-Crate Context
    assert_eq!(
        ro_crate.get("@context").and_then(|v| v.as_str()),
        Some("https://w3id.org/ro/crate/1.3/context"),
        "RO-Crate context must be 1.3"
    );

    let graph = ro_crate
        .get("@graph")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("@graph must be a JSON array"));

    // Verify metadata descriptor exists
    let meta_desc = graph
        .iter()
        .find(|e| e.get("@id").and_then(|v| v.as_str()) == Some("ro-crate-metadata.json"))
        .unwrap_or_else(|| panic!("ro-crate-metadata.json entity missing"));
    assert_eq!(
        meta_desc.get("@type").and_then(|v| v.as_str()),
        Some("CreativeWork")
    );
    assert_eq!(
        meta_desc
            .get("conformsTo")
            .and_then(|v| v.get("@id"))
            .and_then(|v| v.as_str()),
        Some("https://w3id.org/ro/crate/1.3")
    );

    // Verify Root Dataset exists
    let root_ds = graph
        .iter()
        .find(|e| e.get("@id").and_then(|v| v.as_str()) == Some("./"))
        .unwrap_or_else(|| panic!("Root dataset ./ missing"));
    assert_eq!(
        root_ds.get("@type").and_then(|v| v.as_str()),
        Some("Dataset")
    );
    assert_eq!(
        root_ds.get("identifier").and_then(|v| v.as_str()),
        Some("rev-top-001")
    );
    let has_parts = root_ds
        .get("hasPart")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("hasPart must be array"));
    assert_eq!(
        has_parts.len(),
        3,
        "All 3 receipts must be listed in hasPart"
    );

    // Verify Provider deduplication: 2 distinct providers (pubmed, cochrane) despite 3 receipts
    let provider_count = graph
        .iter()
        .filter(|e| {
            e.get("@id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id.starts_with("urn:searchright:provider:"))
        })
        .count();
    assert_eq!(
        provider_count, 2,
        "Duplicate provider entries must be deduplicated in @graph"
    );

    // Verify Actor differentiation: Alice (Person) vs Runner (SoftwareApplication)
    let alice = graph
        .iter()
        .find(|e| {
            e.get("@id").and_then(|v| v.as_str())
                == Some("urn:searchright:actor:investigator-alice")
        })
        .unwrap_or_else(|| panic!("Alice entity missing"));
    assert_eq!(alice.get("@type").and_then(|v| v.as_str()), Some("Person"));

    let agent = graph
        .iter()
        .find(|e| {
            e.get("@id").and_then(|v| v.as_str())
                == Some("urn:searchright:actor:searchright-agent-runner")
        })
        .unwrap_or_else(|| panic!("Agent entity missing"));
    assert_eq!(
        agent.get("@type").and_then(|v| v.as_str()),
        Some("SoftwareApplication")
    );
}

#[test]
fn test_w3c_prov_derivation_chain_and_entities() {
    let plan = sample_plan("rev-prov-002", "PROV Derivation Chain Study");
    let receipts = vec![sample_receipt(
        "rcpt-prov-01",
        "rev-prov-002",
        "openalex",
        120,
        "2026-11-02T14:00:00Z",
    )];
    let events = vec![
        sample_audit_event(
            "ev-p1",
            "rev-prov-002",
            "protocol.drafted",
            "2026-11-02T13:00:00Z",
            "lead-investigator",
            "human",
            "GENESIS",
            "hash-genesis-p1",
        ),
        sample_audit_event(
            "ev-p2",
            "rev-prov-002",
            "strategy.compiled",
            "2026-11-02T13:30:00Z",
            "compiler-service",
            "agent",
            "hash-genesis-p1",
            "hash-p2",
        ),
        sample_audit_event(
            "ev-p3",
            "rev-prov-002",
            "search.dispatched",
            "2026-11-02T14:00:00Z",
            "connector-runtime",
            "agent",
            "hash-p2",
            "hash-p3",
        ),
    ];

    let prov = build_prov(&plan, &receipts, &events)
        .unwrap_or_else(|err| panic!("build_prov failed: {err}"));

    // Verify Prefixes
    let prefixes = prov
        .get("prefix")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("prefix must be object"));
    assert_eq!(
        prefixes.get("prov").and_then(|v| v.as_str()),
        Some("http://www.w3.org/ns/prov#")
    );
    assert_eq!(
        prefixes.get("sr").and_then(|v| v.as_str()),
        Some("https://schemas.searchright.dev/prov/")
    );

    // Verify Entities
    let entities = prov
        .get("entity")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("entity must be object"));
    assert!(entities.contains_key("sr:receipt-rcpt-prov-01"));
    assert!(entities.contains_key("sr:review-rev-prov-002"));

    // Verify Derivations (wasDerivedFrom): ev-p2 derives from ev-p1, ev-p3 derives from ev-p2
    let derivations = prov
        .get("wasDerivedFrom")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("wasDerivedFrom must be object"));
    assert_eq!(
        derivations.len(),
        2,
        "There must be exactly 2 derivations in a 3-event chain starting with GENESIS"
    );

    let deriv_p2 = derivations
        .get("sr:derivation-ev-p2")
        .unwrap_or_else(|| panic!("derivation for ev-p2 missing"));
    assert_eq!(
        deriv_p2
            .get("prov:generatedEntity")
            .and_then(|v| v.as_str()),
        Some("sr:event-ev-p2")
    );
    assert_eq!(
        deriv_p2.get("prov:usedEntity").and_then(|v| v.as_str()),
        Some("sr:event-ev-p1")
    );

    let deriv_p3 = derivations
        .get("sr:derivation-ev-p3")
        .unwrap_or_else(|| panic!("derivation for ev-p3 missing"));
    assert_eq!(
        deriv_p3
            .get("prov:generatedEntity")
            .and_then(|v| v.as_str()),
        Some("sr:event-ev-p3")
    );
    assert_eq!(
        deriv_p3.get("prov:usedEntity").and_then(|v| v.as_str()),
        Some("sr:event-ev-p2")
    );

    // Verify Associations (wasAssociatedWith)
    let associations = prov
        .get("wasAssociatedWith")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("wasAssociatedWith must be object"));
    assert_eq!(associations.len(), 3, "Each event must have an association");
}

#[test]
fn test_unresolved_predecessor_fails_closed() {
    let plan = sample_plan("rev-tamper-003", "Tamper & Unresolved Predecessor");
    let receipts = vec![sample_receipt(
        "rcpt-t01",
        "rev-tamper-003",
        "pubmed",
        5,
        "2026-11-03T10:00:00Z",
    )];
    // Event references a non-existent previous hash not equal to GENESIS
    let broken_events = vec![sample_audit_event(
        "ev-orphan",
        "rev-tamper-003",
        "audit.tampered",
        "2026-11-03T10:05:00Z",
        "malicious-actor",
        "agent",
        "non-existent-prior-hash-deadbeef",
        "hash-orphan",
    )];

    let result = build_bundle(&plan, &receipts, &broken_events);
    match result {
        Err(ProvenanceError::UnresolvedPredecessor { event_id }) => {
            assert_eq!(event_id, "ev-orphan");
        }
        other => panic!("Expected UnresolvedPredecessor, got {other:?}"),
    }
}

#[test]
fn test_cross_review_contamination_rejected() {
    let plan = sample_plan("rev-owner-004", "Legitimate Review");
    let foreign_receipt = sample_receipt(
        "rcpt-foreign",
        "rev-other-999", // Different review!
        "pubmed",
        10,
        "2026-11-04T12:00:00Z",
    );

    let result = build_bundle(&plan, &[foreign_receipt], &[]);
    match result {
        Err(ProvenanceError::ReviewMismatch { kind, identifier }) => {
            assert_eq!(kind, "receipt");
            assert_eq!(identifier, "rcpt-foreign");
        }
        other => panic!("Expected ReviewMismatch, got {other:?}"),
    }
}

#[test]
fn test_duplicate_event_hash_rejected() {
    let plan = sample_plan("rev-dup-005", "Duplicate Hash Invariant");
    let events = vec![
        sample_audit_event(
            "ev-a",
            "rev-dup-005",
            "step.one",
            "2026-11-05T08:00:00Z",
            "user",
            "human",
            "GENESIS",
            "colliding-hash-1234",
        ),
        sample_audit_event(
            "ev-b",
            "rev-dup-005",
            "step.two",
            "2026-11-05T08:01:00Z",
            "user",
            "human",
            "colliding-hash-1234",
            "colliding-hash-1234", // Duplicate hash!
        ),
    ];

    let result = build_bundle(&plan, &[], &events);
    match result {
        Err(ProvenanceError::DuplicateIdentifier { kind, identifier }) => {
            assert_eq!(kind, "audit event hash");
            assert_eq!(identifier, "colliding-hash-1234");
        }
        other => panic!("Expected DuplicateIdentifier, got {other:?}"),
    }
}

#[test]
fn test_deterministic_reproducibility_across_permutations() {
    let plan = sample_plan("rev-det-006", "Determinism Invariant Test");
    let r1 = sample_receipt(
        "r-01",
        "rev-det-006",
        "provider-a",
        10,
        "2026-11-06T00:00:00Z",
    );
    let r2 = sample_receipt(
        "r-02",
        "rev-det-006",
        "provider-b",
        20,
        "2026-11-06T00:01:00Z",
    );
    let r3 = sample_receipt(
        "r-03",
        "rev-det-006",
        "provider-c",
        30,
        "2026-11-06T00:02:00Z",
    );

    let e1 = sample_audit_event(
        "e-01",
        "rev-det-006",
        "init",
        "2026-11-06T00:00:00Z",
        "u1",
        "human",
        "GENESIS",
        "h1",
    );
    let e2 = sample_audit_event(
        "e-02",
        "rev-det-006",
        "run",
        "2026-11-06T00:01:00Z",
        "a1",
        "agent",
        "h1",
        "h2",
    );

    let bundle_asc = build_bundle(
        &plan,
        &[r1.clone(), r2.clone(), r3.clone()],
        &[e1.clone(), e2.clone()],
    )
    .unwrap_or_else(|err| panic!("bundle_asc failed: {err}"));

    // Permute input ordering
    let bundle_permuted = build_bundle(&plan, &[r3, r1, r2], &[e2, e1])
        .unwrap_or_else(|err| panic!("bundle_permuted failed: {err}"));

    let json_asc =
        serde_json::to_string(&bundle_asc).unwrap_or_else(|_| panic!("to_string failed"));
    let json_perm =
        serde_json::to_string(&bundle_permuted).unwrap_or_else(|_| panic!("to_string failed"));

    assert_eq!(
        json_asc, json_perm,
        "ProvenanceBundle output must be bitwise identical regardless of input collection order"
    );
}
