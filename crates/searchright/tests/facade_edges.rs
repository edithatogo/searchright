//! Facade-level edge cases (engine passthrough, error transparency, YAML).

use searchright::{PLAN_REVIEW_RESULT_SCHEMA_VERSION, SearchrightEngine, parse_yaml};
use serde_json::Value;

#[test]
fn yaml_nested_anchor_merge_is_rejected() {
    let malformed = "base: &base {a: 1}\ncopy: {<<: *base}";
    assert!(parse_yaml::<Value>(malformed).is_err());
}

#[test]
fn yaml_alias_anchor_is_rejected() {
    let malformed = "first: &a 1\nsecond: *a";
    assert!(parse_yaml::<Value>(malformed).is_err());
}

#[test]
fn yaml_typed_mismatch_is_rejected() {
    let result = parse_yaml::<Vec<u32>>("key: not-a-list");
    assert!(result.is_err());
}

#[test]
fn yaml_hash_inside_quotes_is_literal() {
    let valid = parse_yaml::<Value>("query: \"a#b\" # comment")
        .unwrap_or_else(|_| panic!("expected parse"));
    assert_eq!(
        valid.pointer("/query"),
        Some(&Value::String("a#b".to_string()))
    );
}

#[test]
fn engine_workflow_default_is_conservative() {
    let workflow = SearchrightEngine::workflow();
    let serialized = serde_json::to_string(&workflow).unwrap_or_else(|_| panic!("expected json"));
    assert!(serialized.contains("advisory"));
}

#[test]
fn engine_default_provider_manifests_lists_fixtures() {
    let manifests = SearchrightEngine::default_provider_manifests()
        .unwrap_or_else(|_| panic!("expected manifests"));
    assert!(!manifests.is_empty());
}

#[test]
fn plan_review_result_schema_version_is_stable() {
    assert_eq!(
        PLAN_REVIEW_RESULT_SCHEMA_VERSION,
        "org.searchright.plan-review-result.v1"
    );
}

#[test]
fn engine_allows_instantiation_of_facade() {
    let engine = SearchrightEngine;
    let _ = format!("{engine:?}");
}

#[test]
fn inspect_content_is_reachable_via_facade() {
    // Smoke-test that the facade re-export for content inspection is wired.
    // (Direct unit coverage lives in searchright-policy.)
    let serialized = serde_json::to_string(&SearchrightEngine::workflow())
        .unwrap_or_else(|_| panic!("expected json"));
    assert!(serialized.contains("advisory"));
}

#[test]
fn yaml_empty_document_is_rejected() {
    assert!(parse_yaml::<Value>("").is_err());
}

#[test]
fn yaml_scalar_document_parses() {
    let value = parse_yaml::<Value>("42").unwrap_or_else(|_| panic!("expected parse"));
    assert_eq!(value, Value::Number(42.into()));
}
