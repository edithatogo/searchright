use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, PROVIDER_COMPONENT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// Capability requested by a sandboxed provider component.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentCapability {
    /// Compile or execute a literature search.
    Search,
    /// Retrieve public bibliographic metadata.
    MetadataRead,
    /// Read caller-supplied files.
    InputFileRead,
    /// Write derived artefacts to a caller-approved workspace.
    WorkspaceWrite,
    /// Make HTTPS requests to declared hosts.
    NetworkRead,
    /// Emit metrics and traces without record/full-text content.
    Telemetry,
}

/// Signature algorithm accepted for provider-component release attestations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentSignatureAlgorithm {
    /// Ed25519 over the canonical Searchright component-release message.
    Ed25519,
}

/// One trusted public key and its bounded component namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderComponentTrustKey {
    /// Stable key identifier referenced by release signatures and revocations.
    pub key_id: String,
    /// Verification algorithm for this key.
    pub algorithm: ComponentSignatureAlgorithm,
    /// Base64url-without-padding encoded public key bytes.
    pub public_key: String,
    /// Component identifiers this key is permitted to sign.
    pub component_ids: Vec<String>,
    /// Inclusive RFC 3339 start of the key's validity window.
    pub valid_from: String,
    /// Exclusive RFC 3339 end of the key's validity window.
    pub valid_until: String,
}

/// Revocation evidence for a previously trusted component-signing key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderComponentRevocation {
    /// Revoked key identifier.
    pub key_id: String,
    /// RFC 3339 time from which the key must be rejected.
    pub revoked_at: String,
    /// Durable, non-secret evidence reference for the revocation decision.
    pub evidence_reference: String,
}

/// Reviewed trust and revocation policy for provider-component releases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderComponentTrustPolicy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable policy identifier.
    pub policy_id: String,
    /// Trusted public keys.
    pub trusted_keys: Vec<ProviderComponentTrustKey>,
    /// Explicit revocations. Unknown key identifiers are rejected.
    #[serde(default)]
    pub revocations: Vec<ProviderComponentRevocation>,
}

/// Detached signature binding one reviewed manifest to exact component bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderComponentReleaseSignature {
    /// Contract identifier.
    pub schema_version: String,
    /// Component identifier copied from the signed manifest.
    pub component_id: String,
    /// Component version copied from the signed manifest.
    pub component_version: String,
    /// Canonical BLAKE3 digest of the manifest JSON bytes used by Searchright.
    pub manifest_digest: String,
    /// BLAKE3 digest of the exact component bytes.
    pub component_digest: String,
    /// Trusted key identifier.
    pub key_id: String,
    /// Exact trust-policy identifier used to evaluate the release.
    pub trust_policy_id: String,
    /// Signature algorithm.
    pub algorithm: ComponentSignatureAlgorithm,
    /// Inclusive RFC 3339 signing time.
    pub signed_at: String,
    /// Exclusive RFC 3339 release-signature expiry.
    pub expires_at: String,
    /// Base64url-without-padding encoded detached signature.
    pub signature: String,
}

/// Signed-capability-ready manifest for a WASI provider component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderComponentManifest {
    /// Contract identifier.
    pub schema_version: String,
    /// Globally stable component identifier.
    pub component_id: String,
    /// Semantic component version.
    pub component_version: String,
    /// Searchright WIT world/ABI version.
    pub abi_version: String,
    /// Requested capabilities.
    pub capabilities: Vec<ComponentCapability>,
    /// HTTPS hosts permitted when `network_read` is requested.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Maximum linear memory in mebibytes.
    pub max_memory_mib: u32,
    /// Maximum execution fuel or equivalent host budget.
    pub max_fuel: u64,
    /// BLAKE3 digest of the component bytes.
    pub component_digest: String,
    /// Whether deterministic fixture mode is implemented.
    pub fixture_mode: bool,
    /// Human-readable rationale for requested authority.
    pub authority_rationale: Vec<String>,
}

impl Validate for ProviderComponentManifest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            PROVIDER_COMPONENT_SCHEMA_VERSION,
            "provider_component.schema_version",
        )?;
        require_text(&self.component_id, "provider_component.component_id")?;
        require_text(
            &self.component_version,
            "provider_component.component_version",
        )?;
        require_text(&self.abi_version, "provider_component.abi_version")?;
        if self.capabilities.is_empty() {
            return Err(ContractError::EmptyCollection(
                "provider_component.capabilities",
            ));
        }
        let unique = self.capabilities.iter().collect::<BTreeSet<_>>();
        if unique.len() != self.capabilities.len() {
            return Err(ContractError::Invariant(
                "provider-component capabilities must be unique".to_owned(),
            ));
        }
        if self.max_memory_mib == 0 || self.max_fuel == 0 {
            return Err(ContractError::Invariant(
                "provider-component memory and fuel budgets must be positive".to_owned(),
            ));
        }
        if self.component_digest.len() != 64
            || !self
                .component_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::Invariant(
                "provider-component digest must be a canonical BLAKE3 hexadecimal digest"
                    .to_owned(),
            ));
        }
        if self
            .capabilities
            .contains(&ComponentCapability::NetworkRead)
            && self.allowed_hosts.is_empty()
        {
            return Err(ContractError::Invariant(
                "network-read provider components must declare allowed hosts".to_owned(),
            ));
        }
        if !self
            .capabilities
            .contains(&ComponentCapability::NetworkRead)
            && !self.allowed_hosts.is_empty()
        {
            return Err(ContractError::Invariant(
                "allowed hosts require the network-read capability".to_owned(),
            ));
        }
        if self.allowed_hosts.iter().any(|host| {
            host.trim().is_empty()
                || host.contains('/')
                || host.contains('@')
                || host.parse::<std::net::IpAddr>().is_ok()
        }) {
            return Err(ContractError::Invariant(
                "provider-component hosts must be bare DNS names, not URLs or IP addresses"
                    .to_owned(),
            ));
        }
        if self.authority_rationale.is_empty()
            || self
                .authority_rationale
                .iter()
                .any(|value| value.trim().is_empty())
        {
            return Err(ContractError::EmptyCollection(
                "provider_component.authority_rationale",
            ));
        }
        Ok(())
    }
}

impl Validate for ProviderComponentTrustPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            crate::PROVIDER_COMPONENT_TRUST_POLICY_SCHEMA_VERSION,
            "provider_component_trust_policy.schema_version",
        )?;
        require_text(&self.policy_id, "provider_component_trust_policy.policy_id")?;
        if self.trusted_keys.is_empty() {
            return Err(ContractError::EmptyCollection(
                "provider_component_trust_policy.trusted_keys",
            ));
        }
        let mut key_ids = BTreeSet::new();
        for key in &self.trusted_keys {
            require_text(&key.key_id, "provider_component_trust_key.key_id")?;
            require_text(&key.public_key, "provider_component_trust_key.public_key")?;
            require_base64url(
                &key.public_key,
                43,
                "provider_component_trust_key.public_key",
            )?;
            require_text(&key.valid_from, "provider_component_trust_key.valid_from")?;
            require_text(&key.valid_until, "provider_component_trust_key.valid_until")?;
            if !key_ids.insert(key.key_id.as_str()) {
                return Err(ContractError::Invariant(
                    "provider-component trust key identifiers must be unique".to_owned(),
                ));
            }
            if key.component_ids.is_empty()
                || key
                    .component_ids
                    .iter()
                    .any(|component_id| component_id.trim().is_empty())
                || key.component_ids.iter().collect::<BTreeSet<_>>().len()
                    != key.component_ids.len()
            {
                return Err(ContractError::Invariant(
                    "provider-component trust keys require unique non-empty component identifiers"
                        .to_owned(),
                ));
            }
        }
        let mut revoked_ids = BTreeSet::new();
        for revocation in &self.revocations {
            require_text(&revocation.key_id, "provider_component_revocation.key_id")?;
            require_text(
                &revocation.revoked_at,
                "provider_component_revocation.revoked_at",
            )?;
            require_text(
                &revocation.evidence_reference,
                "provider_component_revocation.evidence_reference",
            )?;
            if !key_ids.contains(revocation.key_id.as_str()) {
                return Err(ContractError::Invariant(
                    "provider-component revocations must reference trusted keys".to_owned(),
                ));
            }
            if !revoked_ids.insert(revocation.key_id.as_str()) {
                return Err(ContractError::Invariant(
                    "provider-component keys may have only one effective revocation".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

impl Validate for ProviderComponentReleaseSignature {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            crate::PROVIDER_COMPONENT_RELEASE_SIGNATURE_SCHEMA_VERSION,
            "provider_component_release_signature.schema_version",
        )?;
        require_text(
            &self.component_id,
            "provider_component_release_signature.component_id",
        )?;
        require_text(
            &self.component_version,
            "provider_component_release_signature.component_version",
        )?;
        require_digest(
            &self.manifest_digest,
            "provider_component_release_signature.manifest_digest",
        )?;
        require_digest(
            &self.component_digest,
            "provider_component_release_signature.component_digest",
        )?;
        require_text(&self.key_id, "provider_component_release_signature.key_id")?;
        require_text(
            &self.trust_policy_id,
            "provider_component_release_signature.trust_policy_id",
        )?;
        require_text(
            &self.signed_at,
            "provider_component_release_signature.signed_at",
        )?;
        require_text(
            &self.expires_at,
            "provider_component_release_signature.expires_at",
        )?;
        require_text(
            &self.signature,
            "provider_component_release_signature.signature",
        )?;
        require_base64url(
            &self.signature,
            86,
            "provider_component_release_signature.signature",
        )?;
        Ok(())
    }
}

fn require_digest(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ContractError::Invariant(format!(
            "{field} must be a canonical BLAKE3 hexadecimal digest"
        )))
    }
}

fn require_base64url(
    value: &str,
    expected_length: usize,
    field: &'static str,
) -> Result<(), ContractError> {
    if value.len() == expected_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        Ok(())
    } else {
        Err(ContractError::Invariant(format!(
            "{field} must be canonical base64url without padding"
        )))
    }
}
