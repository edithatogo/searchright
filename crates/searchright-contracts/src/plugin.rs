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
    /// HTTPS hosts permitted when network_read is requested.
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
