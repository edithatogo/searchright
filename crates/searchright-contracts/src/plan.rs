use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, REVIEW_PLAN_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Kind of evidence synthesis being planned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    /// Conventional systematic review.
    Systematic,
    /// Scoping review.
    Scoping,
    /// Rapid review with explicit shortcuts.
    Rapid,
    /// Living systematic review.
    Living,
    /// Evidence map.
    EvidenceMap,
    /// Umbrella review or overview of reviews.
    Umbrella,
    /// Custom review kind retained verbatim.
    Other(String),
}

/// Framework used to decompose a research question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkKind {
    /// Population, intervention, comparison, outcome.
    Pico,
    /// Population, exposure, comparator, outcome, study design.
    Pecos,
    /// Population, concept, context.
    Pcc,
    /// Sample, phenomenon, design, evaluation, research type.
    Spider,
    /// Population, exposure, outcome.
    Peo,
    /// A named custom framework.
    Custom(String),
}

/// Structured question framework with named elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QuestionFramework {
    /// Framework type.
    pub kind: FrameworkKind,
    /// Named framework elements; keys retain methodological labels.
    pub elements: BTreeMap<String, String>,
}

impl Validate for QuestionFramework {
    fn validate(&self) -> Result<(), ContractError> {
        if self.elements.is_empty() {
            return Err(ContractError::EmptyCollection(
                "question.framework.elements",
            ));
        }
        for (name, value) in &self.elements {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Err(ContractError::Invariant(
                    "question framework element names and values must be non-empty".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// Research question and optional explanatory context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResearchQuestion {
    /// Natural-language question.
    pub text: String,
    /// Structured decomposition.
    pub framework: QuestionFramework,
    /// Scope notes and assumptions.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Validate for ResearchQuestion {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.text, "question.text")?;
        self.framework.validate()
    }
}

/// Screening phase at which a criterion is normally applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningStage {
    /// Title and abstract stage.
    TitleAbstract,
    /// Full-text stage.
    FullText,
    /// Either stage, depending on available evidence.
    Any,
}

/// One explicit inclusion or exclusion criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EligibilityCriterion {
    /// Stable criterion identifier.
    pub id: String,
    /// Domain such as population, intervention, context, outcome or design.
    pub domain: String,
    /// Operational rule.
    pub rule: String,
    /// Why the rule is required.
    pub rationale: String,
    /// Stage where it is normally applied.
    pub stage: ScreeningStage,
    /// Lower values are evaluated first when selecting a primary exclusion reason.
    pub priority: u16,
}

impl Validate for EligibilityCriterion {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.id, "eligibility.criterion.id")?;
        require_text(&self.domain, "eligibility.criterion.domain")?;
        require_text(&self.rule, "eligibility.criterion.rule")?;
        require_text(&self.rationale, "eligibility.criterion.rationale")
    }
}

/// Complete inclusion/exclusion contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EligibilitySet {
    /// Inclusion criteria.
    pub include: Vec<EligibilityCriterion>,
    /// Exclusion criteria.
    pub exclude: Vec<EligibilityCriterion>,
    /// Version identifier used for amendments.
    pub version: String,
}

impl Validate for EligibilitySet {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.version, "eligibility.version")?;
        if self.include.is_empty() {
            return Err(ContractError::EmptyCollection("eligibility.include"));
        }
        let mut criterion_ids = BTreeSet::new();
        for criterion in self.include.iter().chain(&self.exclude) {
            criterion.validate()?;
            if !criterion_ids.insert(criterion.id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "eligibility criterion identifier `{}` is duplicated",
                    criterion.id
                )));
            }
        }
        Ok(())
    }
}

/// Broad source class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InformationSourceKind {
    /// Bibliographic database.
    Database,
    /// Study or trial register.
    Register,
    /// Repository.
    Repository,
    /// Website or online resource.
    Website,
    /// Citation search.
    CitationSearch,
    /// Contact with investigators or organisations.
    Contact,
    /// Handsearching or browsing.
    Handsearch,
    /// Grey literature source.
    GreyLiterature,
    /// Imported source not queried by Searchright.
    Import,
    /// Other declared source.
    Other(String),
}

/// Planned information source and platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InformationSource {
    /// Stable source identifier.
    pub id: String,
    /// Database/resource name.
    pub name: String,
    /// Platform or interface, when distinct from the database.
    pub platform: Option<String>,
    /// Source class.
    pub kind: InformationSourceKind,
    /// Provider adapter identifier or `manual_import`.
    pub provider: String,
    /// Whether the source is required by the protocol.
    pub required: bool,
    /// Terms/licence or access notes.
    #[serde(default)]
    pub access_notes: Vec<String>,
}

impl Validate for InformationSource {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.id, "information_sources.id")?;
        require_text(&self.name, "information_sources.name")?;
        require_text(&self.provider, "information_sources.provider")?;
        if let Some(platform) = &self.platform {
            require_text(platform, "information_sources.platform")?;
        }
        if self.access_notes.iter().any(|note| note.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "information-source access notes must not contain empty values".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Protocol registration or deposit details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolRegistration {
    /// Registry or repository, for example PROSPERO or OSF.
    pub registry: Option<String>,
    /// Registration identifier or URL.
    pub identifier: Option<String>,
    /// Version of the deposited protocol.
    pub version: String,
    /// Amendments recorded after initial registration.
    #[serde(default)]
    pub amendments: Vec<String>,
}

impl Validate for ProtocolRegistration {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.version, "protocol.version")?;
        match (&self.registry, &self.identifier) {
            (Some(registry), Some(identifier)) => {
                require_text(registry, "protocol.registry")?;
                require_text(identifier, "protocol.identifier")?;
            }
            (None, None) => {}
            _ => {
                return Err(ContractError::Invariant(
                    "protocol registry and identifier must be supplied together".to_owned(),
                ));
            }
        }
        if self
            .amendments
            .iter()
            .any(|amendment| amendment.trim().is_empty())
        {
            return Err(ContractError::Invariant(
                "protocol amendments must not contain empty values".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Governance and review-authority settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewGovernance {
    /// Number of independent title/abstract reviewers.
    pub title_abstract_reviewers: u8,
    /// Number of independent full-text reviewers.
    pub full_text_reviewers: u8,
    /// Whether search strategy peer review is required.
    pub press_review_required: bool,
    /// Named roles permitted to amend the protocol.
    pub protocol_amendment_roles: Vec<String>,
    /// Free-text conflict resolution procedure.
    pub conflict_resolution: String,
}

impl Validate for ReviewGovernance {
    fn validate(&self) -> Result<(), ContractError> {
        if self.title_abstract_reviewers == 0 || self.full_text_reviewers == 0 {
            return Err(ContractError::Invariant(
                "at least one reviewer is required at each screening stage".to_owned(),
            ));
        }
        require_text(&self.conflict_resolution, "governance.conflict_resolution")?;
        if self.protocol_amendment_roles.is_empty() {
            return Err(ContractError::EmptyCollection(
                "governance.protocol_amendment_roles",
            ));
        }
        let mut roles = BTreeSet::new();
        for role in &self.protocol_amendment_roles {
            require_text(role, "governance.protocol_amendment_roles")?;
            if !roles.insert(role.trim()) {
                return Err(ContractError::Invariant(
                    "protocol amendment roles must be unique".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// Canonical plan for a review search and selection process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewPlan {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable review identifier.
    pub review_id: String,
    /// Review title.
    pub title: String,
    /// Review method.
    pub review_kind: ReviewKind,
    /// Research question.
    pub question: ResearchQuestion,
    /// Review objectives.
    #[serde(default)]
    pub objectives: Vec<String>,
    /// Eligibility contract.
    pub eligibility: EligibilitySet,
    /// Planned sources.
    pub information_sources: Vec<InformationSource>,
    /// Search strategy identifiers.
    pub strategy_ids: Vec<String>,
    /// Protocol registration.
    pub protocol: ProtocolRegistration,
    /// Screening/search governance.
    pub governance: ReviewGovernance,
}

impl Validate for ReviewPlan {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            REVIEW_PLAN_SCHEMA_VERSION,
            "schema_version",
        )?;
        require_text(&self.review_id, "review_id")?;
        require_text(&self.title, "title")?;
        self.question.validate()?;
        if self.objectives.is_empty() {
            return Err(ContractError::EmptyCollection("objectives"));
        }
        for objective in &self.objectives {
            require_text(objective, "objectives")?;
        }
        self.eligibility.validate()?;
        if self.information_sources.is_empty() {
            return Err(ContractError::EmptyCollection("information_sources"));
        }
        let mut source_ids = BTreeSet::new();
        for source in &self.information_sources {
            source.validate()?;
            if !source_ids.insert(source.id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "information-source identifier `{}` is duplicated",
                    source.id
                )));
            }
        }
        if self.strategy_ids.is_empty() {
            return Err(ContractError::EmptyCollection("strategy_ids"));
        }
        let mut strategy_ids = BTreeSet::new();
        for strategy_id in &self.strategy_ids {
            require_text(strategy_id, "strategy_ids")?;
            if !strategy_ids.insert(strategy_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "search-strategy identifier `{strategy_id}` is duplicated"
                )));
            }
        }
        self.protocol.validate()?;
        self.governance.validate()
    }
}
