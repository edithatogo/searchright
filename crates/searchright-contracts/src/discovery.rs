use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, DISCOVERY_RUN_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Discovery method used beyond the primary database search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Backward reference checking.
    BackwardCitation,
    /// Forward citation searching.
    ForwardCitation,
    /// Similar-article discovery.
    SimilarArticles,
    /// Trial-registry searching.
    TrialRegistry,
    /// Repository or preprint searching.
    Repository,
    /// Grey literature searching.
    GreyLiterature,
    /// Handsearching.
    Handsearch,
    /// Contact with investigators or organisations.
    Contact,
    /// Another declared method.
    Other(String),
}

/// Evidence-bearing discovery edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryEdge {
    /// Stable edge identifier.
    pub edge_id: String,
    /// Seed record/report identifier.
    pub seed_id: String,
    /// Discovered identifier.
    pub discovered_id: String,
    /// Discovery method.
    pub method: DiscoveryMethod,
    /// Source/provider used.
    pub provider_id: String,
    /// Evidence receipt identifier.
    pub receipt_id: String,
}

/// One bounded supplementary-discovery run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryRun {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Stable run identifier.
    pub run_id: String,
    /// Discovery method.
    pub method: DiscoveryMethod,
    /// Seed identifiers.
    pub seed_ids: Vec<String>,
    /// Discovered edges.
    #[serde(default)]
    pub edges: Vec<DiscoveryEdge>,
    /// Maximum depth used for graph traversal.
    pub maximum_depth: u8,
    /// Maximum records allowed.
    pub maximum_records: u64,
    /// Whether human review is required before adding records to screening.
    pub requires_human_release: bool,
}

impl Validate for DiscoveryEdge {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.edge_id, "discovery.edge_id")?;
        require_text(&self.seed_id, "discovery.seed_id")?;
        require_text(&self.discovered_id, "discovery.discovered_id")?;
        require_text(&self.provider_id, "discovery.provider_id")?;
        require_text(&self.receipt_id, "discovery.receipt_id")?;
        if self.seed_id == self.discovered_id {
            return Err(ContractError::Invariant(
                "discovery edge must not point a seed to itself".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for DiscoveryRun {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DISCOVERY_RUN_SCHEMA_VERSION,
            "discovery.schema_version",
        )?;
        require_text(&self.review_id, "discovery.review_id")?;
        require_text(&self.run_id, "discovery.run_id")?;
        if self.seed_ids.is_empty() {
            return Err(ContractError::EmptyCollection("discovery.seed_ids"));
        }
        if self.maximum_depth == 0 || self.maximum_records == 0 {
            return Err(ContractError::Invariant(
                "discovery budgets must be greater than zero".to_owned(),
            ));
        }
        for edge in &self.edges {
            edge.validate()?;
        }
        Ok(())
    }
}
