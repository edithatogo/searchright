use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use evidence_search_contracts::{
    BibliographicRecord, ProviderCapability, ProviderManifest, ProviderPage, SearchRequest,
    SourceReceipt, Validate,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::Mutex;

use crate::canonical_json;

/// Whether an adapter reads fixtures/replay or makes network calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderMode {
    /// Checked-in deterministic fixture.
    Fixture,
    /// Stored provider response replay.
    Replay,
    /// Live network execution.
    Live,
}

impl ProviderMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Replay => "replay",
            Self::Live => "live",
        }
    }
}

/// Provider adapter boundary shared across products.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Provider manifest.
    fn manifest(&self) -> ProviderManifest;
    /// Execution mode.
    fn mode(&self) -> ProviderMode;
    /// Redacted endpoint label for the receipt.
    fn endpoint_label(&self) -> Option<String>;
    /// Execute one bounded page.
    async fn execute_page(&self, request: &SearchRequest) -> Result<ProviderPage, ProviderError>;
    /// Whether an adapter failure is safe to retry under the bounded policy.
    fn is_retryable(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::Upstream { .. }
                | ProviderError::Timeout { .. }
                | ProviderError::RateLimited { .. }
                | ProviderError::HttpStatus { status: 429 | 500..=599, .. }
        )
    }
}

/// Content-addressed provider-page cache boundary.
#[async_trait]
pub trait PageCache: Send + Sync {
    /// Read one page by a non-secret cache key.
    async fn get(&self, key: &str) -> Result<Option<ProviderPage>, ProviderError>;
    /// Store one page by a non-secret cache key.
    async fn put(&self, key: &str, page: &ProviderPage) -> Result<(), ProviderError>;
}

/// In-memory cache for deterministic tests and single-process replay.
#[derive(Default)]
pub struct MemoryPageCache {
    pages: Mutex<BTreeMap<String, ProviderPage>>,
}

impl MemoryPageCache {
    /// Create an empty memory cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of cached pages.
    pub async fn len(&self) -> usize {
        self.pages.lock().await.len()
    }

    /// Whether the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.pages.lock().await.is_empty()
    }
}

#[async_trait]
impl PageCache for MemoryPageCache {
    async fn get(&self, key: &str) -> Result<Option<ProviderPage>, ProviderError> {
        Ok(self.pages.lock().await.get(key).cloned())
    }

    async fn put(&self, key: &str, page: &ProviderPage) -> Result<(), ProviderError> {
        self.pages
            .lock()
            .await
            .insert(key.to_owned(), page.clone());
        Ok(())
    }
}

/// Result of bounded multi-page execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Records in provider order.
    pub records: Vec<BibliographicRecord>,
    /// Redacted evidence receipt.
    pub receipt: SourceReceipt,
}

/// Provider or runtime failure.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Provider is not registered.
    #[error("provider `{0}` is not registered")]
    NotRegistered(String),
    /// Duplicate provider identifier.
    #[error("provider `{0}` is already registered")]
    AlreadyRegistered(String),
    /// Provider manifest violates runtime invariants.
    #[error("provider manifest is invalid: {0}")]
    InvalidManifest(String),
    /// Search request violates runtime invariants.
    #[error("search request is invalid: {0}")]
    InvalidRequest(String),
    /// Network access was requested without live permission.
    #[error("live provider `{0}` is disabled by execution policy")]
    LiveDisabled(String),
    /// Replay access was requested without replay permission.
    #[error("replay provider `{0}` is disabled by execution policy")]
    ReplayDisabled(String),
    /// Runtime budget was exceeded.
    #[error("provider execution exceeded {kind} budget of {limit}")]
    BudgetExceeded { kind: &'static str, limit: u64 },
    /// Provider returned a record that violated the canonical record contract.
    #[error("provider `{provider}` returned invalid record `{record_id}`: {message}")]
    InvalidRecord {
        provider: String,
        record_id: String,
        message: String,
    },
    /// Provider request exceeded its per-request timeout.
    #[error("provider `{provider}` timed out after {timeout_seconds} seconds")]
    Timeout { provider: String, timeout_seconds: u64 },
    /// Provider explicitly rate limited the request.
    #[error("provider `{provider}` rate limited the request")]
    RateLimited { provider: String, retry_after_ms: Option<u64> },
    /// Provider returned a non-success HTTP status.
    #[error("provider `{provider}` returned HTTP {status}: {message}")]
    HttpStatus {
        provider: String,
        status: u16,
        retry_after_ms: Option<u64>,
        message: String,
    },
    /// Provider response could not be decoded into its declared format.
    #[error("provider `{provider}` returned malformed {format}: {message}")]
    MalformedResponse { provider: String, format: &'static str, message: String },
    /// Provider or caller attempted an operation outside the capability policy.
    #[error("provider policy violation for `{provider}`: {message}")]
    PolicyViolation { provider: String, message: String },
    /// Execution was explicitly cancelled by a caller or task supervisor.
    #[error("provider `{provider}` execution was cancelled")]
    Cancelled { provider: String },
    /// Provider rejected or could not execute the request.
    #[error("provider `{provider}` failed: {message}")]
    Upstream { provider: String, message: String },
    /// Cache read or write failed.
    #[error("provider page cache failed: {0}")]
    Cache(String),
    /// Receipt serialisation failed.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// Timestamp formatting failed.
    #[error(transparent)]
    Timestamp(#[from] time::error::Format),
}

struct ProviderSlot {
    provider: Arc<dyn SearchProvider>,
    last_call: Mutex<Option<tokio::time::Instant>>,
}

/// Registry and bounded execution runtime.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: BTreeMap<String, ProviderSlot>,
    cache: Option<Arc<dyn PageCache>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a page cache and return the configured registry.
    #[must_use]
    pub fn with_cache(mut self, cache: Arc<dyn PageCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Replace or remove the configured page cache.
    pub fn set_cache(&mut self, cache: Option<Arc<dyn PageCache>>) {
        self.cache = cache;
    }

    /// Register a provider under its validated manifest identifier.
    pub fn register(&mut self, provider: Arc<dyn SearchProvider>) -> Result<(), ProviderError> {
        let manifest = provider.manifest();
        let mode = provider.mode();
        let endpoint_label = provider.endpoint_label();
        validate_manifest(&manifest, mode, endpoint_label.as_deref())?;
        if self.providers.contains_key(&manifest.provider_id) {
            return Err(ProviderError::AlreadyRegistered(manifest.provider_id));
        }
        self.providers.insert(
            manifest.provider_id,
            ProviderSlot {
                provider,
                last_call: Mutex::new(None),
            },
        );
        Ok(())
    }

    /// List provider manifests in stable identifier order.
    #[must_use]
    pub fn manifests(&self) -> Vec<ProviderManifest> {
        self.providers
            .values()
            .map(|slot| slot.provider.manifest())
            .collect()
    }

    /// Execute pages until the provider ends or the policy budget is reached.
    pub async fn execute(
        &self,
        provider_id: &str,
        mut request: SearchRequest,
        source_label: &str,
    ) -> Result<ExecutionResult, ProviderError> {
        validate_request(&request, source_label)?;
        let slot = self
            .providers
            .get(provider_id)
            .ok_or_else(|| ProviderError::NotRegistered(provider_id.to_owned()))?;
        let mode = slot.provider.mode();
        if mode == ProviderMode::Live && !request.policy.live_enabled {
            return Err(ProviderError::LiveDisabled(provider_id.to_owned()));
        }
        if mode == ProviderMode::Replay && !request.policy.replay_enabled {
            return Err(ProviderError::ReplayDisabled(provider_id.to_owned()));
        }

        let manifest = slot.provider.manifest();
        let started = tokio::time::Instant::now();
        let total_timeout = request.policy.total_timeout_seconds.map(Duration::from_secs);
        let minimum_interval_ms = request
            .policy
            .min_interval_ms
            .max(manifest.default_min_interval_ms);
        let request_fingerprint = canonical_json(&serde_json::json!({
            "provider_id": provider_id,
            "provider_version": &manifest.version,
            "runtime_version": env!("CARGO_PKG_VERSION"),
            "strategy": &request.strategy,
            "initial_cursor": &request.cursor,
            "page_size": request.page_size,
            "policy": &request.policy,
        }));

        let mut records = Vec::new();
        let mut pages = 0_u32;
        let mut cache_hits = 0_u32;
        let mut cache_writes = 0_u32;
        let mut warnings = Vec::new();
        loop {
            if let Some(limit) = total_timeout
                && started.elapsed() >= limit
            {
                return Err(ProviderError::BudgetExceeded {
                    kind: "total_timeout_seconds",
                    limit: limit.as_secs(),
                });
            }
            if pages >= request.policy.max_pages {
                if request.cursor.is_some() {
                    warnings.push("pagination stopped at max_pages budget".to_owned());
                }
                break;
            }
            let (page, cache_hit, cache_write) = self
                .execute_page_with_retries(
                    slot,
                    provider_id,
                    &manifest.version,
                    &request,
                    minimum_interval_ms,
                    &mut warnings,
                )
                .await?;
            pages += 1;
            cache_hits = cache_hits.saturating_add(if cache_hit { 1 } else { 0 });
            cache_writes = cache_writes.saturating_add(if cache_write { 1 } else { 0 });

            for record in page.records {
                if usize_to_u64(records.len()) >= request.policy.max_records {
                    warnings.push("records truncated at max_records budget".to_owned());
                    request.cursor = None;
                    break;
                }
                records.push(record);
            }
            if usize_to_u64(records.len()) >= request.policy.max_records {
                break;
            }
            request.cursor = page.next_cursor;
            if request.cursor.is_none() {
                break;
            }
        }

        let executed_at = OffsetDateTime::now_utc().format(&Rfc3339)?;
        let query_hash = {
            let bytes = serde_json::to_vec(&request_fingerprint)?;
            blake3::hash(&bytes).to_hex().to_string()
        };
        let result_digest = {
            let bytes = serde_json::to_vec(&records)?;
            blake3::hash(&bytes).to_hex().to_string()
        };
        let receipt_id = uuid::Uuid::now_v7().to_string();
        for record in &mut records {
            record.source_receipt_id.clone_from(&receipt_id);
            record.validate().map_err(|error| ProviderError::InvalidRecord {
                provider: provider_id.to_owned(),
                record_id: record.record_id.clone(),
                message: error.to_string(),
            })?;
        }
        let receipt = SourceReceipt {
            schema_version: evidence_search_contracts::SOURCE_RECEIPT_SCHEMA_VERSION.to_owned(),
            receipt_id,
            review_id: request.review_id,
            run_id: request.run_id,
            provider_id: provider_id.to_owned(),
            source_label: source_label.to_owned(),
            strategy_id: request.strategy.strategy_id,
            query_hash,
            executed_at,
            records_retrieved: usize_to_u64(records.len()),
            pages_retrieved: pages,
            execution_mode: if pages > 0 && cache_hits == pages {
                "replay".to_owned()
            } else if cache_hits > 0 {
                format!("mixed-{}-replay", mode.as_str())
            } else {
                mode.as_str().to_owned()
            },
            endpoint: slot.provider.endpoint_label(),
            policy: request.policy,
            provider_version: manifest.version,
            compiler_version: request.strategy.compiler_version,
            result_digest,
            cache_hits,
            cache_writes,
            warnings,
        };
        Ok(ExecutionResult { records, receipt })
    }

    async fn execute_page_with_retries(
        &self,
        slot: &ProviderSlot,
        provider_id: &str,
        provider_version: &str,
        request: &SearchRequest,
        minimum_interval_ms: u64,
        warnings: &mut Vec<String>,
    ) -> Result<(ProviderPage, bool, bool), ProviderError> {
        let cache_key = page_cache_key(provider_id, provider_version, request)?;
        if request.policy.replay_enabled
            && let Some(cache) = &self.cache
            && let Some(page) = cache.get(&cache_key).await?
        {
            warnings.push(format!("provider page replayed from cache `{cache_key}`"));
            return Ok((page, true, false));
        }

        let mut retry_count = 0_u8;
        loop {
            self.apply_rate_limit(slot, minimum_interval_ms).await;
            let result = tokio::time::timeout(
                Duration::from_secs(request.policy.timeout_seconds),
                slot.provider.execute_page(request),
            )
            .await
            .map_err(|_| ProviderError::Timeout {
                provider: provider_id.to_owned(),
                timeout_seconds: request.policy.timeout_seconds,
            })
            .and_then(|page| page);

            match result {
                Ok(page) => {
                    let mut wrote_cache = false;
                    if request.policy.cache_write_enabled
                        && let Some(cache) = &self.cache
                    {
                        cache.put(&cache_key, &page).await?;
                        warnings.push(format!("provider page cached as `{cache_key}`"));
                        wrote_cache = true;
                    }
                    return Ok((page, false, wrote_cache));
                }
                Err(error)
                    if retry_count < request.policy.max_retries
                        && slot.provider.is_retryable(&error) =>
                {
                    retry_count += 1;
                    warnings.push(format!(
                        "provider page retried after attempt {retry_count}: {error}"
                    ));
                    let provider_retry_after = match &error {
                        ProviderError::RateLimited { retry_after_ms, .. }
                        | ProviderError::HttpStatus { retry_after_ms, .. } => *retry_after_ms,
                        _ => None,
                    };
                    let base = request
                        .policy
                        .retry_base_delay_ms
                        .unwrap_or_else(|| minimum_interval_ms.max(100));
                    let maximum = request
                        .policy
                        .retry_max_delay_ms
                        .unwrap_or_else(|| base.saturating_mul(16));
                    let exponent = u32::from(retry_count.saturating_sub(1)).min(20);
                    let calculated = base.saturating_mul(1_u64 << exponent).min(maximum);
                    let delay_ms = provider_retry_after.unwrap_or(calculated).min(maximum);
                    warnings.push(format!("bounded retry delay: {delay_ms} ms"));
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn apply_rate_limit(&self, slot: &ProviderSlot, min_interval_ms: u64) {
        let delay = {
            let mut reserved = slot.last_call.lock().await;
            let now = tokio::time::Instant::now();
            let next = reserved
                .map(|previous| previous + Duration::from_millis(min_interval_ms))
                .map_or(now, |candidate| candidate.max(now));
            *reserved = Some(next);
            next.saturating_duration_since(now)
        };
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }
}

fn page_cache_key(
    provider_id: &str,
    provider_version: &str,
    request: &SearchRequest,
) -> Result<String, ProviderError> {
    let canonical = canonical_json(&serde_json::json!({
        "provider_id": provider_id,
        "provider_version": provider_version,
        "runtime_version": env!("CARGO_PKG_VERSION"),
        "strategy_id": request.strategy.strategy_id,
        "compilation_hash": request.strategy.compilation_hash,
        "compiler_version": request.strategy.compiler_version,
        "cursor": request.cursor,
        "page_size": request.page_size,
        "policy": request.policy,
    }));
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_manifest(
    manifest: &ProviderManifest,
    mode: ProviderMode,
    endpoint_label: Option<&str>,
) -> Result<(), ProviderError> {
    for (field, value) in [
        ("provider_id", manifest.provider_id.as_str()),
        ("display_name", manifest.display_name.as_str()),
        ("version", manifest.version.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ProviderError::InvalidManifest(format!(
                "`{field}` must not be empty"
            )));
        }
    }
    if !manifest
        .capabilities
        .contains(&ProviderCapability::Search)
    {
        return Err(ProviderError::InvalidManifest(
            "search adapters must declare the search capability".to_owned(),
        ));
    }
    if mode == ProviderMode::Live && manifest.allowed_hosts.is_empty() {
        return Err(ProviderError::InvalidManifest(
            "live adapters must declare at least one allowed endpoint host".to_owned(),
        ));
    }
    if manifest
        .allowed_hosts
        .iter()
        .any(|host| host.trim().is_empty() || host.contains('/') || host.contains('@'))
    {
        return Err(ProviderError::InvalidManifest(
            "allowed hosts must be bare, non-empty host names".to_owned(),
        ));
    }
    if mode == ProviderMode::Live {
        let endpoint_label = endpoint_label.ok_or_else(|| {
            ProviderError::InvalidManifest(
                "live adapters must expose a redacted HTTPS endpoint label".to_owned(),
            )
        })?;
        let endpoint = url::Url::parse(endpoint_label).map_err(|error| {
            ProviderError::InvalidManifest(format!(
                "live endpoint label is not a valid URL: {error}"
            ))
        })?;
        if endpoint.scheme() != "https" {
            return Err(ProviderError::InvalidManifest(
                "live endpoint labels must use HTTPS".to_owned(),
            ));
        }
        let host = endpoint.host_str().ok_or_else(|| {
            ProviderError::InvalidManifest(
                "live endpoint label must include a host".to_owned(),
            )
        })?;
        if !manifest.allowed_hosts.iter().any(|allowed| allowed == host) {
            return Err(ProviderError::InvalidManifest(format!(
                "live endpoint host `{host}` is not present in allowed_hosts"
            )));
        }
    }
    Ok(())
}

fn validate_request(request: &SearchRequest, source_label: &str) -> Result<(), ProviderError> {
    request
        .strategy
        .validate()
        .map_err(|error| ProviderError::InvalidRequest(error.to_string()))?;
    request
        .policy
        .validate()
        .map_err(|error| ProviderError::InvalidRequest(error.to_string()))?;
    for (field, value) in [
        ("review_id", request.review_id.as_str()),
        ("run_id", request.run_id.as_str()),
        ("strategy_id", request.strategy.strategy_id.as_str()),
        ("source_label", source_label),
    ] {
        if value.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(format!(
                "`{field}` must not be empty"
            )));
        }
    }
    if request.page_size == 0 {
        return Err(ProviderError::InvalidRequest(
            "page_size must be greater than zero".to_owned(),
        ));
    }
    if request.policy.max_pages == 0 {
        return Err(ProviderError::InvalidRequest(
            "max_pages must be greater than zero".to_owned(),
        ));
    }
    if request.policy.max_records == 0 {
        return Err(ProviderError::InvalidRequest(
            "max_records must be greater than zero".to_owned(),
        ));
    }
    if request.policy.timeout_seconds == 0 {
        return Err(ProviderError::InvalidRequest(
            "timeout_seconds must be greater than zero".to_owned(),
        ));
    }
    Ok(())
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use evidence_search_contracts::{
        CompiledStrategy, ExecutionPolicy, ProviderSupportLevel, SearchDialect,
    };

    use super::*;

    struct EmptyFixture;

    #[async_trait]
    impl SearchProvider for EmptyFixture {
        fn manifest(&self) -> ProviderManifest {
            ProviderManifest {
                provider_id: "empty".to_owned(),
                display_name: "Empty fixture".to_owned(),
                version: "1".to_owned(),
                support_level: ProviderSupportLevel::FixtureBacked,
                capabilities: vec![ProviderCapability::Search],
                allowed_hosts: Vec::new(),
                authentication_required: false,
                licensed: false,
                default_min_interval_ms: 0,
                policy_notes: Vec::new(),
            }
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Fixture
        }

        fn endpoint_label(&self) -> Option<String> {
            None
        }

        async fn execute_page(
            &self,
            _request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            Ok(ProviderPage {
                schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
                records: Vec::new(),
                next_cursor: None,
                total_available: Some(0),
                diagnostics: BTreeMap::new(),
            })
        }
    }

    fn request() -> SearchRequest {
        SearchRequest {
            review_id: "r1".to_owned(),
            run_id: "run1".to_owned(),
            strategy: CompiledStrategy {
                schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.to_owned(),
                strategy_id: "s1".to_owned(),
                dialect: SearchDialect::GenericBoolean,
                query: "example".to_owned(),
                warnings: Vec::new(),
                fidelity: evidence_search_contracts::TranslationFidelity::Exact,
                review_required: false,
                loss_codes: Vec::new(),
                compilation_hash: "hash".to_owned(),
                compiler_version: "test".to_owned(),
            },
            cursor: None,
            page_size: 100,
            policy: ExecutionPolicy {
                live_enabled: false,
                max_records: 100,
                max_pages: 2,
                timeout_seconds: 2,
                total_timeout_seconds: Some(5),
                max_retries: 0,
                min_interval_ms: 0,
                retry_base_delay_ms: Some(100),
                retry_max_delay_ms: Some(1_000),
                max_response_bytes: Some(1_000_000),
                replay_enabled: true,
                cache_write_enabled: false,
            },
        }
    }

    #[tokio::test]
    async fn fixture_executes_without_live_permission() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        let result = registry.execute("empty", request(), "fixture").await;
        assert!(result.is_ok());
        if let Ok(result) = result {
            assert_eq!(result.receipt.execution_mode, "fixture");
            assert_eq!(result.receipt.records_retrieved, 0);
        }
    }

    #[tokio::test]
    async fn page_cache_supports_write_then_replay() {
        let cache = Arc::new(MemoryPageCache::new());
        let mut registry = ProviderRegistry::new().with_cache(cache.clone());
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        let mut first_request = request();
        first_request.policy.cache_write_enabled = true;
        let first = registry.execute("empty", first_request, "fixture").await;
        assert!(first.is_ok());
        if let Ok(first) = first {
            assert_eq!(first.receipt.cache_writes, 1);
            assert_eq!(first.receipt.cache_hits, 0);
        }
        assert_eq!(cache.len().await, 1);

        let replay = registry.execute("empty", request(), "fixture").await;
        assert!(replay.is_ok());
        if let Ok(replay) = replay {
            assert_eq!(replay.receipt.cache_hits, 1);
            assert_eq!(replay.receipt.execution_mode, "replay");
        }
    }

    #[tokio::test]
    async fn zero_page_size_is_rejected() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        let mut invalid = request();
        invalid.page_size = 0;
        assert!(matches!(
            registry.execute("empty", invalid, "fixture").await,
            Err(ProviderError::InvalidRequest(_))
        ));
    }
}
