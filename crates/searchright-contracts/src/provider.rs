use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{BibliographicRecord, CompiledStrategy};

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
