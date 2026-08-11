use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AUDIT_EVENT_SCHEMA_VERSION, ContractError, Validate, require_schema_version, require_text,
};

/// Actor responsible for an audit event.
#[expect(
    clippy::struct_field_names,
    reason = "actor_id and actor_type are stable serialized contract field names"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Actor {
    /// Stable actor identifier or pseudonym.
    pub actor_id: String,
    /// Actor class such as human, agent, CLI, MCP or provider.
    pub actor_type: String,
    /// Optional tool/model/version provenance.
    pub provenance: Option<String>,
}

/// Event before hash-chain fields are assigned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuditEventDraft {
    /// Contract identifier.
    pub schema_version: String,
    /// Event identifier.
    pub event_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Event type.
    pub event_type: String,
    /// RFC 3339 timestamp.
    pub occurred_at: String,
    /// Responsible actor.
    pub actor: Actor,
    /// Versioned event payload.
    pub payload: Value,
}

/// Hash-chained audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    /// Contract identifier.
    pub schema_version: String,
    /// Event identifier.
    pub event_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Event type.
    pub event_type: String,
    /// RFC 3339 timestamp.
    pub occurred_at: String,
    /// Responsible actor.
    pub actor: Actor,
    /// Versioned event payload.
    pub payload: Value,
    /// Previous event hash or `GENESIS`.
    pub previous_hash: String,
    /// BLAKE3 hash over canonical event content and previous hash.
    pub event_hash: String,
}

impl Validate for Actor {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.actor_id, "audit.actor.actor_id")?;
        require_text(&self.actor_type, "audit.actor.actor_type")?;
        if let Some(provenance) = &self.provenance {
            require_text(provenance, "audit.actor.provenance")?;
        }
        Ok(())
    }
}

impl Validate for AuditEventDraft {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            AUDIT_EVENT_SCHEMA_VERSION,
            "audit.schema_version",
        )?;
        require_text(&self.event_id, "audit.event_id")?;
        require_text(&self.review_id, "audit.review_id")?;
        require_text(&self.event_type, "audit.event_type")?;
        require_text(&self.occurred_at, "audit.occurred_at")?;
        self.actor.validate()
    }
}

impl Validate for AuditEvent {
    fn validate(&self) -> Result<(), ContractError> {
        AuditEventDraft {
            schema_version: self.schema_version.clone(),
            event_id: self.event_id.clone(),
            review_id: self.review_id.clone(),
            event_type: self.event_type.clone(),
            occurred_at: self.occurred_at.clone(),
            actor: self.actor.clone(),
            payload: self.payload.clone(),
        }
        .validate()?;
        require_text(&self.previous_hash, "audit.previous_hash")?;
        require_text(&self.event_hash, "audit.event_hash")
    }
}
