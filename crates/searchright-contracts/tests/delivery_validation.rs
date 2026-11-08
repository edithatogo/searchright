//! Track 16 regression coverage for existing delivery safety boundaries.
//! Validation is local only: none of these tests authorises remote effects.

use searchright_contracts::{
    GitHubRepositorySettings, IntegrationReleaseTrain, ReleaseRehearsal, Validate,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

const SETTINGS: &str = include_str!("../../../conductor/github/repository-settings.json");
const TRAIN: &str = include_str!("../../../integration/release-train.json");
const REHEARSAL: &str = include_str!("../../../release/rehearsal.json");

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rejects<T: DeserializeOwned + Validate>(
    fixture: &str,
    pointer: &str,
    replacement: Value,
    message: &str,
) -> TestResult {
    let mut value: Value = serde_json::from_str(fixture)?;
    let slot = value.pointer_mut(pointer).ok_or("test pointer missing")?;
    *slot = replacement;
    let contract: T = serde_json::from_value(value)?;
    let error = contract
        .validate()
        .err()
        .ok_or("unsafe contract accepted")?;
    assert!(error.to_string().contains(message), "{pointer}: {error}");
    Ok(())
}

fn roundtrip<T: DeserializeOwned + Serialize + Validate>(fixture: &str) -> TestResult {
    let contract: T = serde_json::from_str(fixture)?;
    contract.validate()?;
    let encoded = serde_json::to_value(&contract)?;
    let restored: T = serde_json::from_value(encoded.clone())?;
    restored.validate()?;
    assert_eq!(encoded, serde_json::to_value(restored)?);
    Ok(())
}

#[test]
fn canonical_delivery_manifests_validate_and_roundtrip() -> TestResult {
    roundtrip::<GitHubRepositorySettings>(SETTINGS)?;
    roundtrip::<IntegrationReleaseTrain>(TRAIN)?;
    roundtrip::<ReleaseRehearsal>(REHEARSAL)
}

#[test]
fn repository_settings_reject_unsafe_controls_independently() -> TestResult {
    for (pointer, value, message) in [
        (
            "/apply_permitted",
            json!(true),
            "cannot authorise remote writes",
        ),
        (
            "/features/issues",
            json!(false),
            "requires Issues and Projects",
        ),
        (
            "/features/projects",
            json!(false),
            "requires Issues and Projects",
        ),
        ("/features/wiki", json!(true), "Wiki disabled"),
        (
            "/merge_policy/merge_commit",
            json!(true),
            "disables merge commits",
        ),
        (
            "/merge_policy/squash",
            json!(false),
            "requires squash merging",
        ),
        ("/topics", json!([]), "topics_or_environments"),
        ("/environments", json!([]), "topics_or_environments"),
        ("/ruleset/target", json!("tag"), "must target branches"),
        (
            "/ruleset/deletion",
            json!(true),
            "deny deletion/non-fast-forward",
        ),
        (
            "/ruleset/non_fast_forward",
            json!(true),
            "deny deletion/non-fast-forward",
        ),
        (
            "/ruleset/required_status_checks",
            json!([]),
            "require checks",
        ),
    ] {
        rejects::<GitHubRepositorySettings>(SETTINGS, pointer, value, message)?;
    }
    for repository in ["owner", "/name", "owner/", "owner/name/extra"] {
        rejects::<GitHubRepositorySettings>(
            SETTINGS,
            "/repository",
            json!(repository),
            "owner/name",
        )?;
    }
    Ok(())
}

#[test]
fn release_train_rejects_automatic_incomplete_or_ambiguous_promotion() -> TestResult {
    for (pointer, value, message) in [
        ("/automatic_promotion", json!(true), "must remain explicit"),
        ("/components", json!([]), "requires multiple components"),
        ("/stages", json!([]), "requires multiple components"),
        ("/rollback", json!([]), "requires multiple components"),
        (
            "/components/0/promotion_order",
            json!(0),
            "positive unique order",
        ),
        (
            "/components/1/promotion_order",
            json!(1),
            "positive unique order",
        ),
        (
            "/components/1/repository",
            json!("edithatogo/citeweft"),
            "unique repositories",
        ),
        (
            "/components/0/repository",
            json!("invalid"),
            "unique repositories",
        ),
        ("/stages/0/automatic", json!(true), "explicitly promoted"),
        ("/stages/1/id", json!("contract"), "must be unique"),
    ] {
        rejects::<IntegrationReleaseTrain>(TRAIN, pointer, value, message)?;
    }
    let mut value: Value = serde_json::from_str(TRAIN)?;
    let components = value
        .get_mut("components")
        .and_then(Value::as_array_mut)
        .ok_or("components missing")?;
    components.truncate(1);
    let train: IntegrationReleaseTrain = serde_json::from_value(value)?;
    assert!(train.validate().is_err());
    Ok(())
}

#[test]
fn rehearsal_requires_explicit_promotion_gates_pilots_and_rollback() -> TestResult {
    for (pointer, value, message) in [
        (
            "/required_gates",
            json!([]),
            "required_gates_or_pilot_profiles",
        ),
        (
            "/pilot_profiles",
            json!([]),
            "required_gates_or_pilot_profiles",
        ),
        (
            "/automatic_release",
            json!(true),
            "promotion must be explicit",
        ),
        (
            "/automatic_registry_submission",
            json!(true),
            "promotion must be explicit",
        ),
        (
            "/rollback_required",
            json!(false),
            "rollback must be required",
        ),
    ] {
        rejects::<ReleaseRehearsal>(REHEARSAL, pointer, value, message)?;
    }
    Ok(())
}

#[test]
fn delivery_contracts_reject_wrong_versions_and_blank_required_text() -> TestResult {
    rejects::<GitHubRepositorySettings>(
        SETTINGS,
        "/schema_version",
        json!("unknown"),
        "schema_version",
    )?;
    rejects::<IntegrationReleaseTrain>(
        TRAIN,
        "/schema_version",
        json!("unknown"),
        "schema_version",
    )?;
    rejects::<ReleaseRehearsal>(
        REHEARSAL,
        "/schema_version",
        json!("unknown"),
        "schema_version",
    )?;
    for field in ["repository", "description", "homepage", "claim_boundary"] {
        rejects::<GitHubRepositorySettings>(SETTINGS, &format!("/{field}"), json!("  "), field)?;
    }
    for pointer in [
        "/release_train_id",
        "/generated_at",
        "/claim_boundary",
        "/components/0/repository",
        "/components/0/role",
        "/stages/0/id",
        "/stages/0/required_evidence",
    ] {
        let field = pointer.rsplit('/').next().ok_or("field missing")?;
        rejects::<IntegrationReleaseTrain>(TRAIN, pointer, json!("  "), field)?;
    }
    for field in ["target", "source_epoch", "claim_boundary"] {
        rejects::<ReleaseRehearsal>(REHEARSAL, &format!("/{field}"), json!("  "), field)?;
    }
    Ok(())
}
