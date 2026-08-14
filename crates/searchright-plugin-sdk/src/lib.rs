//! Host-side capability and integrity checks for provider components.

#![forbid(unsafe_code)]

use aws_lc_rs::signature::{ED25519, UnparsedPublicKey};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use searchright_contracts::{
    ComponentCapability, ComponentSignatureAlgorithm, ProviderComponentManifest,
    ProviderComponentReleaseSignature, ProviderComponentTrustPolicy, Validate,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

/// Current stable WIT world expected by the host.
pub const PROVIDER_ABI_VERSION: &str = "searchright:provider/search-provider@0.1.0";

/// Verify manifest semantics, ABI compatibility and component-byte integrity.
pub fn verify_component(
    manifest: &ProviderComponentManifest,
    component_bytes: &[u8],
) -> Result<(), PluginError> {
    manifest.validate()?;
    if manifest.abi_version != PROVIDER_ABI_VERSION {
        return Err(PluginError::AbiMismatch {
            expected: PROVIDER_ABI_VERSION.to_owned(),
            actual: manifest.abi_version.clone(),
        });
    }
    let observed = blake3::hash(component_bytes).to_hex().to_string();
    if observed != manifest.component_digest {
        return Err(PluginError::DigestMismatch {
            expected: manifest.component_digest.clone(),
            actual: observed,
        });
    }
    Ok(())
}

/// Check whether one capability was explicitly granted by the reviewed manifest.
pub fn authorise_capability(
    manifest: &ProviderComponentManifest,
    capability: ComponentCapability,
) -> Result<(), PluginError> {
    manifest.validate()?;
    if manifest.capabilities.contains(&capability) {
        Ok(())
    } else {
        Err(PluginError::CapabilityDenied(capability))
    }
}

/// Produce a deterministic digest for review and signing workflows.
pub fn manifest_digest(manifest: &ProviderComponentManifest) -> Result<String, PluginError> {
    manifest.validate()?;
    let bytes = serde_json::to_vec(manifest)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Verify a detached release signature against exact component bytes and a reviewed trust policy.
///
/// The signature covers a domain-separated, length-prefixed message containing the signature
/// contract version, component identity/version, ABI, manifest digest, component digest, signing
/// time and expiry. Keys are additionally bounded to component identifiers and validity windows.
pub fn verify_signed_component_release(
    manifest: &ProviderComponentManifest,
    component_bytes: &[u8],
    release: &ProviderComponentReleaseSignature,
    policy: &ProviderComponentTrustPolicy,
    now: &str,
) -> Result<(), PluginError> {
    verify_component(manifest, component_bytes)?;
    release.validate()?;
    policy.validate()?;

    let observed_manifest_digest = manifest_digest(manifest)?;
    if release.component_id != manifest.component_id
        || release.component_version != manifest.component_version
        || release.component_digest != manifest.component_digest
        || release.manifest_digest != observed_manifest_digest
        || release.trust_policy_id != policy.policy_id
    {
        return Err(PluginError::ReleaseBindingMismatch);
    }

    let now = parse_time(now, "current time")?;
    let signed_at = parse_time(&release.signed_at, "release signed_at")?;
    let expires_at = parse_time(&release.expires_at, "release expires_at")?;
    if signed_at > now || expires_at <= now || signed_at >= expires_at {
        return Err(PluginError::ReleaseOutsideValidityWindow);
    }

    let key = policy
        .trusted_keys
        .iter()
        .find(|key| key.key_id == release.key_id)
        .ok_or_else(|| PluginError::UntrustedKey(release.key_id.clone()))?;
    if key.algorithm != release.algorithm || !key.component_ids.contains(&manifest.component_id) {
        return Err(PluginError::KeyAuthorityDenied);
    }
    let key_valid_from = parse_time(&key.valid_from, "key valid_from")?;
    let key_valid_until = parse_time(&key.valid_until, "key valid_until")?;
    if key_valid_from > signed_at
        || key_valid_until <= signed_at
        || key_valid_from >= key_valid_until
    {
        return Err(PluginError::KeyOutsideValidityWindow);
    }
    if let Some(revocation) = policy
        .revocations
        .iter()
        .find(|revocation| revocation.key_id == key.key_id)
    {
        let revoked_at = parse_time(&revocation.revoked_at, "key revoked_at")?;
        if revoked_at <= now {
            return Err(PluginError::RevokedKey(key.key_id.clone()));
        }
    }

    let public_key = BASE64_URL_SAFE_NO_PAD
        .decode(&key.public_key)
        .map_err(|_| PluginError::InvalidPublicKeyEncoding)?;
    let signature = BASE64_URL_SAFE_NO_PAD
        .decode(&release.signature)
        .map_err(|_| PluginError::InvalidSignatureEncoding)?;
    if public_key.len() != 32 {
        return Err(PluginError::InvalidPublicKeyEncoding);
    }
    if signature.len() != 64 {
        return Err(PluginError::InvalidSignatureEncoding);
    }
    match release.algorithm {
        ComponentSignatureAlgorithm::Ed25519 => {
            UnparsedPublicKey::new(&ED25519, public_key)
                .verify(&release_message(manifest, release), &signature)
                .map_err(|_| PluginError::SignatureVerificationFailed)?;
        }
    }
    Ok(())
}

/// Construct the canonical domain-separated message signed by a component publisher.
#[must_use]
pub fn release_message(
    manifest: &ProviderComponentManifest,
    release: &ProviderComponentReleaseSignature,
) -> Vec<u8> {
    let mut message = Vec::new();
    for field in [
        "org.searchright.provider-component-release-signature.message.v1",
        release.schema_version.as_str(),
        release.component_id.as_str(),
        release.component_version.as_str(),
        manifest.abi_version.as_str(),
        release.manifest_digest.as_str(),
        release.component_digest.as_str(),
        release.key_id.as_str(),
        release.trust_policy_id.as_str(),
        release.signed_at.as_str(),
        release.expires_at.as_str(),
    ] {
        let length = u64::try_from(field.len()).unwrap_or(u64::MAX);
        message.extend_from_slice(&length.to_be_bytes());
        message.extend_from_slice(field.as_bytes());
    }
    message
}

fn parse_time(value: &str, field: &'static str) -> Result<OffsetDateTime, PluginError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| PluginError::InvalidTimestamp(field))
}

/// Provider-component policy failure.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Manifest serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// WIT world/ABI was incompatible.
    #[error("provider ABI mismatch: expected `{expected}`, found `{actual}`")]
    AbiMismatch {
        /// WIT world or ABI identifier required by the host.
        expected: String,
        /// WIT world or ABI identifier declared by the component.
        actual: String,
    },
    /// Component bytes did not match the reviewed digest.
    #[error("provider component digest mismatch: expected `{expected}`, found `{actual}`")]
    DigestMismatch {
        /// Reviewed component digest recorded in the manifest.
        expected: String,
        /// Digest calculated from the supplied component bytes.
        actual: String,
    },
    /// A capability was not granted.
    #[error("provider component capability `{0:?}` is not authorised")]
    CapabilityDenied(ComponentCapability),
    /// A release did not bind the supplied manifest and component identity exactly.
    #[error("provider component release does not bind the supplied manifest and component bytes")]
    ReleaseBindingMismatch,
    /// A timestamp was not valid RFC 3339.
    #[error("provider component {0} is not a valid RFC 3339 timestamp")]
    InvalidTimestamp(&'static str),
    /// The release was not yet valid, had expired or had an inverted validity window.
    #[error("provider component release is outside its validity window")]
    ReleaseOutsideValidityWindow,
    /// No trusted key had the release key identifier.
    #[error("provider component release key `{0}` is not trusted")]
    UntrustedKey(String),
    /// The selected key was not allowed to sign this component or algorithm.
    #[error("provider component signing key is not authorised for this component")]
    KeyAuthorityDenied,
    /// The key was not valid when the release was signed.
    #[error("provider component signing key is outside its validity window")]
    KeyOutsideValidityWindow,
    /// The key was revoked by the reviewed trust policy.
    #[error("provider component release key `{0}` is revoked")]
    RevokedKey(String),
    /// The trusted public key was not canonical base64url or had the wrong length.
    #[error("provider component public key encoding is invalid")]
    InvalidPublicKeyEncoding,
    /// The detached signature was not canonical base64url or had the wrong length.
    #[error("provider component signature encoding is invalid")]
    InvalidSignatureEncoding,
    /// Ed25519 verification failed.
    #[error("provider component release signature verification failed")]
    SignatureVerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lc_rs::signature::{Ed25519KeyPair, KeyPair};

    fn manifest(bytes: &[u8]) -> ProviderComponentManifest {
        ProviderComponentManifest {
            schema_version: "org.searchright.provider-component.v1".to_owned(),
            component_id: "io.github.edithatogo.searchright.fixture".to_owned(),
            component_version: "0.1.0".to_owned(),
            abi_version: PROVIDER_ABI_VERSION.to_owned(),
            capabilities: vec![ComponentCapability::Search],
            allowed_hosts: Vec::new(),
            max_memory_mib: 64,
            max_fuel: 1_000_000,
            component_digest: blake3::hash(bytes).to_hex().to_string(),
            fixture_mode: true,
            authority_rationale: vec!["execute deterministic fixture searches".to_owned()],
        }
    }

    #[test]
    fn changed_component_bytes_are_rejected() {
        let component = b"component-a";
        let manifest = manifest(component);
        assert!(verify_component(&manifest, component).is_ok());
        assert!(matches!(
            verify_component(&manifest, b"component-b"),
            Err(PluginError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn undeclared_capability_is_denied() {
        let manifest = manifest(b"component-a");
        assert!(matches!(
            authorise_capability(&manifest, ComponentCapability::NetworkRead),
            Err(PluginError::CapabilityDenied(_))
        ));
    }

    fn signed_release(
        component: &[u8],
    ) -> (
        ProviderComponentManifest,
        ProviderComponentReleaseSignature,
        ProviderComponentTrustPolicy,
    ) {
        let manifest = manifest(component);
        let key_pair = Ed25519KeyPair::from_seed_unchecked(&[7_u8; 32])
            .unwrap_or_else(|error| panic!("test key must be valid: {error:?}"));
        let mut release = ProviderComponentReleaseSignature {
            schema_version: "org.searchright.provider-component-release-signature.v1".to_owned(),
            component_id: manifest.component_id.clone(),
            component_version: manifest.component_version.clone(),
            manifest_digest: manifest_digest(&manifest)
                .unwrap_or_else(|error| panic!("manifest must serialise: {error}")),
            component_digest: manifest.component_digest.clone(),
            key_id: "fixture-signing-key-1".to_owned(),
            trust_policy_id: "fixture-component-trust-policy".to_owned(),
            algorithm: ComponentSignatureAlgorithm::Ed25519,
            signed_at: "2026-08-14T00:00:00Z".to_owned(),
            expires_at: "2026-09-14T00:00:00Z".to_owned(),
            signature: String::new(),
        };
        release.signature =
            BASE64_URL_SAFE_NO_PAD.encode(key_pair.sign(&release_message(&manifest, &release)));
        let policy = ProviderComponentTrustPolicy {
            schema_version: "org.searchright.provider-component-trust-policy.v1".to_owned(),
            policy_id: release.trust_policy_id.clone(),
            trusted_keys: vec![searchright_contracts::ProviderComponentTrustKey {
                key_id: release.key_id.clone(),
                algorithm: ComponentSignatureAlgorithm::Ed25519,
                public_key: BASE64_URL_SAFE_NO_PAD.encode(key_pair.public_key().as_ref()),
                component_ids: vec![manifest.component_id.clone()],
                valid_from: "2026-08-01T00:00:00Z".to_owned(),
                valid_until: "2027-08-01T00:00:00Z".to_owned(),
            }],
            revocations: Vec::new(),
        };
        (manifest, release, policy)
    }

    #[test]
    fn signed_release_is_bound_to_exact_manifest_component_and_time() {
        let component = b"component-a";
        let (manifest, release, policy) = signed_release(component);
        assert!(
            verify_signed_component_release(
                &manifest,
                component,
                &release,
                &policy,
                "2026-08-15T00:00:00Z",
            )
            .is_ok()
        );

        let mut changed = release.clone();
        changed.component_version = "0.1.1".to_owned();
        assert!(matches!(
            verify_signed_component_release(
                &manifest,
                component,
                &changed,
                &policy,
                "2026-08-15T00:00:00Z",
            ),
            Err(PluginError::ReleaseBindingMismatch)
        ));
        assert!(matches!(
            verify_signed_component_release(
                &manifest,
                component,
                &release,
                &policy,
                "2026-10-01T00:00:00Z",
            ),
            Err(PluginError::ReleaseOutsideValidityWindow)
        ));
    }

    #[test]
    fn revoked_or_tampered_release_is_denied() {
        let component = b"component-a";
        let (manifest, mut release, mut policy) = signed_release(component);
        release.signature.replace_range(..1, "A");
        assert!(matches!(
            verify_signed_component_release(
                &manifest,
                component,
                &release,
                &policy,
                "2026-08-15T00:00:00Z",
            ),
            Err(PluginError::SignatureVerificationFailed)
        ));

        let (_, release, _) = signed_release(component);
        policy.revocations = vec![searchright_contracts::ProviderComponentRevocation {
            key_id: release.key_id.clone(),
            revoked_at: "2026-08-14T12:00:00Z".to_owned(),
            evidence_reference: "receipt:fixture-key-revocation".to_owned(),
        }];
        assert!(matches!(
            verify_signed_component_release(
                &manifest,
                component,
                &release,
                &policy,
                "2026-08-15T00:00:00Z",
            ),
            Err(PluginError::RevokedKey(_))
        ));
    }
}
