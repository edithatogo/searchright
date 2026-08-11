//! Operational reliability policy for Searchright deployments.

#![forbid(unsafe_code)]

use searchright_contracts::{
    AccessDecision, BackupManifest, ComponentHealth, HealthState, TelemetryPolicy, Validate,
};

/// Return whether every required component is ready without hiding degradation.
pub fn readiness(components: &[ComponentHealth]) -> Result<Readiness, OpsError> {
    if components.is_empty() {
        return Err(OpsError::NoComponents);
    }
    for component in components {
        component.validate()?;
    }
    let unhealthy = components
        .iter()
        .filter(|component| !component.ready)
        .map(|component| component.component.clone())
        .collect::<Vec<_>>();
    let degraded = components
        .iter()
        .filter(|component| component.state == HealthState::Degraded)
        .map(|component| component.component.clone())
        .collect::<Vec<_>>();
    Ok(Readiness {
        ready: unhealthy.is_empty(),
        unhealthy,
        degraded,
    })
}

/// Authorise one telemetry attribute under an explicit policy.
pub fn authorise_telemetry_attribute(
    policy: &TelemetryPolicy,
    attribute: &str,
) -> Result<(), OpsError> {
    policy.validate()?;
    if !policy.enabled {
        return Err(OpsError::TelemetryDisabled);
    }
    if policy
        .prohibited_attributes
        .iter()
        .any(|item| item == attribute)
    {
        return Err(OpsError::TelemetryAttributeProhibited(attribute.to_owned()));
    }
    if !policy
        .attribute_allowlist
        .iter()
        .any(|item| item == attribute)
    {
        return Err(OpsError::TelemetryAttributeNotAllowed(attribute.to_owned()));
    }
    Ok(())
}

/// Require a valid encrypted backup and an authorised operator before restore.
pub fn authorise_restore(
    backup: &BackupManifest,
    operator: &AccessDecision,
) -> Result<(), OpsError> {
    backup.validate()?;
    operator.validate()?;
    if !backup.encrypted {
        return Err(OpsError::UnencryptedRestore);
    }
    if !operator.permitted {
        return Err(OpsError::RestoreNotAuthorised);
    }
    Ok(())
}

/// Aggregated readiness result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Readiness {
    /// Whether all required components may accept work.
    pub ready: bool,
    /// Components that are not ready.
    pub unhealthy: Vec<String>,
    /// Ready components currently operating in degraded mode.
    pub degraded: Vec<String>,
}

/// Operational policy error.
#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// No components were supplied.
    #[error("readiness requires at least one component")]
    NoComponents,
    /// Telemetry is disabled.
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    /// Attribute is explicitly prohibited.
    #[error("telemetry attribute `{0}` is prohibited")]
    TelemetryAttributeProhibited(String),
    /// Attribute is not allowlisted.
    #[error("telemetry attribute `{0}` is not allowlisted")]
    TelemetryAttributeNotAllowed(String),
    /// Backup is not encrypted.
    #[error("restore of an unencrypted backup is prohibited")]
    UnencryptedRestore,
    /// Operator is not authorised.
    #[error("restore operator is not authorised")]
    RestoreNotAuthorised,
}
