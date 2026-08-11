use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, LICENSED_ADAPTER_SCHEMA_VERSION, SearchDialect, Validate,
    require_schema_version, require_text,
};

/// Contract for a bring-your-own-access licensed database adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LicensedAdapterProfile {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable provider identifier.
    pub provider_id: String,
    /// Database/resource name.
    pub database: String,
    /// Platform/interface name.
    pub platform: String,
    /// Query dialect produced by the compiler.
    pub dialect: SearchDialect,
    /// Environment variable whose presence enables caller-supplied authentication.
    pub credential_environment_variable: String,
    /// Explicit live-execution opt-in environment variable.
    pub live_opt_in_environment_variable: String,
    /// Allowed HTTPS hosts.
    pub allowed_hosts: Vec<String>,
    /// Supported lawful export formats.
    pub export_formats: Vec<String>,
    /// Terms/licence review note.
    pub terms_review: String,
    /// Whether Searchright persists raw licensed responses by default.
    pub persist_raw_responses: bool,
}

impl Validate for LicensedAdapterProfile {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            LICENSED_ADAPTER_SCHEMA_VERSION,
            "licensed_adapter.schema_version",
        )?;
        for (field, value) in [
            ("licensed_adapter.provider_id", self.provider_id.as_str()),
            ("licensed_adapter.database", self.database.as_str()),
            ("licensed_adapter.platform", self.platform.as_str()),
            (
                "licensed_adapter.credential_environment_variable",
                self.credential_environment_variable.as_str(),
            ),
            (
                "licensed_adapter.live_opt_in_environment_variable",
                self.live_opt_in_environment_variable.as_str(),
            ),
            ("licensed_adapter.terms_review", self.terms_review.as_str()),
        ] {
            require_text(value, field)?;
        }
        if self.allowed_hosts.is_empty() || self.export_formats.is_empty() {
            return Err(ContractError::Invariant(
                "licensed adapters require allowed hosts and at least one export format".to_owned(),
            ));
        }
        if self.allowed_hosts.iter().any(|host| {
            host.trim().is_empty()
                || host.contains('/')
                || host.contains('@')
                || host.parse::<std::net::IpAddr>().is_ok()
        }) {
            return Err(ContractError::Invariant(
                "licensed-adapter hosts must be bare DNS names".to_owned(),
            ));
        }
        if !self
            .credential_environment_variable
            .chars()
            .all(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
            })
            || !self
                .live_opt_in_environment_variable
                .chars()
                .all(|character| {
                    character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
                })
        {
            return Err(ContractError::Invariant(
                "licensed-adapter environment variables must use uppercase ASCII names".to_owned(),
            ));
        }
        if self.persist_raw_responses {
            return Err(ContractError::Invariant(
                "licensed adapters must not persist raw responses by default".to_owned(),
            ));
        }
        Ok(())
    }
}
