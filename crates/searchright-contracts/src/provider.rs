use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    BibliographicRecord, CompiledStrategy, ContractError, PROVIDER_PAGE_SCHEMA_VERSION,
    SEARCH_RUN_SCHEMA_VERSION, SOURCE_RECEIPT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// Declared maturity of a provider adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSupportLevel {
    /// Contract and roadmap only.
    Planned,
    /// Deterministic fixture coverage.
    FixtureBacked,
    /// Explicit live smoke evidence.
    OptInLive,
    /// Maintained with current policy and compatibility evidence.
    Maintained,
}

/// Provider capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// Execute searches.
    Search,
    /// Paginate a result set using provider cursors or offsets.
    Pagination,
    /// Import exported records.
    Import,
    /// Retrieve one record by identifier.
    Lookup,
    /// Follow references cited by a work.
    BackwardCitation,
    /// Follow works citing a work.
    ForwardCitation,
    /// Subscribe to updates or alerts.
    Updates,
}

/// Provider/plugin descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderManifest {
    /// Stable provider identifier.
    pub provider_id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Adapter version.
    pub version: String,
    /// Support level.
    pub support_level: ProviderSupportLevel,
    /// Capabilities.
    pub capabilities: Vec<ProviderCapability>,
    /// Allowed endpoint hosts.
    pub allowed_hosts: Vec<String>,
    /// Whether authentication is required.
    pub authentication_required: bool,
    /// Whether access is normally licensed/subscription based.
    pub licensed: bool,
    /// Minimum interval between calls.
    pub default_min_interval_ms: u64,
    /// Terms/licensing notes.
    pub policy_notes: Vec<String>,
}

/// Runtime boundaries for a search execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPolicy {
    /// Explicitly permit network calls.
    pub live_enabled: bool,
    /// Maximum records across all pages.
    pub max_records: u64,
    /// Maximum pages.
    pub max_pages: u32,
    /// Per-request timeout.
    pub timeout_seconds: u64,
    /// Maximum retry count.
    pub max_retries: u8,
    /// Minimum interval between calls.
    pub min_interval_ms: u64,
    /// Whether a fixture/replay cache may be read.
    pub replay_enabled: bool,
    /// Whether successful pages may be written to the configured cache.
    #[serde(default)]
    pub cache_write_enabled: bool,
}

/// Provider request over a compiled query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    /// Review identifier.
    pub review_id: String,
    /// Run identifier.
    pub run_id: String,
    /// Compiled strategy.
    pub strategy: CompiledStrategy,
    /// Page cursor supplied by the provider.
    pub cursor: Option<String>,
    /// Requested page size.
    pub page_size: u32,
    /// Runtime policy.
    pub policy: ExecutionPolicy,
}

/// One page returned by a provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderPage {
    /// Contract identifier.
    pub schema_version: String,
    /// Normalised records.
    pub records: Vec<BibliographicRecord>,
    /// Cursor for the next page.
    pub next_cursor: Option<String>,
    /// Provider-reported total, if available.
    pub total_available: Option<u64>,
    /// Provider-specific non-secret diagnostics.
    #[serde(default)]
    pub diagnostics: BTreeMap<String, Value>,
}

/// Redacted evidence for one source execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceReceipt {
    /// Contract identifier.
    pub schema_version: String,
    /// Receipt identifier.
    pub receipt_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Run identifier.
    pub run_id: String,
    /// Provider identifier.
    pub provider_id: String,
    /// Information source and platform label.
    pub source_label: String,
    /// Search strategy identifier.
    pub strategy_id: String,
    /// Hash of rendered query and non-secret parameters.
    pub query_hash: String,
    /// Retrieval timestamp.
    pub executed_at: String,
    /// Number returned by source before deduplication.
    pub records_retrieved: u64,
    /// Pages fetched.
    pub pages_retrieved: u32,
    /// Whether execution was fixture, replay or live.
    pub execution_mode: String,
    /// Redacted endpoint host/path template.
    pub endpoint: Option<String>,
    /// Runtime policy snapshot.
    pub policy: ExecutionPolicy,
    /// Provider adapter version.
    pub provider_version: String,
    /// Query compiler version.
    pub compiler_version: String,
    /// Digest of the canonical returned record sequence.
    pub result_digest: String,
    /// Number of pages read from cache.
    pub cache_hits: u32,
    /// Number of pages written to cache.
    pub cache_writes: u32,
    /// Warnings or partial-result notes.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Search-run summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchRun {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Run identifier.
    pub run_id: String,
    /// Run purpose such as initial, update or validation.
    pub purpose: String,
    /// Start timestamp.
    pub started_at: String,
    /// Completion timestamp.
    pub completed_at: Option<String>,
    /// Receipts for every source.
    #[serde(default)]
    pub receipts: Vec<SourceReceipt>,
    /// Parent run for an update.
    pub supersedes_run_id: Option<String>,
}


impl Validate for ExecutionPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        if self.max_records == 0 || self.max_pages == 0 || self.timeout_seconds == 0 {
            return Err(ContractError::Invariant(
                "execution budgets and timeout must be greater than zero".to_owned(),
            ));
        }
        if self.cache_write_enabled && !self.replay_enabled {
            return Err(ContractError::Invariant(
                "cache writes require replay/cache support to be enabled".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for ProviderPage {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            PROVIDER_PAGE_SCHEMA_VERSION,
            "provider_page.schema_version",
        )?;
        for record in &self.records {
            record.validate()?;
        }
        if self.diagnostics.keys().any(|key| key.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "provider diagnostic keys must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for SourceReceipt {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            SOURCE_RECEIPT_SCHEMA_VERSION,
            "source_receipt.schema_version",
        )?;
        require_text(&self.receipt_id, "source_receipt.receipt_id")?;
        require_text(&self.review_id, "source_receipt.review_id")?;
        require_text(&self.run_id, "source_receipt.run_id")?;
        require_text(&self.provider_id, "source_receipt.provider_id")?;
        require_text(&self.source_label, "source_receipt.source_label")?;
        require_text(&self.strategy_id, "source_receipt.strategy_id")?;
        require_text(&self.query_hash, "source_receipt.query_hash")?;
        require_text(&self.executed_at, "source_receipt.executed_at")?;
        require_text(&self.execution_mode, "source_receipt.execution_mode")?;
        require_text(&self.provider_version, "source_receipt.provider_version")?;
        require_text(&self.compiler_version, "source_receipt.compiler_version")?;
        require_text(&self.result_digest, "source_receipt.result_digest")?;
        self.policy.validate()?;
        if self.pages_retrieved == 0 && self.records_retrieved > 0 {
            return Err(ContractError::Invariant(
                "a receipt cannot report records without a retrieved page".to_owned(),
            ));
        }
        if self.cache_hits.saturating_add(self.cache_writes) > self.pages_retrieved.saturating_mul(2) {
            return Err(ContractError::Invariant(
                "cache counters are inconsistent with retrieved pages".to_owned(),
            ));
        }
        if self.warnings.iter().any(|warning| warning.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "source-receipt warnings must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for SearchRun {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            SEARCH_RUN_SCHEMA_VERSION,
            "search_run.schema_version",
        )?;
        require_text(&self.review_id, "search_run.review_id")?;
        require_text(&self.run_id, "search_run.run_id")?;
        require_text(&self.purpose, "search_run.purpose")?;
        require_text(&self.started_at, "search_run.started_at")?;
        if let Some(completed_at) = self.completed_at.as_deref() {
            require_text(completed_at, "search_run.completed_at")?;
        }
        if let Some(parent) = self.supersedes_run_id.as_deref() {
            require_text(parent, "search_run.supersedes_run_id")?;
            if parent == self.run_id {
                return Err(ContractError::Invariant(
                    "a search run cannot supersede itself".to_owned(),
                ));
            }
        }
        for receipt in &self.receipts {
            receipt.validate()?;
            if receipt.run_id != self.run_id || receipt.review_id != self.review_id {
                return Err(ContractError::Invariant(
                    "search-run receipts must belong to the same run and review".to_owned(),
                ));
            }
        }
        Ok(())
    }
}
