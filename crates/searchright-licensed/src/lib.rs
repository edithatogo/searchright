//! Licensed-source request planning without bundled access, credentials or scraping.

#![forbid(unsafe_code)]

use schemars::JsonSchema;
use searchright_contracts::{CompiledStrategy, LicensedAdapterProfile, Validate};
use serde::{Deserialize, Serialize};

/// Redacted plan that can be reviewed before any licensed network operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LicensedRequestPlan {
    /// Provider profile identifier.
    pub provider_id: String,
    /// Database/resource name.
    pub database: String,
    /// Platform/interface name.
    pub platform: String,
    /// Redacted HTTPS endpoint.
    pub endpoint: String,
    /// Query compilation hash, not credentials.
    pub compilation_hash: String,
    /// Whether the credential environment variable is present.
    pub credential_present: bool,
    /// Whether the explicit live opt-in is enabled.
    pub live_opt_in: bool,
    /// Caller-facing blockers.
    pub blockers: Vec<String>,
}

/// Build a redacted plan without reading, returning or logging the credential value.
pub fn plan_request(
    profile: &LicensedAdapterProfile,
    strategy: &CompiledStrategy,
    endpoint: &str,
) -> Result<LicensedRequestPlan, LicensedError> {
    profile.validate()?;
    let endpoint_url = url::Url::parse(endpoint)?;
    if endpoint_url.scheme() != "https" {
        return Err(LicensedError::InsecureEndpoint);
    }
    let host = endpoint_url
        .host_str()
        .ok_or_else(|| LicensedError::MissingEndpointHost(endpoint.to_owned()))?;
    if !profile.allowed_hosts.iter().any(|allowed| allowed == host) {
        return Err(LicensedError::HostNotAllowed(host.to_owned()));
    }
    if strategy.dialect != profile.dialect {
        return Err(LicensedError::DialectMismatch);
    }
    let credential_present = std::env::var_os(&profile.credential_environment_variable).is_some();
    let live_opt_in = matches!(
        std::env::var(&profile.live_opt_in_environment_variable)
            .ok()
            .as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("YES") | Some("yes")
    );
    let mut blockers = Vec::new();
    if !credential_present {
        blockers.push(format!(
            "credential environment variable `{}` is absent",
            profile.credential_environment_variable
        ));
    }
    if !live_opt_in {
        blockers.push(format!(
            "explicit live opt-in `{}` is not enabled",
            profile.live_opt_in_environment_variable
        ));
    }
    if strategy.review_required {
        blockers.push("compiled strategy still requires human translation review".to_owned());
    }
    Ok(LicensedRequestPlan {
        provider_id: profile.provider_id.clone(),
        database: profile.database.clone(),
        platform: profile.platform.clone(),
        endpoint: endpoint_url.to_string(),
        compilation_hash: strategy.compilation_hash.clone(),
        credential_present,
        live_opt_in,
        blockers,
    })
}

/// Licensed-adapter planning failure.
#[derive(Debug, thiserror::Error)]
pub enum LicensedError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Endpoint URL was malformed.
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// Endpoint must use HTTPS.
    #[error("licensed endpoint must use HTTPS")]
    InsecureEndpoint,
    /// Endpoint omitted its host.
    #[error("licensed endpoint `{0}` has no host")]
    MissingEndpointHost(String),
    /// Endpoint host was not allowed.
    #[error("licensed endpoint host `{0}` is not allowlisted")]
    HostNotAllowed(String),
    /// Strategy dialect did not match the profile.
    #[error("compiled strategy dialect does not match the licensed adapter profile")]
    DialectMismatch,
}
