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

/// Living-review update strategy classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStrategy {
    /// Date-window bounded query strategy.
    Date,
    /// Continuation cursor or opaque provider token.
    Cursor,
    /// High-water mark identifier or timestamp strategy.
    HighWaterMark,
    /// Full replay with deduplication against prior run snapshots.
    Snapshot,
}

impl UpdateStrategy {
    /// Canonical string identifier for this update strategy.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Cursor => "cursor",
            Self::HighWaterMark => "high_water_mark",
            Self::Snapshot => "snapshot",
        }
    }
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

impl UpdateCursor {
    /// Classify the cursor kind into a structured `UpdateStrategy` if known.
    #[must_use]
    pub fn strategy(&self) -> Option<UpdateStrategy> {
        match self.cursor_kind.as_str() {
            "date" => Some(UpdateStrategy::Date),
            "cursor" => Some(UpdateStrategy::Cursor),
            "high_water_mark" | "high-water" | "high_water" => Some(UpdateStrategy::HighWaterMark),
            "snapshot" => Some(UpdateStrategy::Snapshot),
            _ => None,
        }
    }
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
        if self.status != UpdateRunStatus::Completed && self.supersedes_run_id.is_some() {
            return Err(ContractError::Invariant(
                "only a completed living update may supersede an earlier run".to_owned(),
            ));
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
        let mut prior_providers = BTreeSet::new();
        for cursor in &self.cursors_before {
            cursor.validate()?;
            if !prior_providers.insert(cursor.provider_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "living update contains duplicate before-cursor for `{}`",
                    cursor.provider_id
                )));
            }
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
        if !self.changes.is_empty() && !self.requires_human_release {
            return Err(ContractError::Invariant(
                "living-update record changes require human release before downstream screening"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completed_run() -> LivingUpdateRun {
        LivingUpdateRun {
            schema_version: LIVING_UPDATE_SCHEMA_VERSION.to_owned(),
            review_id: "review-1".to_owned(),
            run_id: "run-1".to_owned(),
            parent_run_id: None,
            status: UpdateRunStatus::Completed,
            started_at: "2026-08-01T00:00:00Z".to_owned(),
            completed_at: Some("2026-08-01T01:00:00Z".to_owned()),
            protocol_version: "protocol-v1".to_owned(),
            cursors_before: Vec::new(),
            cursors_after: Vec::new(),
            changes: Vec::new(),
            requires_human_release: true,
            supersedes_run_id: None,
        }
    }

    #[test]
    fn changed_records_require_human_release() {
        let mut update = completed_run();
        update.requires_human_release = false;
        update.changes.push(RecordChange {
            record_id: "record-1".to_owned(),
            kind: RecordChangeKind::Added,
            before_digest: None,
            after_digest: Some("digest-1".to_owned()),
            note: "newly observed".to_owned(),
        });

        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(_))
        ));
    }

    #[test]
    fn duplicate_before_cursors_are_rejected() {
        let cursor = UpdateCursor {
            provider_id: "pubmed".to_owned(),
            cursor_kind: "date".to_owned(),
            value: "2026-08-01".to_owned(),
            retrieved_through: Some("2026-08-01".to_owned()),
        };
        let mut update = completed_run();
        update.cursors_before = vec![cursor.clone(), cursor];

        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(_))
        ));
    }

    #[test]
    fn validate_rejects_invalid_schema_version() {
        let mut update = completed_run();
        update.schema_version = "invalid-schema".to_owned();
        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("schema_version")
        ));
    }

    #[test]
    fn validate_requires_run_id_and_review_id() {
        let mut update = completed_run();
        update.run_id = "".to_owned();
        assert!(matches!(
            update.validate(),
            Err(ContractError::EmptyField(_))
        ));

        let mut update2 = completed_run();
        update2.review_id = "   ".to_owned();
        assert!(matches!(
            update2.validate(),
            Err(ContractError::EmptyField(_))
        ));
    }

    #[test]
    fn validate_rejects_self_as_parent_and_superseded() {
        let mut update = completed_run();
        update.parent_run_id = Some("run-1".to_owned());
        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("cannot name itself as parent")
        ));

        let mut update2 = completed_run();
        update2.supersedes_run_id = Some("run-1".to_owned());
        assert!(matches!(
            update2.validate(),
            Err(ContractError::Invariant(message)) if message.contains("cannot supersede itself")
        ));
    }

    #[test]
    fn validate_rejects_supersession_on_unfinished_run() {
        let mut update = completed_run();
        update.status = UpdateRunStatus::Running;
        update.completed_at = None;
        update.supersedes_run_id = Some("run-0".to_owned());
        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("only a completed living update may supersede")
        ));
    }

    #[test]
    fn validate_completed_at_consistency() {
        let mut update = completed_run();
        update.completed_at = None;
        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("requires completed_at")
        ));

        let mut update2 = completed_run();
        update2.status = UpdateRunStatus::Planned;
        update2.completed_at = Some("2026-08-01T01:00:00Z".to_owned());
        assert!(matches!(
            update2.validate(),
            Err(ContractError::Invariant(message)) if message.contains("must not contain completed_at")
        ));
    }

    #[test]
    fn validate_rejects_duplicate_after_cursors() {
        let cursor = UpdateCursor {
            provider_id: "pubmed".to_owned(),
            cursor_kind: "date".to_owned(),
            value: "2026-08-01".to_owned(),
            retrieved_through: Some("2026-08-01".to_owned()),
        };
        let mut update = completed_run();
        update.cursors_after = vec![cursor.clone(), cursor];

        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("duplicate after-cursor")
        ));
    }

    #[test]
    fn validate_rejects_duplicate_record_changes() {
        let change = RecordChange {
            record_id: "record-1".to_owned(),
            kind: RecordChangeKind::Added,
            before_digest: None,
            after_digest: Some("digest-1".to_owned()),
            note: "newly observed".to_owned(),
        };
        let mut update = completed_run();
        update.changes = vec![change.clone(), change];

        assert!(matches!(
            update.validate(),
            Err(ContractError::Invariant(message)) if message.contains("duplicate record change")
        ));
    }

    #[test]
    fn validate_change_without_digests_fails() {
        let change = RecordChange {
            record_id: "record-1".to_owned(),
            kind: RecordChangeKind::Added,
            before_digest: None,
            after_digest: None,
            note: "no digests provided".to_owned(),
        };
        assert!(matches!(
            change.validate(),
            Err(ContractError::Invariant(message)) if message.contains("at least one content digest")
        ));
    }

    #[test]
    fn validate_cursor_fields_require_text() {
        let cursor = UpdateCursor {
            provider_id: "".to_owned(),
            cursor_kind: "date".to_owned(),
            value: "2026-08-01".to_owned(),
            retrieved_through: None,
        };
        assert!(matches!(
            cursor.validate(),
            Err(ContractError::EmptyField(_))
        ));
    }

    #[test]
    fn update_strategy_as_str_and_cursor_strategy_mapping() {
        assert_eq!(UpdateStrategy::Date.as_str(), "date");
        assert_eq!(UpdateStrategy::Cursor.as_str(), "cursor");
        assert_eq!(UpdateStrategy::HighWaterMark.as_str(), "high_water_mark");
        assert_eq!(UpdateStrategy::Snapshot.as_str(), "snapshot");

        let cursor_date = UpdateCursor {
            provider_id: "pubmed".to_owned(),
            cursor_kind: "date".to_owned(),
            value: "2026-08-01".to_owned(),
            retrieved_through: None,
        };
        assert_eq!(cursor_date.strategy(), Some(UpdateStrategy::Date));

        let cursor_tok = UpdateCursor {
            provider_id: "openalex".to_owned(),
            cursor_kind: "cursor".to_owned(),
            value: "opaque-cursor-token".to_owned(),
            retrieved_through: None,
        };
        assert_eq!(cursor_tok.strategy(), Some(UpdateStrategy::Cursor));

        let cursor_hw = UpdateCursor {
            provider_id: "crossref".to_owned(),
            cursor_kind: "high-water".to_owned(),
            value: "2026-08-01T12:00:00Z".to_owned(),
            retrieved_through: None,
        };
        assert_eq!(cursor_hw.strategy(), Some(UpdateStrategy::HighWaterMark));

        let cursor_snap = UpdateCursor {
            provider_id: "cochrane".to_owned(),
            cursor_kind: "snapshot".to_owned(),
            value: "snapshot-hash".to_owned(),
            retrieved_through: None,
        };
        assert_eq!(cursor_snap.strategy(), Some(UpdateStrategy::Snapshot));

        let cursor_unknown = UpdateCursor {
            provider_id: "other".to_owned(),
            cursor_kind: "unsupported-strategy".to_owned(),
            value: "val".to_owned(),
            retrieved_through: None,
        };
        assert_eq!(cursor_unknown.strategy(), None);
    }
}
