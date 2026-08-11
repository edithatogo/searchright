use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use evidence_search_contracts::{
    BibliographicRecord, ProviderCapability, ProviderManifest, ProviderPage, SearchRequest,
    SourceReceipt, Validate,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::Mutex;

use crate::audit::canonical_json;

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
    const fn as_str(self) -> &'static str {
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
                | ProviderError::HttpStatus {
                    status: 429 | 500..=599,
                    ..
                }
        )
    }
}

/// Content-addressed provider-page cache boundary.
#[async_trait]
pub trait PageCache: Send + Sync {
    /// Read one page by a non-secret cache key.
    async fn get(&self, key: &str) -> Result<Option<CachedProviderPage>, ProviderError>;
    /// Store one page by a non-secret cache key.
    async fn put(&self, key: &str, page: &CachedProviderPage) -> Result<(), ProviderError>;
}

/// Corruption-detecting page envelope stored behind an authority-derived cache namespace.
///
/// The digest and request key detect accidental backend mix-ups or corruption. They are not an
/// authentication mechanism for an untrusted cache backend.
#[derive(Debug, Clone)]
pub struct CachedProviderPage {
    /// Exact cache key this envelope was created for.
    pub request_key: String,
    /// Provider page returned by the adapter.
    pub page: ProviderPage,
    /// BLAKE3 digest of the canonical page representation.
    pub response_digest: String,
}

/// In-memory cache for deterministic tests and single-process replay.
#[derive(Default)]
pub struct MemoryPageCache {
    pages: Mutex<BTreeMap<String, CachedProviderPage>>,
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
    async fn get(&self, key: &str) -> Result<Option<CachedProviderPage>, ProviderError> {
        Ok(self.pages.lock().await.get(key).cloned())
    }

    async fn put(&self, key: &str, page: &CachedProviderPage) -> Result<(), ProviderError> {
        self.pages.lock().await.insert(key.to_owned(), page.clone());
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
    BudgetExceeded {
        /// Name of the exhausted execution budget.
        kind: &'static str,
        /// Configured upper bound for that budget.
        limit: u64,
    },
    /// Provider returned a record that violated the canonical record contract.
    #[error("provider `{provider}` returned invalid record `{record_id}`: {message}")]
    InvalidRecord {
        /// Identifier of the provider that returned the record.
        provider: String,
        /// Provider-native identifier of the invalid record.
        record_id: String,
        /// Contract validation failure detail.
        message: String,
    },
    /// Provider request exceeded its per-request timeout.
    #[error("provider `{provider}` timed out after {timeout_seconds} seconds")]
    Timeout {
        /// Identifier of the provider whose request timed out.
        provider: String,
        /// Applied per-request timeout in seconds.
        timeout_seconds: u64,
    },
    /// Provider explicitly rate limited the request.
    #[error("provider `{provider}` rate limited the request")]
    RateLimited {
        /// Identifier of the provider that applied the rate limit.
        provider: String,
        /// Provider-supplied retry delay, when available.
        retry_after_ms: Option<u64>,
    },
    /// Provider returned a non-success HTTP status.
    #[error("provider `{provider}` returned HTTP {status}: {message}")]
    HttpStatus {
        /// Identifier of the provider that returned the status.
        provider: String,
        /// Non-success HTTP status code.
        status: u16,
        /// Provider-supplied retry delay, when available.
        retry_after_ms: Option<u64>,
        /// Redacted response detail suitable for diagnostics.
        message: String,
    },
    /// Provider response could not be decoded into its declared format.
    #[error("provider `{provider}` returned malformed {format}: {message}")]
    MalformedResponse {
        /// Identifier of the provider that returned the response.
        provider: String,
        /// Declared response format that could not be decoded.
        format: &'static str,
        /// Redacted decoding failure detail.
        message: String,
    },
    /// Provider or caller attempted an operation outside the capability policy.
    #[error("provider policy violation for `{provider}`: {message}")]
    PolicyViolation {
        /// Identifier of the provider involved in the denied operation.
        provider: String,
        /// Capability-policy denial detail.
        message: String,
    },
    /// Execution was explicitly cancelled by a caller or task supervisor.
    #[error("provider `{provider}` execution was cancelled")]
    Cancelled {
        /// Identifier of the provider whose execution was cancelled.
        provider: String,
    },
    /// Provider rejected or could not execute the request.
    #[error("provider `{provider}` failed: {message}")]
    Upstream {
        /// Identifier of the provider that could not execute the request.
        provider: String,
        /// Redacted upstream failure detail.
        message: String,
    },
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
    manifest: ProviderManifest,
    mode: ProviderMode,
    endpoint_label: Option<String>,
    last_call: Mutex<Option<tokio::time::Instant>>,
}

/// Registry and bounded execution runtime.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: BTreeMap<String, ProviderSlot>,
    cache: Option<Arc<dyn PageCache>>,
    cache_namespace: Option<String>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a page cache scoped by a namespace derived from authenticated tenant authority.
    #[must_use]
    pub fn with_cache(
        mut self,
        cache: Arc<dyn PageCache>,
        namespace: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let namespace = validate_cache_namespace(namespace.into())?;
        self.cache = Some(cache);
        self.cache_namespace = Some(namespace);
        Ok(self)
    }

    /// Replace or remove the configured page cache.
    pub fn set_cache(
        &mut self,
        cache: Option<Arc<dyn PageCache>>,
        namespace: Option<String>,
    ) -> Result<(), ProviderError> {
        if cache.is_some() != namespace.is_some() {
            return Err(ProviderError::Cache(
                "cache and authority namespace must be configured together".to_owned(),
            ));
        }
        let namespace = namespace.map(validate_cache_namespace).transpose()?;
        self.cache = cache;
        self.cache_namespace = namespace;
        Ok(())
    }

    /// Register a provider under its validated manifest identifier.
    pub fn register(&mut self, provider: Arc<dyn SearchProvider>) -> Result<(), ProviderError> {
        let manifest = provider.manifest();
        let mode = provider.mode();
        let endpoint_label = provider.endpoint_label();
        let endpoint_label = validate_manifest(&manifest, mode, endpoint_label.as_deref())?;
        if self.providers.contains_key(&manifest.provider_id) {
            return Err(ProviderError::AlreadyRegistered(manifest.provider_id));
        }
        self.providers.insert(
            manifest.provider_id.clone(),
            ProviderSlot {
                provider,
                manifest,
                mode,
                endpoint_label,
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
            .map(|slot| slot.manifest.clone())
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
        let mode = slot.mode;
        if mode == ProviderMode::Replay && !request.policy.replay_enabled {
            return Err(ProviderError::ReplayDisabled(provider_id.to_owned()));
        }

        let manifest = &slot.manifest;
        let started = tokio::time::Instant::now();
        let total_timeout = request
            .policy
            .total_timeout_seconds
            .map(Duration::from_secs);
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
                    started,
                    total_timeout,
                    minimum_interval_ms,
                    &mut warnings,
                )
                .await?;
            pages += 1;
            cache_hits = cache_hits.saturating_add(u32::from(cache_hit));
            cache_writes = cache_writes.saturating_add(u32::from(cache_write));

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
        let receipt_id = uuid::Uuid::now_v7().to_string();
        for record in &mut records {
            record.source_receipt_id.clone_from(&receipt_id);
            record
                .validate()
                .map_err(|error| ProviderError::InvalidRecord {
                    provider: provider_id.to_owned(),
                    record_id: record.record_id.clone(),
                    message: error.to_string(),
                })?;
        }
        let result_digest = canonical_digest(&records)?;
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
            endpoint: slot.endpoint_label.clone(),
            policy: request.policy,
            provider_version: manifest.version.clone(),
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
        started: tokio::time::Instant,
        total_timeout: Option<Duration>,
        minimum_interval_ms: u64,
        warnings: &mut Vec<String>,
    ) -> Result<(ProviderPage, bool, bool), ProviderError> {
        let cache_key = if self.cache.is_some() {
            Some(page_cache_key(
                provider_id,
                provider_version,
                slot.mode,
                slot.endpoint_label.as_deref(),
                self.cache_namespace.as_deref(),
                request,
            )?)
        } else {
            None
        };
        if request.policy.replay_enabled
            && let Some(cache) = &self.cache
            && let Some(cache_key) = cache_key.as_deref()
            && let Some(cached) = tokio::time::timeout(
                remaining_operation_timeout(
                    started,
                    total_timeout,
                    request.policy.timeout_seconds,
                )?,
                cache.get(cache_key),
            )
            .await
            .map_err(|_| ProviderError::Timeout {
                provider: provider_id.to_owned(),
                timeout_seconds: request.policy.timeout_seconds,
            })??
        {
            if cached.request_key != cache_key {
                return Err(ProviderError::Cache(
                    "cached provider page is bound to a different request key".to_owned(),
                ));
            }
            cached.page.validate().map_err(|error| {
                ProviderError::Cache(format!("cached provider page is invalid: {error}"))
            })?;
            let actual_digest = provider_page_digest(&cached.page)?;
            if actual_digest != cached.response_digest {
                return Err(ProviderError::Cache(
                    "cached provider page failed its response digest check".to_owned(),
                ));
            }
            warnings.push(format!("provider page replayed from cache `{cache_key}`"));
            return Ok((cached.page, true, false));
        }

        let mut retry_count = 0_u8;
        loop {
            if slot.mode == ProviderMode::Live && !request.policy.live_enabled {
                return Err(ProviderError::LiveDisabled(provider_id.to_owned()));
            }
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
                    page.validate().map_err(|error| ProviderError::Upstream {
                        provider: provider_id.to_owned(),
                        message: format!("provider page is invalid: {error}"),
                    })?;
                    let mut wrote_cache = false;
                    if request.policy.cache_write_enabled
                        && let Some(cache) = &self.cache
                        && let Some(cache_key) = cache_key.as_deref()
                    {
                        tokio::time::timeout(
                            remaining_operation_timeout(
                                started,
                                total_timeout,
                                request.policy.timeout_seconds,
                            )?,
                            cache.put(
                                cache_key,
                                &CachedProviderPage {
                                    request_key: cache_key.to_owned(),
                                    response_digest: provider_page_digest(&page)?,
                                    page: page.clone(),
                                },
                            ),
                        )
                        .await
                        .map_err(|_| ProviderError::Timeout {
                            provider: provider_id.to_owned(),
                            timeout_seconds: request.policy.timeout_seconds,
                        })??;
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
            drop(reserved);
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
    provider_mode: ProviderMode,
    endpoint_label: Option<&str>,
    cache_namespace: Option<&str>,
    request: &SearchRequest,
) -> Result<String, ProviderError> {
    let cache_namespace = cache_namespace.ok_or_else(|| {
        ProviderError::Cache("a non-empty cache namespace is required".to_owned())
    })?;
    if cache_namespace.trim().is_empty() {
        return Err(ProviderError::Cache(
            "a non-empty cache namespace is required".to_owned(),
        ));
    }
    let canonical = canonical_json(&serde_json::json!({
        "cache_namespace": cache_namespace,
        "provider_id": provider_id,
        "provider_version": provider_version,
        "provider_mode": provider_mode.as_str(),
        "endpoint": endpoint_label,
        "review_id": request.review_id,
        "runtime_version": env!("CARGO_PKG_VERSION"),
        "strategy": request.strategy,
        "cursor": request.cursor,
        "page_size": request.page_size,
        "policy": {
            "max_records": request.policy.max_records,
            "max_pages": request.policy.max_pages,
            "timeout_seconds": request.policy.timeout_seconds,
            "total_timeout_seconds": request.policy.total_timeout_seconds,
            "max_retries": request.policy.max_retries,
            "min_interval_ms": request.policy.min_interval_ms,
            "retry_base_delay_ms": request.policy.retry_base_delay_ms,
            "retry_max_delay_ms": request.policy.retry_max_delay_ms,
            "max_response_bytes": request.policy.max_response_bytes,
        },
    }));
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn provider_page_digest(page: &ProviderPage) -> Result<String, ProviderError> {
    canonical_digest(page)
}

fn canonical_digest<T: serde::Serialize>(value: &T) -> Result<String, ProviderError> {
    let value = serde_json::to_value(value)?;
    let canonical = canonical_json(&value);
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_cache_namespace(namespace: String) -> Result<String, ProviderError> {
    if namespace.trim().is_empty() {
        return Err(ProviderError::Cache(
            "a non-empty authority-derived cache namespace is required".to_owned(),
        ));
    }
    Ok(namespace)
}

fn validate_manifest(
    manifest: &ProviderManifest,
    mode: ProviderMode,
    endpoint_label: Option<&str>,
) -> Result<Option<String>, ProviderError> {
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
    if !manifest.capabilities.contains(&ProviderCapability::Search) {
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
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(ProviderError::InvalidManifest(
                "live endpoint labels must not contain credentials, a query, or a fragment"
                    .to_owned(),
            ));
        }
        let host = endpoint.host_str().ok_or_else(|| {
            ProviderError::InvalidManifest("live endpoint label must include a host".to_owned())
        })?;
        if !manifest.allowed_hosts.iter().any(|allowed| allowed == host) {
            return Err(ProviderError::InvalidManifest(format!(
                "live endpoint host `{host}` is not present in allowed_hosts"
            )));
        }
        return Ok(Some(endpoint.origin().ascii_serialization()));
    }
    Ok(endpoint_label.map(str::to_owned))
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

fn remaining_operation_timeout(
    started: tokio::time::Instant,
    total_timeout: Option<Duration>,
    per_operation_seconds: u64,
) -> Result<Duration, ProviderError> {
    let per_operation = Duration::from_secs(per_operation_seconds);
    if let Some(total_timeout) = total_timeout {
        let remaining = total_timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(ProviderError::BudgetExceeded {
                kind: "total_timeout_seconds",
                limit: total_timeout.as_secs(),
            });
        }
        return Ok(remaining.min(per_operation));
    }
    Ok(per_operation)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use evidence_search_contracts::{
        BibliographicRecord, CompiledStrategy, ExecutionPolicy, ProviderSupportLevel,
        RecordIdentifiers, RecordKind, SearchDialect,
    };

    use super::*;

    struct EmptyFixture;

    struct RecordFixture;

    struct CountingLive {
        calls: Arc<AtomicUsize>,
        endpoint: String,
    }

    struct TamperCache {
        envelope: CachedProviderPage,
    }

    struct StatefulEndpointLive {
        endpoint_calls: AtomicUsize,
    }

    struct PendingCache;
    struct PendingWriteCache;

    #[async_trait]
    impl PageCache for TamperCache {
        async fn get(&self, _key: &str) -> Result<Option<CachedProviderPage>, ProviderError> {
            Ok(Some(self.envelope.clone()))
        }

        async fn put(&self, _key: &str, _page: &CachedProviderPage) -> Result<(), ProviderError> {
            Ok(())
        }
    }

    #[async_trait]
    impl PageCache for PendingCache {
        async fn get(&self, _key: &str) -> Result<Option<CachedProviderPage>, ProviderError> {
            std::future::pending().await
        }

        async fn put(&self, _key: &str, _page: &CachedProviderPage) -> Result<(), ProviderError> {
            std::future::pending().await
        }
    }

    #[async_trait]
    impl PageCache for PendingWriteCache {
        async fn get(&self, _key: &str) -> Result<Option<CachedProviderPage>, ProviderError> {
            Ok(None)
        }

        async fn put(&self, _key: &str, _page: &CachedProviderPage) -> Result<(), ProviderError> {
            std::future::pending().await
        }
    }

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

    #[async_trait]
    impl SearchProvider for RecordFixture {
        fn manifest(&self) -> ProviderManifest {
            EmptyFixture.manifest()
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
                records: vec![BibliographicRecord {
                    schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                        .to_owned(),
                    record_id: "record-1".to_owned(),
                    source_receipt_id: "pending".to_owned(),
                    native_id: "native-1".to_owned(),
                    kind: RecordKind::JournalArticle,
                    identifiers: RecordIdentifiers::default(),
                    title: "Example".to_owned(),
                    abstract_text: None,
                    authors: Vec::new(),
                    container_title: None,
                    publication_year: None,
                    publication_date: None,
                    languages: Vec::new(),
                    subjects: Vec::new(),
                    urls: Vec::new(),
                    provider_metadata: serde_json::Value::Null,
                }],
                next_cursor: None,
                total_available: Some(1),
                diagnostics: BTreeMap::new(),
            })
        }
    }

    #[async_trait]
    impl SearchProvider for CountingLive {
        fn manifest(&self) -> ProviderManifest {
            ProviderManifest {
                provider_id: "live".to_owned(),
                display_name: "Counting live provider".to_owned(),
                version: "1".to_owned(),
                support_level: ProviderSupportLevel::OptInLive,
                capabilities: vec![ProviderCapability::Search],
                allowed_hosts: vec!["example.test".to_owned(), "other.example.test".to_owned()],
                authentication_required: false,
                licensed: false,
                default_min_interval_ms: 0,
                policy_notes: Vec::new(),
            }
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some(self.endpoint.clone())
        }

        async fn execute_page(
            &self,
            _request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            EmptyFixture.execute_page(_request).await
        }
    }

    #[async_trait]
    impl SearchProvider for StatefulEndpointLive {
        fn manifest(&self) -> ProviderManifest {
            CountingLive {
                calls: Arc::new(AtomicUsize::new(0)),
                endpoint: String::new(),
            }
            .manifest()
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            if self.endpoint_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                Some("https://example.test/search".to_owned())
            } else {
                Some("https://user:secret@example.test/search?api_key=secret".to_owned())
            }
        }

        async fn execute_page(
            &self,
            request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            EmptyFixture.execute_page(request).await
        }
    }

    fn request() -> SearchRequest {
        SearchRequest {
            review_id: "r1".to_owned(),
            run_id: "run1".to_owned(),
            strategy: CompiledStrategy {
                schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION
                    .to_owned(),
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

    fn registry_with_cache(cache: Arc<dyn PageCache>, namespace: &str) -> ProviderRegistry {
        let configured = ProviderRegistry::new().with_cache(cache, namespace);
        assert!(configured.is_ok());
        configured.unwrap_or_default()
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
        let mut registry = registry_with_cache(cache.clone(), "tenant-a");
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
    async fn receipt_digest_covers_returned_records() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.register(Arc::new(RecordFixture)).is_ok());
        let result = registry.execute("empty", request(), "fixture").await;
        assert!(result.is_ok());
        if let Ok(result) = result {
            let recomputed = canonical_digest(&result.records);
            assert!(recomputed.is_ok());
            assert_eq!(result.receipt.result_digest, recomputed.unwrap_or_default());
            assert!(
                result
                    .records
                    .iter()
                    .all(|record| record.source_receipt_id == result.receipt.receipt_id)
            );
        }
    }

    #[tokio::test]
    async fn live_cache_replays_without_live_authority_and_miss_fails_closed() {
        let cache = Arc::new(MemoryPageCache::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let provider = Arc::new(CountingLive {
            calls: calls.clone(),
            endpoint: "https://example.test/search".to_owned(),
        });
        let mut registry = registry_with_cache(cache.clone(), "tenant-a");
        assert!(registry.register(provider).is_ok());

        let mut online = request();
        online.policy.live_enabled = true;
        online.policy.cache_write_enabled = true;
        assert!(
            registry
                .execute("live", online, "live source")
                .await
                .is_ok()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let offline = request();
        let replay = registry.execute("live", offline, "live source").await;
        assert!(replay.is_ok());
        if let Ok(replay) = replay {
            assert_eq!(replay.receipt.execution_mode, "replay");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut miss = request();
        miss.cursor = Some("uncached-page".to_owned());
        assert!(matches!(
            registry.execute("live", miss, "live source").await,
            Err(ProviderError::LiveDisabled(provider)) if provider == "live"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut changed_query = request();
        changed_query.strategy.query = "different query with reused hash".to_owned();
        assert!(matches!(
            registry
                .execute("live", changed_query, "live source")
                .await,
            Err(ProviderError::LiveDisabled(provider)) if provider == "live"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cache_namespace_prevents_cross_authority_replay() {
        let cache = Arc::new(MemoryPageCache::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let mut writer = registry_with_cache(cache.clone(), "tenant-a");
        assert!(
            writer
                .register(Arc::new(CountingLive {
                    calls: calls.clone(),
                    endpoint: "https://example.test/search".to_owned(),
                }))
                .is_ok()
        );
        let mut online = request();
        online.policy.live_enabled = true;
        online.policy.cache_write_enabled = true;
        assert!(writer.execute("live", online, "live source").await.is_ok());

        let mut reader = registry_with_cache(cache.clone(), "tenant-b");
        assert!(
            reader
                .register(Arc::new(CountingLive {
                    calls: calls.clone(),
                    endpoint: "https://example.test/search".to_owned(),
                }))
                .is_ok()
        );
        assert!(matches!(
            reader.execute("live", request(), "live source").await,
            Err(ProviderError::LiveDisabled(provider)) if provider == "live"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut same_tenant_reader = registry_with_cache(cache.clone(), "tenant-a");
        assert!(
            same_tenant_reader
                .register(Arc::new(CountingLive {
                    calls: calls.clone(),
                    endpoint: "https://example.test/search".to_owned(),
                }))
                .is_ok()
        );
        let mut other_review = request();
        other_review.review_id = "r2".to_owned();
        assert!(matches!(
            same_tenant_reader
                .execute("live", other_review, "live source")
                .await,
            Err(ProviderError::LiveDisabled(provider)) if provider == "live"
        ));

        let mut other_endpoint = registry_with_cache(cache, "tenant-a");
        assert!(
            other_endpoint
                .register(Arc::new(CountingLive {
                    calls: calls.clone(),
                    endpoint: "https://other.example.test/search".to_owned(),
                }))
                .is_ok()
        );
        assert!(matches!(
            other_endpoint
                .execute("live", request(), "live source")
                .await,
            Err(ProviderError::LiveDisabled(provider)) if provider == "live"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cache_rejects_wrong_request_envelope() {
        let page = EmptyFixture.execute_page(&request()).await;
        assert!(page.is_ok());
        let page = page.unwrap_or_else(|_| ProviderPage {
            schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
            records: Vec::new(),
            next_cursor: None,
            total_available: None,
            diagnostics: BTreeMap::new(),
        });
        let digest = provider_page_digest(&page);
        assert!(digest.is_ok());
        let cache = Arc::new(TamperCache {
            envelope: CachedProviderPage {
                request_key: "wrong-request".to_owned(),
                page,
                response_digest: digest.unwrap_or_default(),
            },
        });
        let mut registry = registry_with_cache(cache, "tenant-a");
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        assert!(matches!(
            registry.execute("empty", request(), "fixture").await,
            Err(ProviderError::Cache(message)) if message.contains("different request key")
        ));
    }

    #[tokio::test]
    async fn cache_rejects_stale_response_digest() {
        let page = EmptyFixture.execute_page(&request()).await;
        assert!(page.is_ok());
        if let Ok(page) = page {
            let cache = Arc::new(TamperCache {
                envelope: CachedProviderPage {
                    request_key: page_cache_key(
                        "empty",
                        "1",
                        ProviderMode::Fixture,
                        None,
                        Some("tenant-a"),
                        &request(),
                    )
                    .unwrap_or_default(),
                    page,
                    response_digest: "stale".to_owned(),
                },
            });
            let mut registry = registry_with_cache(cache, "tenant-a");
            assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
            assert!(matches!(
                registry.execute("empty", request(), "fixture").await,
                Err(ProviderError::Cache(message)) if message.contains("response digest")
            ));
        }
    }

    #[tokio::test]
    async fn cache_rejects_semantically_invalid_pages_with_valid_digests() {
        let page = ProviderPage {
            schema_version: "invalid-schema".to_owned(),
            records: Vec::new(),
            next_cursor: None,
            total_available: None,
            diagnostics: BTreeMap::new(),
        };
        let digest = provider_page_digest(&page);
        assert!(digest.is_ok());
        let key = page_cache_key(
            "empty",
            "1",
            ProviderMode::Fixture,
            None,
            Some("tenant-a"),
            &request(),
        );
        assert!(key.is_ok());
        let cache = Arc::new(TamperCache {
            envelope: CachedProviderPage {
                request_key: key.unwrap_or_default(),
                page,
                response_digest: digest.unwrap_or_default(),
            },
        });
        let mut registry = registry_with_cache(cache, "tenant-a");
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        assert!(matches!(
            registry.execute("empty", request(), "fixture").await,
            Err(ProviderError::Cache(message)) if message.contains("invalid")
        ));
    }

    #[tokio::test]
    async fn registration_snapshots_the_validated_endpoint() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(StatefulEndpointLive {
            endpoint_calls: AtomicUsize::new(0),
        });
        assert!(registry.register(provider.clone()).is_ok());
        let mut online = request();
        online.policy.live_enabled = true;
        let result = registry.execute("live", online, "live source").await;
        assert!(result.is_ok());
        if let Ok(result) = result {
            assert_eq!(
                result.receipt.endpoint.as_deref(),
                Some("https://example.test")
            );
        }
        assert_eq!(provider.endpoint_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn endpoint_paths_are_redacted_to_the_validated_origin() {
        let mut registry = ProviderRegistry::new();
        assert!(
            registry
                .register(Arc::new(CountingLive {
                    calls: Arc::new(AtomicUsize::new(0)),
                    endpoint: "https://example.test/api-key/secret/search".to_owned(),
                }))
                .is_ok()
        );
        let mut online = request();
        online.policy.live_enabled = true;
        let result = registry.execute("live", online, "live source").await;
        assert!(result.is_ok());
        if let Ok(result) = result {
            assert_eq!(
                result.receipt.endpoint.as_deref(),
                Some("https://example.test")
            );
        }
    }

    #[tokio::test]
    async fn cache_reads_are_bounded_by_the_execution_timeout() {
        let mut registry = registry_with_cache(Arc::new(PendingCache), "tenant-a");
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        let mut bounded = request();
        bounded.policy.timeout_seconds = 1;
        bounded.policy.total_timeout_seconds = Some(1);
        let result = registry.execute("empty", bounded, "fixture").await;
        assert!(matches!(result, Err(ProviderError::Timeout { .. })));
    }

    #[tokio::test]
    async fn cache_writes_are_bounded_by_the_execution_timeout() {
        let mut registry = registry_with_cache(Arc::new(PendingWriteCache), "tenant-a");
        assert!(registry.register(Arc::new(EmptyFixture)).is_ok());
        let mut bounded = request();
        bounded.policy.timeout_seconds = 1;
        bounded.policy.total_timeout_seconds = Some(1);
        bounded.policy.cache_write_enabled = true;
        let result = registry.execute("empty", bounded, "fixture").await;
        assert!(matches!(result, Err(ProviderError::Timeout { .. })));
    }

    #[test]
    fn cache_configuration_rejects_missing_or_empty_authority_namespace() {
        let cache = Arc::new(MemoryPageCache::new());
        assert!(
            ProviderRegistry::new()
                .with_cache(cache.clone(), " ")
                .is_err()
        );
        let mut registry = ProviderRegistry::new();
        assert!(registry.set_cache(Some(cache), None).is_err());
    }

    #[test]
    fn canonical_digest_ignores_json_object_insertion_order() {
        let first = serde_json::from_str::<serde_json::Value>(r#"{"a":1,"b":2}"#);
        let second = serde_json::from_str::<serde_json::Value>(r#"{"b":2,"a":1}"#);
        assert!(first.is_ok() && second.is_ok());
        if let (Ok(first), Ok(second)) = (first, second) {
            assert_eq!(
                canonical_digest(&first).unwrap_or_default(),
                canonical_digest(&second).unwrap_or_default()
            );
        }
    }

    #[test]
    fn live_endpoint_labels_reject_secret_bearing_components() {
        for endpoint in [
            "https://user:secret@example.test/search",
            "https://example.test/search?api_key=secret",
            "https://example.test/search#secret",
        ] {
            let mut registry = ProviderRegistry::new();
            let result = registry.register(Arc::new(CountingLive {
                calls: Arc::new(AtomicUsize::new(0)),
                endpoint: endpoint.to_owned(),
            }));
            assert!(matches!(result, Err(ProviderError::InvalidManifest(_))));
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
