//! Operational reliability, telemetry, backup and incident contracts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    BACKUP_MANIFEST_SCHEMA_VERSION, COMPONENT_HEALTH_SCHEMA_VERSION, ContractError,
    INCIDENT_RECORD_SCHEMA_VERSION, TELEMETRY_POLICY_SCHEMA_VERSION, Validate,
    require_schema_version, require_text,
};

/// Operational health state for one component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    /// Component is healthy and ready.
    Healthy,
    /// Component is degraded but may serve bounded operations.
    Degraded,
    /// Component is not ready for service.
    Unhealthy,
    /// Component was intentionally disabled.
    Disabled,
}

/// Health observation for one Searchright component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ComponentHealth {
    /// Contract identifier.
    pub schema_version: String,
    /// Component name.
    pub component: String,
    /// Current state.
    pub state: HealthState,
    /// Stable diagnostic codes.
    #[serde(default)]
    pub diagnostics: Vec<String>,
    /// Whether requests may be accepted.
    pub ready: bool,
    /// Observation time or deterministic source epoch.
    pub observed_at: String,
}

/// Explicit telemetry policy; telemetry is disabled unless approved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TelemetryPolicy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable policy identifier.
    pub policy_id: String,
    /// Whether telemetry is enabled.
    pub enabled: bool,
    /// Optional approved collector endpoint.
    pub endpoint: Option<String>,
    /// Explicitly permitted attribute names.
    #[serde(default)]
    pub attribute_allowlist: Vec<String>,
    /// Attributes that may never be emitted.
    pub prohibited_attributes: Vec<String>,
    /// Sampling rate per million events.
    pub sampling_per_million: u32,
    /// Maximum retention period.
    pub retention_days: u32,
    /// Human or institutional approver.
    pub approved_by: Option<String>,
}

/// Kind of backup represented by a manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackupKind {
    /// Full logical backup.
    Full,
    /// Incremental backup chained to a parent.
    Incremental,
    /// Export-only research object.
    ResearchObject,
}

/// Content-addressed backup manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BackupManifest {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable backup identifier.
    pub backup_id: String,
    /// Review or tenant scope.
    pub scope_id: String,
    /// Backup kind.
    pub kind: BackupKind,
    /// Optional parent backup for an incremental chain.
    pub parent_backup_id: Option<String>,
    /// Digest algorithm name.
    pub digest_algorithm: String,
    /// Lowercase hexadecimal digest.
    pub digest: String,
    /// Whether the payload is encrypted at rest.
    pub encrypted: bool,
    /// Key reference only; never key material.
    pub key_reference: Option<String>,
    /// Included logical content classes.
    pub content_classes: Vec<String>,
    /// Creation time or deterministic source epoch.
    pub created_at: String,
    /// Required retention period.
    pub retention_days: u32,
    /// Whether a restore rehearsal is required before promotion.
    pub restore_test_required: bool,
}

/// Incident severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    /// No user impact; tracked for learning.
    Informational,
    /// Limited or recoverable degradation.
    Low,
    /// Material service or integrity impact.
    Medium,
    /// Severe integrity, availability or confidentiality impact.
    High,
    /// Critical safety, integrity or broad confidentiality impact.
    Critical,
}

/// Immutable operational incident record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentRecord {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable incident identifier.
    pub incident_id: String,
    /// Severity.
    pub severity: IncidentSeverity,
    /// Detection time.
    pub detected_at: String,
    /// Affected components.
    pub components: Vec<String>,
    /// User-visible impact summary.
    pub impact: String,
    /// Containment actions.
    pub containment: Vec<String>,
    /// Whether data exposure is suspected.
    pub data_exposure_suspected: bool,
    /// Current incident status.
    pub status: String,
    /// Whether a human post-incident review is required.
    pub postmortem_required: bool,
}

impl Validate for ComponentHealth {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            COMPONENT_HEALTH_SCHEMA_VERSION,
            "component_health.schema_version",
        )?;
        require_text(&self.component, "component_health.component")?;
        require_text(&self.observed_at, "component_health.observed_at")?;
        if self.ready && !matches!(self.state, HealthState::Healthy | HealthState::Degraded) {
            return Err(ContractError::Invariant(
                "unhealthy or disabled components cannot be ready".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for TelemetryPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            TELEMETRY_POLICY_SCHEMA_VERSION,
            "telemetry_policy.schema_version",
        )?;
        require_text(&self.policy_id, "telemetry_policy.policy_id")?;
        if !self.enabled {
            if self.endpoint.is_some()
                || self.sampling_per_million != 0
                || self.approved_by.is_some()
            {
                return Err(ContractError::Invariant(
                    "disabled telemetry must not declare an endpoint, sampling or approver"
                        .to_owned(),
                ));
            }
        } else if self.endpoint.is_none() || self.approved_by.as_deref().is_none_or(str::is_empty) {
            return Err(ContractError::Invariant(
                "enabled telemetry requires endpoint and human approval".to_owned(),
            ));
        }
        if self.sampling_per_million > 1_000_000 || self.retention_days > 365 {
            return Err(ContractError::Invariant(
                "telemetry sampling or retention exceeds bounded policy".to_owned(),
            ));
        }
        for prohibited in &self.prohibited_attributes {
            if self.attribute_allowlist.contains(prohibited) {
                return Err(ContractError::Invariant(format!(
                    "telemetry attribute `{prohibited}` is both allowed and prohibited"
                )));
            }
        }
        Ok(())
    }
}

impl Validate for BackupManifest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            BACKUP_MANIFEST_SCHEMA_VERSION,
            "backup_manifest.schema_version",
        )?;
        require_text(&self.backup_id, "backup_manifest.backup_id")?;
        require_text(&self.scope_id, "backup_manifest.scope_id")?;
        require_text(&self.digest_algorithm, "backup_manifest.digest_algorithm")?;
        require_text(&self.digest, "backup_manifest.digest")?;
        require_text(&self.created_at, "backup_manifest.created_at")?;
        if self.content_classes.is_empty() || self.retention_days == 0 {
            return Err(ContractError::Invariant(
                "backup requires content classes and positive retention".to_owned(),
            ));
        }
        if self.encrypted != self.key_reference.is_some() {
            return Err(ContractError::Invariant("encrypted backups require a key reference and unencrypted backups must not declare one".to_owned()));
        }
        if matches!(self.kind, BackupKind::Incremental) != self.parent_backup_id.is_some() {
            return Err(ContractError::Invariant(
                "only incremental backups require a parent".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for IncidentRecord {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            INCIDENT_RECORD_SCHEMA_VERSION,
            "incident_record.schema_version",
        )?;
        require_text(&self.incident_id, "incident_record.incident_id")?;
        require_text(&self.detected_at, "incident_record.detected_at")?;
        require_text(&self.impact, "incident_record.impact")?;
        require_text(&self.status, "incident_record.status")?;
        if self.components.is_empty() || self.containment.is_empty() {
            return Err(ContractError::EmptyCollection(
                "incident_record.components_or_containment",
            ));
        }
        if matches!(
            self.severity,
            IncidentSeverity::High | IncidentSeverity::Critical
        ) && !self.postmortem_required
        {
            return Err(ContractError::Invariant(
                "high or critical incidents require a postmortem".to_owned(),
            ));
        }
        Ok(())
    }
}
