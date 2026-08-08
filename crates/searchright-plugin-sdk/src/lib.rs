//! Host-side capability and integrity checks for provider components.

#![forbid(unsafe_code)]

use searchright_contracts::{
    ComponentCapability, ProviderComponentManifest, Validate,
};

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
    AbiMismatch { expected: String, actual: String },
    /// Component bytes did not match the reviewed digest.
    #[error("provider component digest mismatch: expected `{expected}`, found `{actual}`")]
    DigestMismatch { expected: String, actual: String },
    /// A capability was not granted.
    #[error("provider component capability `{0:?}` is not authorised")]
    CapabilityDenied(ComponentCapability),
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
