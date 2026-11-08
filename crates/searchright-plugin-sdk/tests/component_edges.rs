//! Component verification edge cases.

use searchright_contracts::{
    ComponentCapability, ComponentSignatureAlgorithm, ProviderComponentManifest,
    ProviderComponentReleaseSignature, ProviderComponentRevocation, ProviderComponentTrustKey,
    ProviderComponentTrustPolicy,
};
use searchright_plugin_sdk::{
    PROVIDER_ABI_VERSION, PluginError, authorise_capability, manifest_digest, release_message,
    verify_component, verify_signed_component_release,
};

const COMPONENT: &[u8] = b"component-bytes";

fn manifest(bytes: &[u8]) -> ProviderComponentManifest {
    ProviderComponentManifest {
        schema_version: "org.searchright.provider-component.v1".to_string(),
        component_id: "io.github.edithatogo.searchright.fixture".to_string(),
        component_version: "0.1.0".to_string(),
        abi_version: PROVIDER_ABI_VERSION.to_string(),
        capabilities: vec![ComponentCapability::Search],
        allowed_hosts: vec![],
        max_memory_mib: 64,
        max_fuel: 1_000_000,
        component_digest: blake3::hash(bytes).to_hex().to_string(),
        fixture_mode: true,
        authority_rationale: vec!["fixture searches".to_string()],
    }
}

fn signed_release(
    component: &[u8],
) -> (
    ProviderComponentManifest,
    ProviderComponentReleaseSignature,
    ProviderComponentTrustPolicy,
) {
    use aws_lc_rs::signature::{Ed25519KeyPair, KeyPair};
    use base64::Engine as _;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;

    let m = manifest(component);
    let key_pair = Ed25519KeyPair::from_seed_unchecked(&[7_u8; 32])
        .unwrap_or_else(|_| panic!("test key must be valid"));
    let mut release = ProviderComponentReleaseSignature {
        schema_version: "org.searchright.provider-component-release-signature.v1".to_string(),
        component_id: m.component_id.clone(),
        component_version: m.component_version.clone(),
        manifest_digest: manifest_digest(&m).unwrap_or_else(|_| panic!("manifest must serialise")),
        component_digest: m.component_digest.clone(),
        key_id: "fixture-signing-key-1".to_string(),
        trust_policy_id: "fixture-component-trust-policy".to_string(),
        algorithm: ComponentSignatureAlgorithm::Ed25519,
        signed_at: "2026-08-14T00:00:00Z".to_string(),
        expires_at: "2026-09-14T00:00:00Z".to_string(),
        signature: String::new(),
    };
    release.signature =
        BASE64_URL_SAFE_NO_PAD.encode(key_pair.sign(&release_message(&m, &release)));
    let policy = ProviderComponentTrustPolicy {
        schema_version: "org.searchright.provider-component-trust-policy.v1".to_string(),
        policy_id: release.trust_policy_id.clone(),
        trusted_keys: vec![ProviderComponentTrustKey {
            key_id: release.key_id.clone(),
            algorithm: ComponentSignatureAlgorithm::Ed25519,
            public_key: BASE64_URL_SAFE_NO_PAD.encode(key_pair.public_key().as_ref()),
            component_ids: vec![m.component_id.clone()],
            valid_from: "2026-08-01T00:00:00Z".to_string(),
            valid_until: "2027-08-01T00:00:00Z".to_string(),
        }],
        revocations: vec![],
    };
    (m, release, policy)
}

#[test]
fn abi_mismatch_is_rejected_with_expected_and_actual() {
    let mut m = manifest(COMPONENT);
    m.abi_version = "wrong-abi".to_string();
    assert!(matches!(
        verify_component(&m, COMPONENT),
        Err(PluginError::AbiMismatch { expected, actual }) if expected == PROVIDER_ABI_VERSION && actual == "wrong-abi"
    ));
}

#[test]
fn invalid_manifest_schema_propagates_contract_error() {
    let mut m = manifest(COMPONENT);
    m.schema_version = "org.searchright.provider-component.v99".to_string();
    assert!(matches!(
        verify_component(&m, COMPONENT),
        Err(PluginError::Contract(_))
    ));
    assert!(matches!(
        authorise_capability(&m, ComponentCapability::Search),
        Err(PluginError::Contract(_))
    ));
}

#[test]
fn empty_component_bytes_digest_is_still_checked() {
    let m = manifest(COMPONENT);
    assert!(matches!(
        verify_component(&m, b""),
        Err(PluginError::DigestMismatch { .. })
    ));
}

#[test]
fn manifest_digest_is_deterministic_and_hex() {
    let m = manifest(COMPONENT);
    let first = manifest_digest(&m).unwrap_or_else(|_| panic!("expected digest"));
    let second = manifest_digest(&m).unwrap_or_else(|_| panic!("expected digest"));
    assert_eq!(first, second);
    assert_eq!(first.len(), 64);
    assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn capability_granted_search_is_authorised() {
    let m = manifest(COMPONENT);
    assert!(authorise_capability(&m, ComponentCapability::Search).is_ok());
}

#[test]
fn each_undeclared_capability_is_denied() {
    let m = manifest(COMPONENT);
    assert!(matches!(
        authorise_capability(&m, ComponentCapability::NetworkRead),
        Err(PluginError::CapabilityDenied(_))
    ));
}

#[test]
fn release_message_is_deterministic_and_domain_separated() {
    let (m, r, _) = signed_release(COMPONENT);
    let first = release_message(&m, &r);
    let second = release_message(&m, &r);
    assert_eq!(first, second);
    assert!(!first.is_empty());
}

#[test]
fn release_before_signed_at_is_outside_window() {
    let (m, r, p) = signed_release(COMPONENT);
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-13T00:00:00Z"),
        Err(PluginError::ReleaseOutsideValidityWindow)
    ));
}

#[test]
fn release_at_expiry_is_outside_window() {
    let (m, r, p) = signed_release(COMPONENT);
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-09-14T00:00:00Z"),
        Err(PluginError::ReleaseOutsideValidityWindow)
    ));
}

#[test]
fn untrusted_key_id_is_rejected() {
    let (m, mut r, p) = signed_release(COMPONENT);
    r.key_id = "unknown-key".to_string();
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-15T00:00:00Z"),
        Err(PluginError::UntrustedKey(_))
    ));
}

#[test]
fn key_not_authorised_for_component_is_denied() {
    let (m, r, mut p) = signed_release(COMPONENT);
    p.trusted_keys
        .first_mut()
        .unwrap_or_else(|| panic!("expected key"))
        .component_ids = vec!["other-component".to_string()];
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-15T00:00:00Z"),
        Err(PluginError::KeyAuthorityDenied)
    ));
}

#[test]
fn future_revocation_does_not_block_release() {
    let (m, r, mut p) = signed_release(COMPONENT);
    p.revocations = vec![ProviderComponentRevocation {
        key_id: r.key_id.clone(),
        revoked_at: "2027-01-01T00:00:00Z".to_string(),
        evidence_reference: "receipt:future".to_string(),
    }];
    assert!(verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-15T00:00:00Z").is_ok());
}

#[test]
fn invalid_signature_contract_is_rejected() {
    let (m, mut r, p) = signed_release(COMPONENT);
    r.signature = "!!not-base64!!".to_string();
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-15T00:00:00Z"),
        Err(PluginError::Contract(_))
    ));
}

#[test]
fn invalid_public_key_contract_is_rejected() {
    let (m, r, mut p) = signed_release(COMPONENT);
    p.trusted_keys
        .first_mut()
        .unwrap_or_else(|| panic!("expected key"))
        .public_key = "!!not-base64!!".to_string();
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "2026-08-15T00:00:00Z"),
        Err(PluginError::Contract(_))
    ));
}

#[test]
fn invalid_now_timestamp_is_rejected() {
    let (m, r, p) = signed_release(COMPONENT);
    assert!(matches!(
        verify_signed_component_release(&m, COMPONENT, &r, &p, "not-a-timestamp"),
        Err(PluginError::InvalidTimestamp(_))
    ));
}
