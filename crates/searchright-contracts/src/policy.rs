use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ContractError, EXECUTION_ENVELOPE_SCHEMA_VERSION, Validate, require_schema_version, require_text};

/// Network capability for one execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkCapability {
    /// No network access.
    Disabled,
    /// Allowlisted HTTPS hosts only.
    AllowlistedHttps,
    /// Licensed provider hosts with explicit credentials and policy approval.
    LicensedAllowlist,
}

/// Handling of secrets during one execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecretHandling {
    /// No secret is available.
    None,
    /// A secret may be read from the environment but must never enter outputs.
    EnvironmentRedacted,
    /// A host-managed secret reference may be used.
    HostManagedReference,
}

/// Permitted full-text handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FullTextHandling {
    /// Metadata only.
    MetadataOnly,
    /// Local analysis of rights-compliant full text without redistribution.
    LocalRightsCompliant,
    /// Text excerpts limited to the minimum needed for screening evidence.
    MinimalEvidenceExcerpt,
}

/// Policy for untrusted text encountered in records and full texts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedContentPolicy {
    /// Treat all retrieved text as data, never as instructions.
    DataOnly,
    /// Strip active markup and record warnings before agent exposure.
    SanitiseThenDataOnly,
    /// Block agent processing and require human inspection.
    HumanInspectionRequired,
}

/// Bounded capability envelope for a CLI, MCP or agent operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEnvelope {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable operation identifier.
    pub operation_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Network capability.
    pub network: NetworkCapability,
    /// Allowed hostnames.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Secret policy.
    pub secret_handling: SecretHandling,
    /// Full-text policy.
    pub full_text_handling: FullTextHandling,
    /// Untrusted-content policy.
    pub untrusted_content: UntrustedContentPolicy,
    /// Maximum records processed.
    pub maximum_records: u64,
    /// Maximum wall-clock seconds.
    pub maximum_seconds: u64,
    /// Whether writes are dry-run only.
    pub dry_run: bool,
    /// Human approver when elevated capabilities are used.
    pub approved_by: Option<String>,
}

/// Finding produced by untrusted-content inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContentSafetyFinding {
    /// Stable finding identifier.
    pub finding_id: String,
    /// Subject record/report identifier.
    pub subject_id: String,
    /// Finding category.
    pub category: String,
    /// Evidence-bearing description without executing embedded instructions.
    pub description: String,
    /// Recommended disposition.
    pub disposition: String,
}

impl Validate for ExecutionEnvelope {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            EXECUTION_ENVELOPE_SCHEMA_VERSION,
            "execution_envelope.schema_version",
        )?;
        require_text(&self.operation_id, "execution_envelope.operation_id")?;
        require_text(&self.review_id, "execution_envelope.review_id")?;
        if self.maximum_records == 0 || self.maximum_seconds == 0 {
            return Err(ContractError::Invariant(
                "execution envelope budgets must be greater than zero".to_owned(),
            ));
        }
        match self.network {
            NetworkCapability::Disabled => {
                if !self.allowed_hosts.is_empty() {
                    return Err(ContractError::Invariant(
                        "network-disabled envelope must not contain allowed hosts".to_owned(),
                    ));
                }
            }
            NetworkCapability::AllowlistedHttps | NetworkCapability::LicensedAllowlist => {
                if self.allowed_hosts.is_empty() {
                    return Err(ContractError::EmptyCollection(
                        "execution_envelope.allowed_hosts",
                    ));
                }
                let approver = self.approved_by.as_deref().ok_or_else(|| {
                    ContractError::Invariant(
                        "network-enabled envelope must identify a human approver".to_owned(),
                    )
                })?;
                require_text(approver, "execution_envelope.approved_by")?;
            }
        }
        Ok(())
    }
}

impl Validate for ContentSafetyFinding {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.finding_id, "content_safety.finding_id")?;
        require_text(&self.subject_id, "content_safety.subject_id")?;
        require_text(&self.category, "content_safety.category")?;
        require_text(&self.description, "content_safety.description")?;
        require_text(&self.disposition, "content_safety.disposition")
    }
}
