//! Access decision edge cases.

use searchright_access::{
    ReplayLedger, authenticated_read_request, authorise, authorise_with_replay,
};
use searchright_contracts::{
    ACCESS_REQUEST_SCHEMA_VERSION, AccessScope, PrincipalKind, TENANT_POLICY_SCHEMA_VERSION,
    TenantPolicy,
};

fn policy() -> TenantPolicy {
    TenantPolicy {
        schema_version: TENANT_POLICY_SCHEMA_VERSION.to_string(),
        tenant_id: "tenant-demo".to_string(),
        allowed_regions: vec!["AU".to_string()],
        allowed_scopes: vec![AccessScope::ReviewRead, AccessScope::SearchExecute],
        maximum_concurrent_tasks: 4,
        external_model_processing_allowed: false,
        restricted_full_text_persistence_allowed: false,
        cross_tenant_aggregation_allowed: false,
        approved_by: "Governance officer".to_string(),
        policy_version: "1".to_string(),
    }
}

fn request() -> searchright_contracts::AccessRequest {
    authenticated_read_request(
        "req-1".to_string(),
        "principal-1".to_string(),
        PrincipalKind::Human,
        "tenant-demo".to_string(),
        vec![AccessScope::ReviewRead],
        "AU".to_string(),
    )
}

#[test]
fn authenticated_read_request_has_read_only_flags() {
    let req = request();
    assert!(req.authenticated);
    assert!(!req.external_write);
    assert!(!req.final_eligibility_decision);
    assert!(!req.human_approval);
    assert_eq!(req.schema_version, ACCESS_REQUEST_SCHEMA_VERSION);
}

#[test]
fn ledger_capacity_zero_clamps_to_one() {
    let mut ledger = ReplayLedger::with_capacity(0);
    ledger.record_accepted("a");
    assert!(ledger.has_seen("a"));
    ledger.record_accepted("b");
    // Only one entry retained.
    assert!(!ledger.has_seen("a"));
    assert!(ledger.has_seen("b"));
}

#[test]
fn ledger_duplicate_record_is_idempotent() {
    let mut ledger = ReplayLedger::with_capacity(2);
    ledger.record_accepted("a");
    ledger.record_accepted("a");
    ledger.record_accepted("b");
    assert!(ledger.has_seen("a"));
    assert!(ledger.has_seen("b"));
}

#[test]
fn ledger_evicts_oldest_when_full() {
    let mut ledger = ReplayLedger::with_capacity(2);
    ledger.record_accepted("a");
    ledger.record_accepted("b");
    ledger.record_accepted("c");
    assert!(!ledger.has_seen("a"));
    assert!(ledger.has_seen("b"));
    assert!(ledger.has_seen("c"));
}

#[test]
fn ledger_has_seen_unknown_is_false() {
    let ledger = ReplayLedger::with_capacity(4);
    assert!(!ledger.has_seen("never"));
}

#[test]
fn denied_request_is_not_recorded_in_replay_ledger() {
    let pol = policy();
    let mut denied = request();
    denied.tenant_id = "tenant-other".to_string();
    let mut ledger = ReplayLedger::with_capacity(4);
    let decision = authorise_with_replay(&pol, &denied, &mut ledger)
        .unwrap_or_else(|_| panic!("expected decision"));
    assert!(!decision.permitted);
    assert!(!ledger.has_seen(&denied.request_id));
}

#[test]
fn invalid_policy_schema_propagates_contract_error() {
    let mut pol = policy();
    pol.schema_version = "org.searchright.tenant-policy.v99".to_string();
    assert!(authorise(&pol, &request()).is_err());
}

#[test]
fn invalid_request_schema_propagates_contract_error() {
    let mut req = request();
    req.schema_version = "org.searchright.access-request.v99".to_string();
    assert!(authorise(&policy(), &req).is_err());
}

#[test]
fn search_execute_scope_is_permitted_when_allowlisted() {
    let mut req = request();
    req.scopes = vec![AccessScope::SearchExecute];
    let decision = authorise(&policy(), &req).unwrap_or_else(|_| panic!("expected decision"));
    assert!(decision.permitted);
}

#[test]
fn human_approval_required_only_for_write_or_final_decision() {
    // Read-only request: no human approval required.
    let decision = authorise(&policy(), &request()).unwrap_or_else(|_| panic!("expected decision"));
    assert!(!decision.human_approval_required);
    // External-write blocker sets human_approval_required.
    let mut req = request();
    req.external_write = true;
    let decision = authorise(&policy(), &req).unwrap_or_else(|_| panic!("expected decision"));
    assert!(decision.human_approval_required);
}

#[test]
fn denied_decision_retains_request_and_policy_tenant() {
    let mut req = request();
    req.tenant_id = "tenant-other".to_string();
    let decision = authorise(&policy(), &req).unwrap_or_else(|_| panic!("expected decision"));
    assert_eq!(decision.request_id, req.request_id);
    assert_eq!(decision.tenant_id, "tenant-demo");
}

#[test]
fn empty_scopes_propagates_contract_error() {
    let mut req = request();
    req.scopes.clear();
    assert!(authorise(&policy(), &req).is_err());
}
