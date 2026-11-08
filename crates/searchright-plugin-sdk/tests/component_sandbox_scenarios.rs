//! Integration scenarios for Track 24: WASI component trust, signed Ed25519 manifests,
//! capability controls, and strict sandbox enforcement.

#![forbid(unsafe_code)]

use aws_lc_rs::signature::{Ed25519KeyPair, KeyPair};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use searchright_contracts::{
    ComponentCapability, ComponentSignatureAlgorithm, ProviderComponentManifest,
    ProviderComponentReleaseSignature, ProviderComponentRevocation, ProviderComponentTrustKey,
    ProviderComponentTrustPolicy,
};
use searchright_plugin_sdk::{
    PROVIDER_ABI_VERSION, PluginError, authorise_capability, manifest_digest, release_message,
    verify_component, verify_signed_component_release,
};

const TEST_COMPONENT_BYTES: &[u8] = b"wasi-provider-component-wasm-bytecode-fixture-v1";

fn create_test_manifest(
    bytes: &[u8],
    capabilities: Vec<ComponentCapability>,
) -> ProviderComponentManifest {
    let allowed_hosts = if capabilities.contains(&ComponentCapability::NetworkRead) {
        vec!["eutils.ncbi.nlm.nih.gov".to_string()]
    } else {
        vec![]
    };
    ProviderComponentManifest {
        schema_version: "org.searchright.provider-component.v1".to_string(),
        component_id: "org.searchright.provider.pubmed-wasi".to_string(),
        component_version: "1.0.0".to_string(),
        abi_version: PROVIDER_ABI_VERSION.to_string(),
        capabilities,
        allowed_hosts,
        max_memory_mib: 128,
        max_fuel: 5_000_000,
        component_digest: blake3::hash(bytes).to_hex().to_string(),
        fixture_mode: true,
        authority_rationale: vec!["PubMed open eUtils API discovery in sandboxed WASI".to_string()],
    }
}

fn create_signed_bundle(
    component_bytes: &[u8],
    capabilities: Vec<ComponentCapability>,
) -> (
    ProviderComponentManifest,
    ProviderComponentReleaseSignature,
    ProviderComponentTrustPolicy,
) {
    let manifest = create_test_manifest(component_bytes, capabilities);
    let Ok(manifest_hash) = manifest_digest(&manifest) else {
        panic!("manifest must serialize");
    };

    let Ok(key_pair) = Ed25519KeyPair::from_seed_unchecked(&[42_u8; 32]) else {
        panic!("key pair generation must succeed");
    };

    let mut release = ProviderComponentReleaseSignature {
        schema_version: "org.searchright.provider-component-release-signature.v1".to_string(),
        component_id: manifest.component_id.clone(),
        component_version: manifest.component_version.clone(),
        manifest_digest: manifest_hash,
        component_digest: manifest.component_digest.clone(),
        key_id: "publisher-primary-key-2026".to_string(),
        trust_policy_id: "searchright-production-trust-policy-v1".to_string(),
        algorithm: ComponentSignatureAlgorithm::Ed25519,
        signed_at: "2026-08-01T00:00:00Z".to_string(),
        expires_at: "2026-12-01T00:00:00Z".to_string(),
        signature: String::new(),
    };

    let message = release_message(&manifest, &release);
    release.signature = BASE64_URL_SAFE_NO_PAD.encode(key_pair.sign(&message));

    let policy = ProviderComponentTrustPolicy {
        schema_version: "org.searchright.provider-component-trust-policy.v1".to_string(),
        policy_id: "searchright-production-trust-policy-v1".to_string(),
        trusted_keys: vec![ProviderComponentTrustKey {
            key_id: "publisher-primary-key-2026".to_string(),
            algorithm: ComponentSignatureAlgorithm::Ed25519,
            public_key: BASE64_URL_SAFE_NO_PAD.encode(key_pair.public_key().as_ref()),
            component_ids: vec![manifest.component_id.clone()],
            valid_from: "2026-01-01T00:00:00Z".to_string(),
            valid_until: "2027-01-01T00:00:00Z".to_string(),
        }],
        revocations: vec![],
    };

    (manifest, release, policy)
}

/// Scenario 1: Complete Ed25519 signed manifest release verification (T24-A001).
#[test]
fn test_component_signed_release_verification_success() {
    let (manifest, release, policy) =
        create_signed_bundle(TEST_COMPONENT_BYTES, vec![ComponentCapability::Search]);

    // Verifying at valid timestamp within the signing and key window
    let eval_time = "2026-08-15T12:00:00Z";
    let Ok(()) = verify_signed_component_release(
        &manifest,
        TEST_COMPONENT_BYTES,
        &release,
        &policy,
        eval_time,
    ) else {
        panic!("valid signed component release must verify successfully");
    };
}

/// Scenario 2: Byte-level tampering detection (T24-A001, T24-A002).
#[test]
fn test_tampered_component_bytes_rejected() {
    let (manifest, release, policy) =
        create_signed_bundle(TEST_COMPONENT_BYTES, vec![ComponentCapability::Search]);

    let mut tampered_bytes = TEST_COMPONENT_BYTES.to_vec();
    tampered_bytes.push(0xFF);

    let eval_time = "2026-08-15T12:00:00Z";
    let result =
        verify_signed_component_release(&manifest, &tampered_bytes, &release, &policy, eval_time);

    let Err(PluginError::DigestMismatch { expected, actual }) = result else {
        panic!("tampered component bytes must yield DigestMismatch");
    };
    assert_ne!(expected, actual);
}

/// Scenario 3: Granular capability authorization and least-privilege denial (T24-A002).
#[test]
fn test_capability_authorization_least_privilege() {
    let manifest = create_test_manifest(
        TEST_COMPONENT_BYTES,
        vec![
            ComponentCapability::Search,
            ComponentCapability::MetadataRead,
        ],
    );

    // Granted capabilities pass
    let Ok(()) = authorise_capability(&manifest, ComponentCapability::Search) else {
        panic!("Search capability must be authorised");
    };
    let Ok(()) = authorise_capability(&manifest, ComponentCapability::MetadataRead) else {
        panic!("MetadataRead capability must be authorised");
    };

    // Undeclared capabilities fail closed
    let network_result = authorise_capability(&manifest, ComponentCapability::NetworkRead);
    let Err(PluginError::CapabilityDenied(ComponentCapability::NetworkRead)) = network_result
    else {
        panic!("Undeclared NetworkRead capability must be denied");
    };
}

/// Scenario 4: Cryptographic key revocation enforcement (T24-A001).
#[test]
fn test_revoked_key_fails_closed() {
    let (manifest, release, mut policy) =
        create_signed_bundle(TEST_COMPONENT_BYTES, vec![ComponentCapability::Search]);

    // Revoke the signing key as of 2026-08-10
    policy.revocations.push(ProviderComponentRevocation {
        key_id: "publisher-primary-key-2026".to_string(),
        revoked_at: "2026-08-10T00:00:00Z".to_string(),
        evidence_reference: "Compromised credential drill receipt".to_string(),
    });

    // Evaluation at 2026-08-15 (after revocation) MUST be rejected
    let eval_time = "2026-08-15T00:00:00Z";
    let result = verify_signed_component_release(
        &manifest,
        TEST_COMPONENT_BYTES,
        &release,
        &policy,
        eval_time,
    );

    let Err(PluginError::RevokedKey(key_id)) = result else {
        panic!("Revoked key must be rejected with RevokedKey");
    };
    assert_eq!(key_id, "publisher-primary-key-2026");
}

/// Scenario 5: Validity window boundary enforcement (T24-A001, T24-A005).
#[test]
fn test_validity_window_expired_release_rejected() {
    let (manifest, release, policy) =
        create_signed_bundle(TEST_COMPONENT_BYTES, vec![ComponentCapability::Search]);

    // Evaluation after release expiry (expires 2026-12-01)
    let expired_time = "2026-12-15T00:00:00Z";
    let result = verify_signed_component_release(
        &manifest,
        TEST_COMPONENT_BYTES,
        &release,
        &policy,
        expired_time,
    );

    let Err(PluginError::ReleaseOutsideValidityWindow) = result else {
        panic!("Expired release must fail with ReleaseOutsideValidityWindow");
    };
}

/// Scenario 6: Component byte and ABI verification independence (T24-A001, T24-A006).
#[test]
fn test_component_abi_mismatch_rejected() {
    let mut manifest =
        create_test_manifest(TEST_COMPONENT_BYTES, vec![ComponentCapability::Search]);
    manifest.abi_version = "searchright:provider/search-provider@9.9.9".to_string();

    let result = verify_component(&manifest, TEST_COMPONENT_BYTES);
    let Err(PluginError::AbiMismatch { expected, actual }) = result else {
        panic!("ABI version mismatch must be rejected");
    };
    assert_eq!(expected, PROVIDER_ABI_VERSION);
    assert_eq!(actual, "searchright:provider/search-provider@9.9.9");
}
