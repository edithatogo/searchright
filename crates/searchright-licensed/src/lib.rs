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
    if !endpoint_url.username().is_empty() || endpoint_url.password().is_some() {
        return Err(LicensedError::CredentialBearingEndpoint);
    }
    let host = endpoint_url
        .host_str()
        .ok_or(LicensedError::MissingEndpointHost)?;
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
        Some("1" | "true" | "TRUE" | "YES" | "yes")
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
        endpoint: endpoint_url.origin().ascii_serialization(),
        compilation_hash: strategy.compilation_hash.clone(),
        credential_present,
        live_opt_in,
        blockers,
    })
}

/// Alias for `plan_request` conforming to licensed adapter naming conventions.
pub use plan_request as plan_licensed_request;

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
    #[error("licensed endpoint has no host")]
    MissingEndpointHost,
    /// Endpoint embedded credentials in URL user information.
    #[error("licensed endpoint must not embed credentials")]
    CredentialBearingEndpoint,
    /// Endpoint host was not allowed.
    #[error("licensed endpoint host `{0}` is not allowlisted")]
    HostNotAllowed(String),
    /// Strategy dialect did not match the profile.
    #[error("compiled strategy dialect does not match the licensed adapter profile")]
    DialectMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (LicensedAdapterProfile, CompiledStrategy) {
        let Ok(profile) = noyalib::from_str::<LicensedAdapterProfile>(include_str!(
            "../../../contracts/examples/licensed-adapter.yaml"
        )) else {
            panic!("licensed adapter fixture must parse");
        };
        let Ok(mut strategy) = noyalib::from_str::<CompiledStrategy>(include_str!(
            "../../../contracts/examples/compiled-strategy.yaml"
        )) else {
            panic!("compiled strategy fixture must parse");
        };
        strategy.dialect = profile.dialect.clone();
        (profile, strategy)
    }

    #[test]
    fn request_plan_exposes_only_the_endpoint_origin() {
        let (profile, strategy) = inputs();
        let secret = "TRACK09_SENTINEL_SECRET";
        let Ok(plan) = plan_request(
            &profile,
            &strategy,
            &format!("https://embase.com/search?api_key={secret}#private"),
        ) else {
            panic!("request plan must be generated");
        };
        assert_eq!(plan.endpoint, "https://embase.com");
        let Ok(serialized) = serde_json::to_string(&plan) else {
            panic!("request plan must serialize");
        };
        assert!(!serialized.contains(secret));
    }

    #[test]
    fn credential_bearing_endpoint_is_rejected_without_reflection() {
        let (profile, strategy) = inputs();
        let secret = "TRACK09_SENTINEL_SECRET";
        let result = plan_request(
            &profile,
            &strategy,
            &format!("https://user:{secret}@embase.com/search"),
        );
        let Err(error) = result else {
            panic!("credential-bearing endpoint must fail closed");
        };
        assert!(matches!(error, LicensedError::CredentialBearingEndpoint));
        assert!(!error.to_string().contains(secret));
    }

    #[test]
    fn insecure_endpoint_is_rejected() {
        let (profile, strategy) = inputs();
        let result = plan_request(&profile, &strategy, "http://embase.com/search");
        let Err(error) = result else {
            panic!("insecure endpoint must fail closed");
        };
        assert!(matches!(error, LicensedError::InsecureEndpoint));
    }

    #[test]
    fn unallowlisted_host_is_rejected() {
        let (profile, strategy) = inputs();
        let result = plan_request(
            &profile,
            &strategy,
            "https://unauthorized-domain.com/search",
        );
        let Err(error) = result else {
            panic!("unallowlisted host must fail closed");
        };
        assert!(matches!(error, LicensedError::HostNotAllowed(_)));
    }

    #[test]
    fn dialect_mismatch_is_rejected() {
        let (profile, mut strategy) = inputs();
        strategy.dialect = searchright_contracts::SearchDialect::PubMed;
        let result = plan_request(&profile, &strategy, "https://embase.com/search");
        let Err(error) = result else {
            panic!("dialect mismatch must fail closed");
        };
        assert!(matches!(error, LicensedError::DialectMismatch));
    }

    #[test]
    fn strategy_review_required_adds_blocker() {
        let (profile, mut strategy) = inputs();
        strategy.review_required = true;
        let Ok(plan) = plan_request(&profile, &strategy, "https://embase.com/search") else {
            panic!("request plan must succeed with review blocker");
        };
        assert!(
            plan.blockers
                .iter()
                .any(|b| b.contains("human translation review"))
        );
    }

    #[test]
    fn plan_licensed_request_alias_functions_identically() {
        let (profile, strategy) = inputs();
        let Ok(plan) = plan_licensed_request(&profile, &strategy, "https://embase.com/search")
        else {
            panic!("plan_licensed_request alias must succeed");
        };
        assert_eq!(plan.provider_id, "embase-byo");
        assert_eq!(plan.endpoint, "https://embase.com");
    }
}
