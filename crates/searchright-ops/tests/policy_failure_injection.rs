//! Failure-injection tests for operational policy gates.
//!
//! Each test drives one policy function with an invalid input and asserts
//! the expected `OpsError` variant, so a future policy relaxation cannot
//! silently pass these gates.

use searchright_contracts::{
    ACCESS_DECISION_SCHEMA_VERSION, AccessDecision, BACKUP_MANIFEST_SCHEMA_VERSION, BackupKind,
    BackupManifest, COMPONENT_HEALTH_SCHEMA_VERSION, ComponentHealth, HealthState,
    TELEMETRY_POLICY_SCHEMA_VERSION, TelemetryPolicy,
};
use searchright_ops::{OpsError, authorise_restore, authorise_telemetry_attribute, readiness};

fn health(component: &str, state: HealthState, ready: bool) -> ComponentHealth {
    ComponentHealth {
        schema_version: COMPONENT_HEALTH_SCHEMA_VERSION.to_string(),
        component: component.to_string(),
        state,
        diagnostics: vec![],
        ready,
        observed_at: "2026-08-06T00:00:00Z".to_string(),
    }
}

fn telemetry_policy(enabled: bool) -> TelemetryPolicy {
    TelemetryPolicy {
        schema_version: TELEMETRY_POLICY_SCHEMA_VERSION.to_string(),
        policy_id: "telemetry-policy-1".to_string(),
        enabled,
        endpoint: enabled.then_some("collector.example.test".to_string()),
        attribute_allowlist: vec!["review_id".to_string(), "duration_ms".to_string()],
        prohibited_attributes: vec!["free_text".to_string()],
        sampling_per_million: if enabled { 1000 } else { 0 },
        retention_days: 30,
        approved_by: enabled.then_some("privacy-reviewer".to_string()),
    }
}

fn backup(encrypted: bool, key_reference: bool) -> BackupManifest {
    BackupManifest {
        schema_version: BACKUP_MANIFEST_SCHEMA_VERSION.to_string(),
        backup_id: "backup-1".to_string(),
        scope_id: "scope-1".to_string(),
        kind: BackupKind::Full,
        parent_backup_id: None,
        digest_algorithm: "sha256".to_string(),
        digest: "a".repeat(64),
        encrypted,
        key_reference: key_reference.then_some("kms://key/1".to_string()),
        content_classes: vec!["review_state".to_string()],
        created_at: "2026-08-06T00:00:00Z".to_string(),
        retention_days: 365,
        restore_test_required: true,
    }
}

fn decision(permitted: bool) -> AccessDecision {
    AccessDecision {
        schema_version: ACCESS_DECISION_SCHEMA_VERSION.to_string(),
        request_id: "request-1".to_string(),
        tenant_id: "tenant-1".to_string(),
        permitted,
        blockers: if permitted {
            vec![]
        } else {
            vec!["restore.denied".to_string()]
        },
        human_approval_required: false,
    }
}

// --- readiness ---

#[test]
fn readiness_rejects_empty_components() {
    assert!(matches!(readiness(&[]), Err(OpsError::NoComponents)));
}

#[test]
fn readiness_rejects_invalid_component_contract() {
    let mut bad = health("search", HealthState::Healthy, true);
    bad.schema_version = "org.searchright.component-health.v99".to_string();
    assert!(matches!(readiness(&[bad]), Err(OpsError::Contract(_))));
}

#[test]
fn readiness_rejects_ready_unhealthy_component() {
    let bad = health("search", HealthState::Unhealthy, true);
    assert!(matches!(readiness(&[bad]), Err(OpsError::Contract(_))));
}

#[test]
fn readiness_all_healthy_is_ready() {
    let components = vec![
        health("search", HealthState::Healthy, true),
        health("store", HealthState::Healthy, true),
    ];
    let result = readiness(&components).unwrap_or_else(|_| panic!("expected readiness"));
    assert!(result.ready);
    assert!(result.unhealthy.is_empty());
    assert!(result.degraded.is_empty());
}

#[test]
fn readiness_degraded_ready_component_stays_ready_but_is_listed() {
    let components = vec![
        health("search", HealthState::Healthy, true),
        health("store", HealthState::Degraded, true),
    ];
    let result = readiness(&components).unwrap_or_else(|_| panic!("expected readiness"));
    assert!(result.ready);
    assert_eq!(result.degraded, vec!["store"]);
    assert!(result.unhealthy.is_empty());
}

#[test]
fn readiness_one_unhealthy_component_blocks_and_is_listed() {
    let components = vec![
        health("search", HealthState::Healthy, true),
        health("store", HealthState::Unhealthy, false),
    ];
    let result = readiness(&components).unwrap_or_else(|_| panic!("expected readiness"));
    assert!(!result.ready);
    assert_eq!(result.unhealthy, vec!["store"]);
}

#[test]
fn readiness_disabled_component_is_not_ready() {
    let components = vec![health("mcp", HealthState::Disabled, false)];
    let result = readiness(&components).unwrap_or_else(|_| panic!("expected readiness"));
    assert!(!result.ready);
    assert_eq!(result.unhealthy, vec!["mcp"]);
}

// --- telemetry ---

#[test]
fn telemetry_disabled_policy_is_rejected() {
    let policy = telemetry_policy(false);
    assert!(matches!(
        authorise_telemetry_attribute(&policy, "review_id"),
        Err(OpsError::TelemetryDisabled)
    ));
}

#[test]
fn telemetry_invalid_policy_propagates_contract_error() {
    let mut policy = telemetry_policy(true);
    policy.schema_version = "org.searchright.telemetry-policy.v99".to_string();
    assert!(matches!(
        authorise_telemetry_attribute(&policy, "review_id"),
        Err(OpsError::Contract(_))
    ));
}

#[test]
fn telemetry_prohibited_attribute_is_rejected() {
    let policy = telemetry_policy(true);
    assert!(matches!(
        authorise_telemetry_attribute(&policy, "free_text"),
        Err(OpsError::TelemetryAttributeProhibited(_))
    ));
}

#[test]
fn telemetry_unlisted_attribute_is_rejected() {
    let policy = telemetry_policy(true);
    assert!(matches!(
        authorise_telemetry_attribute(&policy, "query_text"),
        Err(OpsError::TelemetryAttributeNotAllowed(_))
    ));
}

#[test]
fn telemetry_allowlisted_attribute_is_authorised() {
    let policy = telemetry_policy(true);
    assert!(authorise_telemetry_attribute(&policy, "duration_ms").is_ok());
}

// --- restore ---

#[test]
fn restore_unencrypted_backup_is_rejected() {
    let manifest = backup(false, false);
    assert!(matches!(
        authorise_restore(&manifest, &decision(true)),
        Err(OpsError::UnencryptedRestore)
    ));
}

#[test]
fn restore_encryption_key_mismatch_propagates_contract_error() {
    let manifest = backup(true, false);
    assert!(matches!(
        authorise_restore(&manifest, &decision(true)),
        Err(OpsError::Contract(_))
    ));
}

#[test]
fn restore_unauthorised_operator_is_rejected() {
    let manifest = backup(true, true);
    assert!(matches!(
        authorise_restore(&manifest, &decision(false)),
        Err(OpsError::RestoreNotAuthorised)
    ));
}

#[test]
fn restore_denied_operator_with_no_blockers_propagates_contract_error() {
    let manifest = backup(true, true);
    let mut denied = decision(false);
    denied.blockers.clear();
    assert!(matches!(
        authorise_restore(&manifest, &denied),
        Err(OpsError::Contract(_))
    ));
}

#[test]
fn restore_encrypted_backup_with_authorised_operator_ok() {
    let manifest = backup(true, true);
    assert!(authorise_restore(&manifest, &decision(true)).is_ok());
}
