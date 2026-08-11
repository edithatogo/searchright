//! Delivery-control contracts for repository settings, release trains and rehearsals.

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, GITHUB_REPOSITORY_SETTINGS_SCHEMA_VERSION,
    INTEGRATION_RELEASE_TRAIN_SCHEMA_VERSION, RELEASE_REHEARSAL_SCHEMA_VERSION, Validate,
    require_schema_version, require_text,
};

/// GitHub repository visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryVisibility {
    /// Public repository.
    Public,
    /// Private repository.
    Private,
}

/// Feature switches controlled by the repository-settings manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryFeatures {
    /// GitHub Issues are enabled.
    pub issues: bool,
    /// GitHub Projects are enabled.
    pub projects: bool,
    /// GitHub Discussions are enabled.
    pub discussions: bool,
    /// GitHub Wiki is enabled.
    pub wiki: bool,
}

/// Merge policy controlled by the repository-settings manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryMergePolicy {
    /// Squash merging is enabled.
    pub squash: bool,
    /// Rebase merging is enabled.
    pub rebase: bool,
    /// Merge commits are enabled.
    pub merge_commit: bool,
    /// Head branches are deleted after merge.
    pub delete_head_branch: bool,
    /// Auto-merge is enabled.
    pub allow_auto_merge: bool,
    /// Pull requests may update their branches.
    pub allow_update_branch: bool,
}

/// Security controls requested for the remote repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositorySecurityControls {
    /// Vulnerability alerts are enabled where supported.
    pub vulnerability_alerts: bool,
    /// Automated security fixes are enabled where supported.
    pub automated_security_fixes: bool,
    /// Secret scanning is enabled where supported.
    pub secret_scanning: bool,
    /// Secret-scanning push protection is enabled where supported.
    pub push_protection: bool,
    /// Private vulnerability reporting is enabled where supported.
    pub private_vulnerability_reporting: bool,
}

/// GitHub ruleset enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RulesetEnforcement {
    /// Actively enforce the ruleset.
    Active,
    /// Evaluate the ruleset without blocking.
    Evaluate,
    /// Keep the ruleset disabled.
    Disabled,
}

/// Non-destructive branch-ruleset declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryRuleset {
    /// Ruleset name.
    pub name: String,
    /// Ruleset target. Version 1 supports only `branch`.
    pub target: String,
    /// Ruleset enforcement mode.
    pub enforcement: RulesetEnforcement,
    /// Included reference patterns.
    pub include: Vec<String>,
    /// Required status-check contexts.
    pub required_status_checks: Vec<String>,
    /// Linear history is required.
    pub required_linear_history: bool,
    /// Signed commits are required.
    pub required_signed_commits: bool,
    /// Branch deletion is allowed. Version 1 requires false.
    pub deletion: bool,
    /// Non-fast-forward updates are allowed. Version 1 requires false.
    pub non_fast_forward: bool,
}

/// Declarative GitHub repository control-plane manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubRepositorySettings {
    /// Contract identifier.
    pub schema_version: String,
    /// Repository in `owner/name` form.
    pub repository: String,
    /// Repository visibility.
    pub visibility: RepositoryVisibility,
    /// Repository description.
    pub description: String,
    /// Repository homepage.
    pub homepage: String,
    /// Feature switches.
    pub features: RepositoryFeatures,
    /// Merge policy.
    pub merge_policy: RepositoryMergePolicy,
    /// Repository topics.
    pub topics: Vec<String>,
    /// Protected deployment environments.
    pub environments: Vec<String>,
    /// Requested security controls.
    pub security: RepositorySecurityControls,
    /// Main-branch ruleset declaration.
    pub ruleset: RepositoryRuleset,
    /// Source manifests never authorise remote mutation.
    pub apply_permitted: bool,
    /// Explicit public-claim boundary.
    pub claim_boundary: String,
}

/// One component participating in the cross-repository release train.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseTrainComponent {
    /// Repository in `owner/name` form.
    pub repository: String,
    /// Architectural role.
    pub role: String,
    /// Optional integration-passport path.
    pub passport: Option<String>,
    /// Optional consumer-contract identifier.
    pub consumer_contract: Option<String>,
    /// Ordered promotion position.
    pub promotion_order: u32,
}

/// One evidence gate in the release train.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseTrainStage {
    /// Stable stage identifier.
    pub id: String,
    /// Evidence level required for the stage.
    pub required_evidence: String,
    /// Automatic promotion is prohibited in version 1.
    pub automatic: bool,
}

/// Cross-repository release-train contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationReleaseTrain {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable release-train identifier.
    pub release_train_id: String,
    /// Reproducible generation label.
    pub generated_at: String,
    /// Automatic promotion is prohibited in version 1.
    pub automatic_promotion: bool,
    /// Ordered participating repositories.
    pub components: Vec<ReleaseTrainComponent>,
    /// Evidence stages.
    pub stages: Vec<ReleaseTrainStage>,
    /// Reversible rollback instructions.
    pub rollback: Vec<String>,
    /// Explicit public-claim boundary.
    pub claim_boundary: String,
}

/// Release-rehearsal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseRehearsalStatus {
    /// Rehearsal is specified but has not run.
    PreparedNotExecuted,
    /// Rehearsal is in progress.
    InProgress,
    /// Rehearsal executed and failed.
    Failed,
    /// Rehearsal executed and passed.
    Passed,
}

/// Release-candidate rehearsal and pilot contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseRehearsal {
    /// Contract identifier.
    pub schema_version: String,
    /// Target release identifier.
    pub target: String,
    /// Rehearsal state.
    pub status: ReleaseRehearsalStatus,
    /// Source epoch or reproducible date label.
    pub source_epoch: String,
    /// Gates required before promotion.
    pub required_gates: Vec<String>,
    /// Pilot deployment profiles.
    pub pilot_profiles: Vec<String>,
    /// Automatic release is prohibited.
    pub automatic_release: bool,
    /// Automatic registry submission is prohibited.
    pub automatic_registry_submission: bool,
    /// Rollback rehearsal is mandatory.
    pub rollback_required: bool,
    /// Explicit public-claim boundary.
    pub claim_boundary: String,
}

fn owner_name(value: &str) -> bool {
    let mut parts = value.split('/');
    matches!((parts.next(), parts.next(), parts.next()), (Some(owner), Some(name), None) if !owner.is_empty() && !name.is_empty())
}

impl Validate for GitHubRepositorySettings {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            GITHUB_REPOSITORY_SETTINGS_SCHEMA_VERSION,
            "github_repository_settings.schema_version",
        )?;
        require_text(&self.repository, "github_repository_settings.repository")?;
        require_text(&self.description, "github_repository_settings.description")?;
        require_text(&self.homepage, "github_repository_settings.homepage")?;
        require_text(
            &self.claim_boundary,
            "github_repository_settings.claim_boundary",
        )?;
        if !owner_name(&self.repository) {
            return Err(ContractError::Invariant(
                "GitHub repository must use non-empty owner/name form".to_owned(),
            ));
        }
        if self.apply_permitted {
            return Err(ContractError::Invariant(
                "repository settings manifests cannot authorise remote writes".to_owned(),
            ));
        }
        if !self.features.issues || !self.features.projects || self.features.wiki {
            return Err(ContractError::Invariant(
                "Searchright requires Issues and Projects and keeps the Wiki disabled".to_owned(),
            ));
        }
        if self.merge_policy.merge_commit || !self.merge_policy.squash {
            return Err(ContractError::Invariant(
                "Searchright requires squash merging and disables merge commits".to_owned(),
            ));
        }
        if self.topics.is_empty() || self.environments.is_empty() {
            return Err(ContractError::EmptyCollection(
                "github_repository_settings.topics_or_environments",
            ));
        }
        if self.ruleset.target != "branch"
            || self.ruleset.deletion
            || self.ruleset.non_fast_forward
            || self.ruleset.required_status_checks.is_empty()
        {
            return Err(ContractError::Invariant(
                "main ruleset must target branches, deny deletion/non-fast-forward, and require checks"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for IntegrationReleaseTrain {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            INTEGRATION_RELEASE_TRAIN_SCHEMA_VERSION,
            "integration_release_train.schema_version",
        )?;
        require_text(
            &self.release_train_id,
            "integration_release_train.release_train_id",
        )?;
        require_text(&self.generated_at, "integration_release_train.generated_at")?;
        require_text(
            &self.claim_boundary,
            "integration_release_train.claim_boundary",
        )?;
        if self.automatic_promotion {
            return Err(ContractError::Invariant(
                "release-train promotion must remain explicit".to_owned(),
            ));
        }
        if self.components.len() < 2 || self.stages.is_empty() || self.rollback.is_empty() {
            return Err(ContractError::Invariant(
                "release train requires multiple components, stages and rollback".to_owned(),
            ));
        }
        let mut repositories = BTreeSet::new();
        let mut orders = BTreeSet::new();
        for component in &self.components {
            require_text(&component.repository, "release_train_component.repository")?;
            require_text(&component.role, "release_train_component.role")?;
            if !owner_name(&component.repository)
                || component.promotion_order == 0
                || !repositories.insert(component.repository.as_str())
                || !orders.insert(component.promotion_order)
            {
                return Err(ContractError::Invariant(
                    "release-train components require unique repositories and positive unique order"
                        .to_owned(),
                ));
            }
        }
        let mut stage_ids = BTreeSet::new();
        for stage in &self.stages {
            require_text(&stage.id, "release_train_stage.id")?;
            require_text(
                &stage.required_evidence,
                "release_train_stage.required_evidence",
            )?;
            if stage.automatic || !stage_ids.insert(stage.id.as_str()) {
                return Err(ContractError::Invariant(
                    "release-train stages must be unique and explicitly promoted".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

impl Validate for ReleaseRehearsal {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            RELEASE_REHEARSAL_SCHEMA_VERSION,
            "release_rehearsal.schema_version",
        )?;
        require_text(&self.target, "release_rehearsal.target")?;
        require_text(&self.source_epoch, "release_rehearsal.source_epoch")?;
        require_text(&self.claim_boundary, "release_rehearsal.claim_boundary")?;
        if self.required_gates.is_empty() || self.pilot_profiles.is_empty() {
            return Err(ContractError::EmptyCollection(
                "release_rehearsal.required_gates_or_pilot_profiles",
            ));
        }
        if self.automatic_release || self.automatic_registry_submission || !self.rollback_required {
            return Err(ContractError::Invariant(
                "release and registry promotion must be explicit and rollback must be required"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}
