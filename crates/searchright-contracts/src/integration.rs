//! Cross-repository integration and GitHub issue-hierarchy contracts.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    CONSUMER_CONTRACT_SUITE_SCHEMA_VERSION, ContractError, GITHUB_ISSUE_HIERARCHY_SCHEMA_VERSION,
    GITHUB_PROJECT_SCHEMA_VERSION, INTEGRATION_PASSPORT_SCHEMA_VERSION, Validate,
    require_schema_version, require_text,
};

/// Supported cross-repository integration mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationMode {
    /// Compile-time Rust dependency behind a feature boundary.
    RustDependency,
    /// Stable CLI JSON/JSONL exchange.
    CliJson,
    /// Model Context Protocol tool/resource exchange.
    Mcp,
    /// Capability-bounded WebAssembly component exchange.
    WasiComponent,
    /// Read-only benchmark or fixture dataset.
    DatasetFixture,
    /// Versioned standards or documentation pack.
    DocumentationPack,
    /// Centrally governed policy inherited by verification tooling.
    PolicyInheritance,
    /// Generated adapter from a neutral contract catalogue.
    GeneratedAdapter,
}

/// Direction of dependency across repository boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DependencyDirection {
    /// Searchright consumes an upstream contract or artefact.
    SearchrightConsumesUpstream,
    /// A downstream repository consumes Searchright.
    DownstreamConsumesSearchright,
    /// Both sides integrate only through a neutral, versioned contract.
    BidirectionalViaNeutralContract,
    /// No runtime dependency; only governance or documentation is shared.
    NoRuntimeDependency,
}

/// Relationship between the pinned repository and any canonical upstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LocalForkRole {
    /// Independently maintained original repository.
    Original,
    /// Passive or periodically refreshed mirror of another repository.
    Mirror,
    /// Fork that carries an explicit, reviewable local delta.
    PatchCarrier,
    /// Independently versioned product derived from an upstream codebase.
    DerivedProduct,
}

/// Licence and redistribution review state for an integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LicenceReviewStatus {
    /// Code/content use and redistribution have an identified licence basis.
    Cleared,
    /// The integration is used only as a policy or documentation reference.
    ReferenceOnly,
    /// Reuse remains blocked until a licence review is recorded.
    ReviewRequired,
    /// The local fork inherits a verified upstream licence without broadening it.
    UpstreamLicenceInherited,
}

/// Canonical upstream identity for a forked or derived integration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CanonicalUpstreamReference {
    /// Canonical repository in `owner/name` form.
    pub repository: String,
    /// Exact upstream revision when it has been independently pinned.
    pub revision: Option<String>,
    /// Observed canonical default branch when known.
    pub default_branch: Option<String>,
    /// Explicit boundary on what was verified about the upstream.
    pub verification_status: String,
}

/// One named contract crossing a repository boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationContractReference {
    /// Stable contract name.
    pub name: String,
    /// Contract or schema version.
    pub version: String,
    /// Repository-relative schema, WIT, OpenAPI or fixture path.
    pub path: String,
    /// Whether compatibility with this contract blocks integration promotion.
    pub required: bool,
}

/// One deterministic integration verification gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationVerificationGate {
    /// Stable gate identifier.
    pub id: String,
    /// Network-free command or external evidence instruction.
    pub command: String,
    /// Highest evidence level this gate can establish.
    pub evidence_level: String,
    /// Whether the gate must pass before cutover.
    pub required: bool,
}

/// Pinned and reviewable contract for one repository integration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationPassport {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable integration identifier.
    pub integration_id: String,
    /// GitHub repository in `owner/name` form.
    pub repository: String,
    /// Exact 40-character Git revision observed during preparation.
    pub revision: String,
    /// Observed default branch.
    pub default_branch: String,
    /// Canonical upstream for a fork or derived product; absent for originals.
    pub canonical_upstream: Option<CanonicalUpstreamReference>,
    /// Relationship between the pinned repository and its canonical upstream.
    pub local_fork_role: LocalForkRole,
    /// SPDX expression or `NOASSERTION` for executable source.
    pub code_license: String,
    /// SPDX expression or `NOASSERTION` for documentation, standards or data content.
    pub content_license: String,
    /// Optional model licence where model weights or services enter the integration.
    pub model_license: Option<String>,
    /// Human-readable redistribution and derivative-use boundary.
    pub redistribution: String,
    /// Current licence-review state.
    pub licence_review_status: LicenceReviewStatus,
    /// Policy for detecting and reviewing upstream/local drift.
    pub drift_policy: String,
    /// Integration mechanism.
    pub mode: IntegrationMode,
    /// Dependency direction.
    pub dependency_direction: DependencyDirection,
    /// Optional Cargo feature or runtime capability switch.
    pub feature_flag: Option<String>,
    /// Contracts consumed from the integration.
    #[serde(default)]
    pub inputs: Vec<IntegrationContractReference>,
    /// Contracts emitted to the integration.
    #[serde(default)]
    pub outputs: Vec<IntegrationContractReference>,
    /// Default execution cannot use the network.
    pub default_network: bool,
    /// Default execution cannot write to external systems.
    pub default_external_writes: bool,
    /// Default execution cannot emit telemetry.
    pub default_telemetry: bool,
    /// Whether an automated job may change the pinned revision.
    pub automatic_revision_updates: bool,
    /// Capabilities permitted to the integration adapter.
    #[serde(default)]
    pub allowed_capabilities: Vec<String>,
    /// Verification gates required by this passport.
    pub verification: Vec<IntegrationVerificationGate>,
    /// Reversible steps for disabling or rolling back the integration.
    pub rollback: Vec<String>,
    /// Explicit public-claim boundary.
    pub claim_boundary: String,
}

/// Evidence state for one consumer-driven contract interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsumerContractStatus {
    /// The interaction is specified but has not executed in both repositories.
    PreparedNotExecuted,
    /// Local deterministic fixtures have passed in the consumer repository.
    FixtureVerified,
    /// Producer and consumer repositories have both supplied compatible receipts.
    DownstreamVerified,
    /// The interaction is intentionally disabled because a gate failed or drifted.
    Suspended,
}

/// One producer-consumer interaction governed by versioned contracts and fixtures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConsumerContractInteraction {
    /// Stable interaction identifier.
    pub id: String,
    /// Integration passport identifier that authorises this interaction.
    pub integration_id: String,
    /// Producer repository in `owner/name` form.
    pub producer: String,
    /// Consumer repository in `owner/name` form.
    pub consumer: String,
    /// Neutral contract or schema version governing the exchange.
    pub contract_version: String,
    /// Producer-side contracts; external paths use the `external://` scheme.
    pub producer_contracts: Vec<String>,
    /// Consumer-side contracts; local paths are repository-relative.
    pub consumer_contracts: Vec<String>,
    /// Deterministic local fixtures used to test the interaction.
    pub fixture_paths: Vec<String>,
    /// Gates expected in the producer repository.
    pub producer_gates: Vec<String>,
    /// Gates expected in the consumer repository.
    pub consumer_gates: Vec<String>,
    /// Behaviour required when compatibility is absent or uncertain.
    pub failure_semantics: String,
    /// Whether a passing automated job may promote the integration without review.
    pub automatic_promotion: bool,
    /// Current evidence state.
    pub status: ConsumerContractStatus,
}

/// Consumer-driven contract suite for all active repository integrations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConsumerContractSuite {
    /// Contract identifier.
    pub schema_version: String,
    /// Reproducible generation date or source-epoch label.
    pub generated_at: String,
    /// Declared producer-consumer interactions.
    pub interactions: Vec<ConsumerContractInteraction>,
    /// Explicit public-claim boundary.
    pub claim_boundary: String,
}

/// Kind of issue represented in the generated GitHub hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GitHubIssueKind {
    /// One repository roadmap epic.
    Epic,
    /// One issue corresponding to a Conductor track.
    Track,
    /// One subissue corresponding to a plan phase.
    Phase,
    /// One subissue corresponding to a top-level Conductor plan task.
    Task,
}

/// Scalar value projected into one GitHub Project custom field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum GitHubProjectFieldValue {
    /// Text or single-select option name.
    Text(String),
    /// Integral number for deterministic phase/task identifiers.
    Number(u32),
}

/// One generated GitHub issue or subissue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubIssueNode {
    /// Stable idempotency key embedded in the issue body.
    pub key: String,
    /// Issue title.
    pub title: String,
    /// Hierarchy kind.
    pub kind: GitHubIssueKind,
    /// Parent idempotency key for a track, phase or task.
    pub parent_key: Option<String>,
    /// Repository-relative rendered Markdown body path.
    pub body_path: String,
    /// Labels requested by the sync plan.
    pub labels: Vec<String>,
    /// Local planning state; not a claim about remote GitHub state.
    pub status: String,
    /// Canonical desired issue state; only task state may be remotely mirrored.
    pub desired_state: String,
    /// Track identifier when applicable.
    pub track_id: Option<String>,
    /// Phase number when applicable.
    pub phase_number: Option<u8>,
    /// Top-level task number when applicable.
    pub task_number: Option<u32>,
    /// Manifest-owned GitHub Project field projection.
    pub project_fields: BTreeMap<String, GitHubProjectFieldValue>,
}

/// Deterministic roadmap-to-GitHub issue hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubIssueHierarchy {
    /// Contract identifier.
    pub schema_version: String,
    /// Intended repository in `owner/name` form.
    pub repository: String,
    /// Stable epic key.
    pub epic_key: String,
    /// Generated nodes.
    pub nodes: Vec<GitHubIssueNode>,
    /// Reproducible generation date or source-epoch label.
    pub generated_at: String,
    /// Whether this artefact itself authorises remote mutation.
    pub apply_permitted: bool,
    /// Remote issue-state policy.
    pub state_sync_policy: String,
    /// Canonical GitHub Project manifest path.
    pub project_manifest: String,
}

/// Owner kind for a GitHub Project v2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GitHubProjectOwnerType {
    /// Project owned by a user account.
    User,
    /// Project owned by an organisation.
    Organization,
}

/// Supported custom-field data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GitHubProjectFieldDataType {
    /// Single-select field.
    #[serde(rename = "SINGLE_SELECT")]
    SingleSelect,
    /// Free text field.
    #[serde(rename = "TEXT")]
    Text,
    /// Numeric field.
    #[serde(rename = "NUMBER")]
    Number,
    /// Date field.
    #[serde(rename = "DATE")]
    Date,
}

/// One manifest-owned GitHub Project custom field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubProjectField {
    /// Exact custom-field name.
    pub name: String,
    /// GitHub Project data type.
    pub data_type: GitHubProjectFieldDataType,
    /// Allowed option names for single-select fields.
    #[serde(default)]
    pub options: Vec<String>,
}

/// Supported GitHub Project view layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GitHubProjectViewLayout {
    /// Board view.
    #[serde(rename = "BOARD_LAYOUT")]
    Board,
    /// Roadmap view.
    #[serde(rename = "ROADMAP_LAYOUT")]
    Roadmap,
    /// Table view.
    #[serde(rename = "TABLE_LAYOUT")]
    Table,
}

/// One requested GitHub Project view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubProjectView {
    /// View name.
    pub name: String,
    /// View layout.
    pub layout: GitHubProjectViewLayout,
    /// GitHub Project filter string.
    pub filter: String,
}

/// Non-destructive Project synchronisation policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubProjectSyncPolicy {
    /// Canonical issue hierarchy path.
    pub hierarchy_path: String,
    /// Canonical label manifest path.
    pub labels_path: String,
    /// Stable identity custom field.
    pub identity_field: String,
    /// Issue-state policy.
    pub state_policy: String,
    /// Deletion policy.
    pub delete_policy: String,
    /// Archival policy.
    pub archive_policy: String,
    /// Custom-field ownership policy.
    pub field_policy: String,
    /// Evidence-promotion policy.
    pub promotion_policy: String,
}

/// Declarative GitHub Project v2 projection manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitHubProjectManifest {
    /// Contract identifier.
    pub schema_version: String,
    /// Project owner login.
    pub owner: String,
    /// Owner kind.
    pub owner_type: GitHubProjectOwnerType,
    /// Linked repository in owner/name form.
    pub repository: String,
    /// Project title.
    pub title: String,
    /// Short Project description.
    pub short_description: String,
    /// Repository-relative Project README path.
    pub readme_path: String,
    /// Public/private visibility label.
    pub visibility: String,
    /// Remote number remains absent in the source manifest.
    pub project_number: Option<u64>,
    /// Whether the repository should be linked.
    pub link_repository: bool,
    /// Source manifests never authorise remote mutation.
    pub apply_permitted: bool,
    /// Manifest-owned custom fields.
    pub fields: Vec<GitHubProjectField>,
    /// Requested Project views.
    pub views: Vec<GitHubProjectView>,
    /// Synchronisation policy.
    pub sync: GitHubProjectSyncPolicy,
}

fn is_hex_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl Validate for IntegrationPassport {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            INTEGRATION_PASSPORT_SCHEMA_VERSION,
            "integration_passport.schema_version",
        )?;
        require_text(&self.integration_id, "integration_passport.integration_id")?;
        require_text(&self.repository, "integration_passport.repository")?;
        require_text(&self.default_branch, "integration_passport.default_branch")?;
        require_text(&self.code_license, "integration_passport.code_license")?;
        require_text(
            &self.content_license,
            "integration_passport.content_license",
        )?;
        if let Some(model_license) = &self.model_license {
            require_text(model_license, "integration_passport.model_license")?;
        }
        require_text(&self.redistribution, "integration_passport.redistribution")?;
        require_text(&self.drift_policy, "integration_passport.drift_policy")?;
        require_text(&self.claim_boundary, "integration_passport.claim_boundary")?;
        let mut repository_parts = self.repository.split('/');
        let owner = repository_parts.next().unwrap_or_default();
        let name = repository_parts.next().unwrap_or_default();
        if owner.is_empty() || name.is_empty() || repository_parts.next().is_some() {
            return Err(ContractError::Invariant(
                "integration repository must use non-empty owner/name form".to_owned(),
            ));
        }
        if !is_hex_revision(&self.revision) {
            return Err(ContractError::Invariant(
                "integration revision must be an exact 40-character hexadecimal Git revision"
                    .to_owned(),
            ));
        }
        match (&self.local_fork_role, &self.canonical_upstream) {
            (LocalForkRole::Original, None) => {}
            (LocalForkRole::Original, Some(_)) => {
                return Err(ContractError::Invariant(
                    "original integrations must not declare a canonical upstream".to_owned(),
                ));
            }
            (_, None) => {
                return Err(ContractError::Invariant(
                    "mirrors, patch carriers and derived products must declare a canonical upstream".to_owned(),
                ));
            }
            (_, Some(upstream)) => {
                require_text(
                    &upstream.repository,
                    "integration_passport.canonical_upstream.repository",
                )?;
                let mut parts = upstream.repository.split('/');
                if parts.next().unwrap_or_default().is_empty()
                    || parts.next().unwrap_or_default().is_empty()
                    || parts.next().is_some()
                {
                    return Err(ContractError::Invariant(
                        "canonical upstream repository must use owner/name form".to_owned(),
                    ));
                }
                if let Some(revision) = &upstream.revision
                    && !is_hex_revision(revision)
                {
                    return Err(ContractError::Invariant(
                        "canonical upstream revision must be an exact 40-character hexadecimal Git revision".to_owned(),
                    ));
                }
                if let Some(branch) = &upstream.default_branch {
                    require_text(
                        branch,
                        "integration_passport.canonical_upstream.default_branch",
                    )?;
                }
                require_text(
                    &upstream.verification_status,
                    "integration_passport.canonical_upstream.verification_status",
                )?;
            }
        }
        if self.default_network || self.default_external_writes || self.default_telemetry {
            return Err(ContractError::Invariant(
                "integration defaults must deny network, external writes and telemetry".to_owned(),
            ));
        }
        if self.automatic_revision_updates {
            return Err(ContractError::Invariant(
                "integration passports may detect drift but must not update revisions automatically"
                    .to_owned(),
            ));
        }
        if self.inputs.is_empty() && self.outputs.is_empty() {
            return Err(ContractError::EmptyCollection(
                "integration_passport.inputs_or_outputs",
            ));
        }
        let mut contracts = BTreeSet::new();
        for contract in self.inputs.iter().chain(self.outputs.iter()) {
            require_text(&contract.name, "integration_passport.contract.name")?;
            require_text(&contract.version, "integration_passport.contract.version")?;
            require_text(&contract.path, "integration_passport.contract.path")?;
            let key = format!("{}:{}:{}", contract.name, contract.version, contract.path);
            if !contracts.insert(key) {
                return Err(ContractError::Invariant(
                    "integration contract references must be unique".to_owned(),
                ));
            }
        }
        if self.verification.is_empty() {
            return Err(ContractError::EmptyCollection(
                "integration_passport.verification",
            ));
        }
        let mut gates = BTreeSet::new();
        for gate in &self.verification {
            require_text(&gate.id, "integration_passport.verification.id")?;
            require_text(&gate.command, "integration_passport.verification.command")?;
            require_text(
                &gate.evidence_level,
                "integration_passport.verification.evidence_level",
            )?;
            if !gates.insert(gate.id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "duplicate integration verification gate `{}`",
                    gate.id
                )));
            }
        }
        if self.rollback.is_empty() || self.rollback.iter().any(|step| step.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "integration rollback steps must be non-empty".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for ConsumerContractSuite {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            CONSUMER_CONTRACT_SUITE_SCHEMA_VERSION,
            "consumer_contract_suite.schema_version",
        )?;
        require_text(&self.generated_at, "consumer_contract_suite.generated_at")?;
        require_text(
            &self.claim_boundary,
            "consumer_contract_suite.claim_boundary",
        )?;
        if self.interactions.is_empty() {
            return Err(ContractError::EmptyCollection(
                "consumer_contract_suite.interactions",
            ));
        }
        let mut ids = BTreeSet::new();
        for interaction in &self.interactions {
            require_text(&interaction.id, "consumer_contract_interaction.id")?;
            require_text(
                &interaction.integration_id,
                "consumer_contract_interaction.integration_id",
            )?;
            require_text(
                &interaction.producer,
                "consumer_contract_interaction.producer",
            )?;
            require_text(
                &interaction.consumer,
                "consumer_contract_interaction.consumer",
            )?;
            require_text(
                &interaction.contract_version,
                "consumer_contract_interaction.contract_version",
            )?;
            require_text(
                &interaction.failure_semantics,
                "consumer_contract_interaction.failure_semantics",
            )?;
            if interaction.producer == interaction.consumer {
                return Err(ContractError::Invariant(format!(
                    "consumer interaction `{}` must cross a repository boundary",
                    interaction.id
                )));
            }
            if !ids.insert(interaction.id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "duplicate consumer interaction `{}`",
                    interaction.id
                )));
            }
            if interaction.automatic_promotion {
                return Err(ContractError::Invariant(format!(
                    "consumer interaction `{}` must require explicit promotion",
                    interaction.id
                )));
            }
            for (field, values) in [
                ("producer_contracts", &interaction.producer_contracts),
                ("consumer_contracts", &interaction.consumer_contracts),
                ("fixture_paths", &interaction.fixture_paths),
                ("producer_gates", &interaction.producer_gates),
                ("consumer_gates", &interaction.consumer_gates),
            ] {
                if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
                    return Err(ContractError::Invariant(format!(
                        "consumer interaction `{}` requires non-empty {field}",
                        interaction.id
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Validate for GitHubIssueHierarchy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            GITHUB_ISSUE_HIERARCHY_SCHEMA_VERSION,
            "github_issue_hierarchy.schema_version",
        )?;
        require_text(&self.repository, "github_issue_hierarchy.repository")?;
        require_text(&self.epic_key, "github_issue_hierarchy.epic_key")?;
        require_text(&self.generated_at, "github_issue_hierarchy.generated_at")?;
        if self.apply_permitted {
            return Err(ContractError::Invariant(
                "rendered issue hierarchy must not authorise remote writes".to_owned(),
            ));
        }
        if self.state_sync_policy != "task_issues_only" {
            return Err(ContractError::Invariant(
                "only task issue state may be synchronised".to_owned(),
            ));
        }
        require_text(
            &self.project_manifest,
            "github_issue_hierarchy.project_manifest",
        )?;
        if self.nodes.is_empty() {
            return Err(ContractError::EmptyCollection(
                "github_issue_hierarchy.nodes",
            ));
        }
        let mut by_key = BTreeMap::new();
        for node in &self.nodes {
            require_text(&node.key, "github_issue_hierarchy.node.key")?;
            require_text(&node.title, "github_issue_hierarchy.node.title")?;
            require_text(&node.body_path, "github_issue_hierarchy.node.body_path")?;
            require_text(&node.status, "github_issue_hierarchy.node.status")?;
            require_text(
                &node.desired_state,
                "github_issue_hierarchy.node.desired_state",
            )?;
            if node.status != "prepared_not_synced" {
                return Err(ContractError::Invariant(
                    "local issue nodes must remain prepared_not_synced".to_owned(),
                ));
            }
            if !matches!(node.desired_state.as_str(), "open" | "closed") {
                return Err(ContractError::Invariant(format!(
                    "issue `{}` has invalid desired state",
                    node.key
                )));
            }
            if by_key.insert(node.key.as_str(), node).is_some() {
                return Err(ContractError::Invariant(format!(
                    "duplicate issue key `{}`",
                    node.key
                )));
            }
        }
        let mut epics = self
            .nodes
            .iter()
            .filter(|node| node.kind == GitHubIssueKind::Epic);
        let Some(epic) = epics.next() else {
            return Err(ContractError::Invariant(
                "issue hierarchy requires exactly one root epic matching epic_key".to_owned(),
            ));
        };
        if epics.next().is_some() || epic.key != self.epic_key || epic.parent_key.is_some() {
            return Err(ContractError::Invariant(
                "issue hierarchy requires exactly one root epic matching epic_key".to_owned(),
            ));
        }
        for node in &self.nodes {
            match node.kind {
                GitHubIssueKind::Epic => {}
                GitHubIssueKind::Track => {
                    if node.parent_key.as_deref() != Some(self.epic_key.as_str()) {
                        return Err(ContractError::Invariant(format!(
                            "track issue `{}` must be a child of the roadmap epic",
                            node.key
                        )));
                    }
                }
                GitHubIssueKind::Phase => {
                    let Some(parent_key) = node.parent_key.as_deref() else {
                        return Err(ContractError::Invariant(format!(
                            "phase issue `{}` requires a parent track",
                            node.key
                        )));
                    };
                    let Some(parent) = by_key.get(parent_key) else {
                        return Err(ContractError::Invariant(format!(
                            "phase issue `{}` refers to unknown parent `{parent_key}`",
                            node.key
                        )));
                    };
                    if parent.kind != GitHubIssueKind::Track {
                        return Err(ContractError::Invariant(format!(
                            "phase issue `{}` parent must be a track",
                            node.key
                        )));
                    }
                }
                GitHubIssueKind::Task => {
                    let Some(parent_key) = node.parent_key.as_deref() else {
                        return Err(ContractError::Invariant(format!(
                            "task issue `{}` requires a parent phase",
                            node.key
                        )));
                    };
                    let Some(parent) = by_key.get(parent_key) else {
                        return Err(ContractError::Invariant(format!(
                            "task issue `{}` refers to unknown parent `{parent_key}`",
                            node.key
                        )));
                    };
                    if parent.kind != GitHubIssueKind::Phase {
                        return Err(ContractError::Invariant(format!(
                            "task issue `{}` parent must be a phase",
                            node.key
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Validate for GitHubProjectManifest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            GITHUB_PROJECT_SCHEMA_VERSION,
            "github_project.schema_version",
        )?;
        for (value, field) in [
            (&self.owner, "github_project.owner"),
            (&self.repository, "github_project.repository"),
            (&self.title, "github_project.title"),
            (&self.short_description, "github_project.short_description"),
            (&self.readme_path, "github_project.readme_path"),
            (&self.visibility, "github_project.visibility"),
        ] {
            require_text(value, field)?;
        }
        if self.project_number.is_some() || self.apply_permitted {
            return Err(ContractError::Invariant(
                "source Project manifest cannot contain a remote number or authorise writes"
                    .to_owned(),
            ));
        }
        if self.fields.is_empty() || self.views.is_empty() {
            return Err(ContractError::EmptyCollection(
                "github_project.fields_or_views",
            ));
        }
        let mut field_names = BTreeSet::new();
        for field in &self.fields {
            require_text(&field.name, "github_project.field.name")?;
            if !field_names.insert(field.name.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "duplicate Project field `{}`",
                    field.name
                )));
            }
            if field.data_type == GitHubProjectFieldDataType::SingleSelect {
                if field.options.is_empty() {
                    return Err(ContractError::EmptyCollection(
                        "github_project.field.options",
                    ));
                }
            } else if !field.options.is_empty() {
                return Err(ContractError::Invariant(format!(
                    "non-select Project field `{}` declares options",
                    field.name
                )));
            }
        }
        if self.sync.delete_policy != "never"
            || self.sync.archive_policy != "never_automatic"
            || self.sync.promotion_policy != "remote_state_cannot_promote_evidence"
        {
            return Err(ContractError::Invariant(
                "Project synchronisation must be non-destructive and evidence-neutral".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepared_suite(automatic_promotion: bool) -> ConsumerContractSuite {
        ConsumerContractSuite {
            schema_version: CONSUMER_CONTRACT_SUITE_SCHEMA_VERSION.to_owned(),
            generated_at: "source-epoch:2026-08-06".to_owned(),
            interactions: vec![ConsumerContractInteraction {
                id: "producer-to-consumer".to_owned(),
                integration_id: "integration-a".to_owned(),
                producer: "example/producer".to_owned(),
                consumer: "example/consumer".to_owned(),
                contract_version: "example.contract.v1".to_owned(),
                producer_contracts: vec![
                    "external://example/producer/schema.json@deadbeef".to_owned(),
                ],
                consumer_contracts: vec!["contracts/schema.json".to_owned()],
                fixture_paths: vec!["contracts/example.json".to_owned()],
                producer_gates: vec!["external: producer-test".to_owned()],
                consumer_gates: vec!["consumer-test".to_owned()],
                failure_semantics: "Disable the integration.".to_owned(),
                automatic_promotion,
                status: ConsumerContractStatus::PreparedNotExecuted,
            }],
            claim_boundary: "Prepared contracts are not downstream proof.".to_owned(),
        }
    }

    #[test]
    fn prepared_consumer_contract_suite_is_valid() {
        assert_eq!(prepared_suite(false).validate(), Ok(()));
    }

    #[test]
    fn automatic_consumer_contract_promotion_is_rejected() {
        assert!(matches!(
            prepared_suite(true).validate(),
            Err(ContractError::Invariant(_))
        ));
    }
}
