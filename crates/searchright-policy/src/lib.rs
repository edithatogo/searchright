//! Runtime capability and untrusted-content policy checks.

#![forbid(unsafe_code)]

use searchright_contracts::{
    ContentSafetyFinding, ExecutionEnvelope, NetworkCapability, UntrustedContentPolicy, Validate,
};

/// Validate a requested HTTPS endpoint against an execution envelope.
pub fn authorise_endpoint(
    envelope: &ExecutionEnvelope,
    endpoint: &url::Url,
) -> Result<(), PolicyError> {
    envelope.validate()?;
    if envelope.network == NetworkCapability::Disabled {
        return Err(PolicyError::NetworkDisabled);
    }
    if endpoint.scheme() != "https" {
        return Err(PolicyError::InsecureScheme(endpoint.scheme().to_owned()));
    }
    let host = endpoint
        .host_str()
        .ok_or_else(|| PolicyError::MissingHost(endpoint.to_string()))?;
    if !envelope.allowed_hosts.iter().any(|allowed| allowed == host) {
        return Err(PolicyError::HostNotAllowed(host.to_owned()));
    }
    Ok(())
}

/// Inspect untrusted text for instruction-like or active-content markers.
#[must_use]
pub fn inspect_untrusted_text(
    subject_id: &str,
    text: &str,
    policy: UntrustedContentPolicy,
) -> Vec<ContentSafetyFinding> {
    let lower = text.to_ascii_lowercase();
    let patterns = [
        ("instruction_like", "ignore previous instructions"),
        ("instruction_like", "system prompt"),
        ("secret_request", "api key"),
        ("active_markup", "<script"),
        ("active_markup", "javascript:"),
        ("tool_invocation", "call_tool"),
    ];
    patterns
        .iter()
        .enumerate()
        .filter(|(_, (_, pattern))| lower.contains(pattern))
        .map(|(index, (category, pattern))| ContentSafetyFinding {
            finding_id: format!("content-{subject_id}-{index}"),
            subject_id: subject_id.to_owned(),
            category: (*category).to_owned(),
            description: format!("untrusted text contains marker `{pattern}`"),
            disposition: match policy {
                UntrustedContentPolicy::DataOnly => {
                    "retain as inert data and prohibit instruction execution".to_owned()
                }
                UntrustedContentPolicy::SanitiseThenDataOnly => {
                    "remove active markup, retain an audit warning and expose only inert text"
                        .to_owned()
                }
                UntrustedContentPolicy::HumanInspectionRequired => {
                    "block agent processing pending human inspection".to_owned()
                }
            },
        })
        .collect()
}

/// Capability-policy error.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Network capability is disabled.
    #[error("network access is disabled by the execution envelope")]
    NetworkDisabled,
    /// Endpoint is not HTTPS.
    #[error("endpoint scheme `{0}` is not permitted")]
    InsecureScheme(String),
    /// Endpoint did not include a hostname.
    #[error("endpoint `{0}` has no hostname")]
    MissingHost(String),
    /// Host was not on the explicit allowlist.
    #[error("endpoint host `{0}` is not allowlisted")]
    HostNotAllowed(String),
}
