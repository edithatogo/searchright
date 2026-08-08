use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, LIVING_UPDATE_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Update-run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpdateRunStatus {
    /// Run was planned but not started.
    Planned,
    /// Run is executing.
    Running,
    /// Run completed successfully.
    Completed,
    /// Run failed and requires review.
    Failed,
    /// Run was intentionally cancelled.
    Cancelled,
}

/// High-water mark for one provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCursor {
    /// Provider identifier.
    pub provider_id: String,
    /// Cursor strategy, for example date, opaque token, offset or snapshot.
    pub cursor_kind: String,
    /// Redacted cursor value.
    pub value: String,
    /// Inclusive retrieval boundary when applicable.
    pub retrieved_through: Option<String>,
}

/// Change classification for one bibliographic record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordChangeKind {
    /// Newly observed record.
    Added,
    /// Existing record changed materially.
    Updated,
    /// Previously observed record is no longer returned.
    MissingFromSource,
    /// Record was merged into a canonical duplicate cluster.
    Merged,
    /// Record was restored after previously being missing.
    Restored,
}

/// One record change between runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecordChange {
    /// Stable record identifier.
    pub record_id: String,
    /// Change type.
    pub kind: RecordChangeKind,
    /// Prior content digest when available.
    pub before_digest: Option<String>,
    /// Current content digest when available.
    pub after_digest: Option<String>,
    /// Evidence-bearing explanation.
    pub note: String,
}

/// Immutable lineage entry for a living-review update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LivingUpdateRun {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Stable run identifier.
    pub run_id: String,
    /// Parent run identifier, absent for the first run.
    pub parent_run_id: Option<String>,
    /// Run state.
    pub status: UpdateRunStatus,
    /// RFC 3339 start time.
    pub started_at: String,
    /// RFC 3339 completion time when finished.
    pub completed_at: Option<String>,
    /// Search/protocol version applied.
    pub protocol_version: String,
    /// Provider cursors before execution.
    #[serde(default)]
    pub cursors_before: Vec<UpdateCursor>,
    /// Provider cursors after successful execution.
    #[serde(default)]
    pub cursors_after: Vec<UpdateCursor>,
    /// Changed records.
    #[serde(default)]
    pub changes: Vec<RecordChange>,
    /// Whether human approval is required before downstream screening changes.
    pub requires_human_release: bool,
    /// Identifier of a run superseded by this completed run.
    pub supersedes_run_id: Option<String>,
}

impl Validate for UpdateCursor {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.provider_id, "living_update.cursor.provider_id")?;
        require_text(&self.cursor_kind, "living_update.cursor.cursor_kind")?;
        require_text(&self.value, "living_update.cursor.value")?;
        if let Some(boundary) = &self.retrieved_through {
            require_text(boundary, "living_update.cursor.retrieved_through")?;
        }
        Ok(())
    }
}

impl Validate for RecordChange {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.record_id, "living_update.change.record_id")?;
        require_text(&self.note, "living_update.change.note")?;
        if self.before_digest.is_none() && self.after_digest.is_none() {
            return Err(ContractError::Invariant(
                "record change must contain at least one content digest".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for LivingUpdateRun {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            LIVING_UPDATE_SCHEMA_VERSION,
            "living_update.schema_version",
        )?;
        require_text(&self.review_id, "living_update.review_id")?;
        require_text(&self.run_id, "living_update.run_id")?;
        require_text(&self.started_at, "living_update.started_at")?;
        require_text(&self.protocol_version, "living_update.protocol_version")?;
        if self.parent_run_id.as_deref() == Some(self.run_id.as_str()) {
            return Err(ContractError::Invariant(
                "living update cannot name itself as parent".to_owned(),
            ));
        }
        if self.supersedes_run_id.as_deref() == Some(self.run_id.as_str()) {
            return Err(ContractError::Invariant(
                "living update cannot supersede itself".to_owned(),
            ));
        }
        match self.status {
            UpdateRunStatus::Completed => {
                if self.completed_at.is_none() {
                    return Err(ContractError::Invariant(
                        "completed living update requires completed_at".to_owned(),
                    ));
                }
            }
            UpdateRunStatus::Running | UpdateRunStatus::Planned => {
                if self.completed_at.is_some() {
                    return Err(ContractError::Invariant(
                        "unfinished living update must not contain completed_at".to_owned(),
                    ));
                }
            }
            UpdateRunStatus::Failed | UpdateRunStatus::Cancelled => {}
        }
        let mut providers = BTreeSet::new();
        for cursor in &self.cursors_after {
            cursor.validate()?;
            if !providers.insert(cursor.provider_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "living update contains duplicate after-cursor for `{}`",
                    cursor.provider_id
                )));
            }
        }
        for cursor in &self.cursors_before {
            cursor.validate()?;
        }
        let mut records = BTreeSet::new();
        for change in &self.changes {
            change.validate()?;
            if !records.insert(change.record_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "living update contains duplicate record change `{}`",
                    change.record_id
                )));
            }
        }
        Ok(())
    }
}
