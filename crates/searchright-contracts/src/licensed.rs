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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> LicensedAdapterProfile {
        LicensedAdapterProfile {
            schema_version: LICENSED_ADAPTER_SCHEMA_VERSION.to_owned(),
            provider_id: "scopus-byo".to_owned(),
            database: "Scopus".to_owned(),
            platform: "Elsevier APIs".to_owned(),
            dialect: SearchDialect::Scopus,
            credential_environment_variable: "SEARCHRIGHT_SCOPUS_API_KEY".to_owned(),
            live_opt_in_environment_variable: "SEARCHRIGHT_SCOPUS_LIVE".to_owned(),
            allowed_hosts: vec!["api.elsevier.com".to_owned()],
            export_formats: vec!["ris".to_owned(), "csv".to_owned()],
            terms_review: "Caller must hold an applicable API entitlement.".to_owned(),
            persist_raw_responses: false,
        }
    }

    #[test]
    fn valid_profile_passes_validation() {
        let profile = valid_profile();
        let Ok(()) = profile.validate() else {
            panic!("valid profile must pass validation");
        };
    }

    #[test]
    fn invalid_schema_version_fails() {
        let mut profile = valid_profile();
        profile.schema_version = "org.searchright.invalid.v1".to_owned();
        let Err(err) = profile.validate() else {
            panic!("invalid schema version must fail");
        };
        assert!(
            matches!(err, ContractError::Invariant(msg) if msg.contains("licensed_adapter.schema_version"))
        );
    }

    #[test]
    fn empty_provider_id_fails() {
        let mut profile = valid_profile();
        profile.provider_id = "   ".to_owned();
        let Err(err) = profile.validate() else {
            panic!("blank provider id must fail");
        };
        assert!(matches!(
            err,
            ContractError::EmptyField("licensed_adapter.provider_id")
        ));
    }

    #[test]
    fn empty_allowed_hosts_fails() {
        let mut profile = valid_profile();
        profile.allowed_hosts = vec![];
        let Err(err) = profile.validate() else {
            panic!("empty allowed hosts must fail");
        };
        assert!(matches!(err, ContractError::Invariant(_)));
    }

    #[test]
    fn empty_export_formats_fails() {
        let mut profile = valid_profile();
        profile.export_formats = vec![];
        let Err(err) = profile.validate() else {
            panic!("empty export formats must fail");
        };
        assert!(matches!(err, ContractError::Invariant(_)));
    }

    #[test]
    fn disallowed_host_formats_fail() {
        let mut profile = valid_profile();
        profile.allowed_hosts = vec!["api.elsevier.com/v1".to_owned()];
        let Err(err) = profile.validate() else {
            panic!("host with path must fail");
        };
        assert!(matches!(err, ContractError::Invariant(_)));

        let mut profile_ip = valid_profile();
        profile_ip.allowed_hosts = vec!["127.0.0.1".to_owned()];
        let Err(err_ip) = profile_ip.validate() else {
            panic!("IP host must fail");
        };
        assert!(matches!(err_ip, ContractError::Invariant(_)));

        let mut profile_user = valid_profile();
        profile_user.allowed_hosts = vec!["user@api.elsevier.com".to_owned()];
        let Err(err_user) = profile_user.validate() else {
            panic!("host with userinfo must fail");
        };
        assert!(matches!(err_user, ContractError::Invariant(_)));
    }

    #[test]
    fn lowercase_env_vars_fail() {
        let mut profile = valid_profile();
        profile.credential_environment_variable = "searchright_scopus_api_key".to_owned();
        let Err(err) = profile.validate() else {
            panic!("lowercase env var must fail");
        };
        assert!(matches!(err, ContractError::Invariant(_)));

        let mut profile_opt = valid_profile();
        profile_opt.live_opt_in_environment_variable = "searchright_scopus_live".to_owned();
        let Err(err_opt) = profile_opt.validate() else {
            panic!("lowercase opt-in env var must fail");
        };
        assert!(matches!(err_opt, ContractError::Invariant(_)));
    }

    #[test]
    fn persisting_raw_responses_fails() {
        let mut profile = valid_profile();
        profile.persist_raw_responses = true;
        let Err(err) = profile.validate() else {
            panic!("persisting raw responses must fail closed");
        };
        assert!(matches!(err, ContractError::Invariant(_)));
    }
}
