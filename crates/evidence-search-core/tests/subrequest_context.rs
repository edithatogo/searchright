//! Deterministic core scheduling tests; no sockets or live provider calls.
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use evidence_search_contracts::{
    CompiledStrategy, ExecutionPolicy, ProviderCapability, ProviderManifest, ProviderPage,
    ProviderSupportLevel, SearchDialect, SearchRequest, TranslationFidelity,
};
use evidence_search_core::{
    MemoryPageCache, PageExecutionContext, ProviderError, ProviderMode, ProviderRegistry,
    SearchProvider,
};
use tokio::{sync::Mutex, time::Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request() -> SearchRequest {
    SearchRequest {
        review_id: "fixture-review".into(),
        run_id: "fixture-run".into(),
        strategy: CompiledStrategy {
            schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.into(),
            strategy_id: "fixture-strategy".into(),
            dialect: SearchDialect::GenericBoolean,
            query: "fixture".into(),
            warnings: Vec::new(),
            fidelity: TranslationFidelity::Exact,
            review_required: false,
            loss_codes: Vec::new(),
            compilation_hash: "fixture-hash".into(),
            compiler_version: "fixture".into(),
        },
        cursor: None,
        page_size: 1,
        policy: ExecutionPolicy {
            live_enabled: false,
            max_records: 1,
            max_pages: 1,
            timeout_seconds: 10,
            total_timeout_seconds: Some(20),
            max_retries: 0,
            min_interval_ms: 0,
            retry_base_delay_ms: Some(1),
            retry_max_delay_ms: Some(100),
            max_response_bytes: Some(100),
            replay_enabled: true,
            cache_write_enabled: false,
        },
    }
}

fn page() -> ProviderPage {
    ProviderPage {
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.into(),
        records: Vec::new(),
        next_cursor: None,
        total_available: Some(0),
        diagnostics: BTreeMap::new(),
    }
}

struct MultiCall {
    id: &'static str,
    calls: Arc<Mutex<Vec<Instant>>>,
    factory_calls: Arc<AtomicUsize>,
    steps: usize,
    interval: u64,
    retry_after: Option<u64>,
    live: bool,
}

impl MultiCall {
    fn fixture(id: &'static str, calls: Arc<Mutex<Vec<Instant>>>, steps: usize) -> Self {
        Self {
            id,
            calls,
            factory_calls: Arc::new(AtomicUsize::new(0)),
            steps,
            interval: 0,
            retry_after: None,
            live: false,
        }
    }
}

#[async_trait]
impl SearchProvider for MultiCall {
    fn manifest(&self) -> ProviderManifest {
        ProviderManifest {
            provider_id: self.id.into(),
            display_name: "Synthetic subrequest fixture".into(),
            version: "1".into(),
            support_level: if self.live {
                ProviderSupportLevel::OptInLive
            } else {
                ProviderSupportLevel::FixtureBacked
            },
            capabilities: vec![ProviderCapability::Search],
            allowed_hosts: if self.live {
                vec!["example.test".into()]
            } else {
                Vec::new()
            },
            authentication_required: false,
            licensed: false,
            default_min_interval_ms: self.interval,
            policy_notes: Vec::new(),
        }
    }

    fn mode(&self) -> ProviderMode {
        if self.live {
            ProviderMode::Live
        } else {
            ProviderMode::Fixture
        }
    }

    fn endpoint_label(&self) -> Option<String> {
        self.live.then(|| "https://example.test/search".into())
    }

    async fn execute_page(&self, _: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        Err(ProviderError::InvalidRequest(
            "context-aware fixture requires registry admission".into(),
        ))
    }

    async fn execute_page_with_context(
        &self,
        _: &SearchRequest,
        context: &mut PageExecutionContext<'_>,
    ) -> Result<ProviderPage, ProviderError> {
        for _ in 0..self.steps {
            context
                .run_subrequest(|| {
                    self.factory_calls.fetch_add(1, Ordering::SeqCst);
                    async {
                        self.calls.lock().await.push(Instant::now());
                        Ok(())
                    }
                })
                .await?;
        }
        if let Some(retry_after_ms) = self.retry_after {
            return Err(ProviderError::RateLimited {
                provider: self.id.into(),
                retry_after_ms: Some(retry_after_ms),
            });
        }
        Ok(page())
    }
}

#[tokio::test(start_paused = true)]
async fn each_subrequest_is_spaced_without_double_initial_delay() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 2);
    provider.interval = 250;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let start = Instant::now();
    registry.execute("fixture", request(), "fixture").await?;
    let observed = calls.lock().await.clone();
    assert_eq!(
        observed.as_slice(),
        &[start, start + Duration::from_millis(250)]
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn distinct_providers_share_group_floor_and_concurrent_admission() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut first = MultiCall::fixture("first", calls.clone(), 2);
    first.interval = 100;
    let second = MultiCall::fixture("second", calls.clone(), 2);
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(Arc::new(first), "shared")?;
    registry.register_with_rate_group(Arc::new(second), "shared")?;
    let (a, b) = tokio::join!(
        registry.execute("first", request(), "fixture"),
        registry.execute("second", request(), "fixture")
    );
    a?;
    b?;
    let mut observed = calls.lock().await.clone();
    observed.sort();
    assert_eq!(observed.len(), 4);
    assert!(observed.windows(2).all(
        |pair| matches!(pair, [first, second] if *second - *first >= Duration::from_millis(100))
    ));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn page_deadline_prevents_second_factory_invocation() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 2);
    let factories = provider.factory_calls.clone();
    provider.interval = 2_000;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let mut bounded = request();
    bounded.policy.timeout_seconds = 1;
    assert!(matches!(
        registry.execute("fixture", bounded, "fixture").await,
        Err(ProviderError::Timeout { .. })
    ));
    assert_eq!(calls.lock().await.len(), 1);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_during_admission_never_invokes_next_operation() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 2);
    let factories = provider.factory_calls.clone();
    provider.interval = 2_000;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let task = tokio::spawn(async move { registry.execute("fixture", request(), "fixture").await });
    tokio::task::yield_now().await;
    assert_eq!(calls.lock().await.len(), 1);
    task.abort();
    assert!(task.await.is_err());
    tokio::time::advance(Duration::from_secs(3)).await;
    assert_eq!(calls.lock().await.len(), 1);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn subrequest_limit_is_finite_and_shared_across_retries() -> TestResult {
    for retry in [false, true] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut provider =
            MultiCall::fixture("fixture", calls.clone(), if retry { 17 } else { 33 });
        provider.retry_after = retry.then_some(0);
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(provider))?;
        let mut bounded = request();
        bounded.policy.max_retries = 2;
        assert!(matches!(
            registry.execute("fixture", bounded, "fixture").await,
            Err(ProviderError::BudgetExceeded {
                kind: "subrequests_per_page",
                limit: 32
            })
        ));
        assert_eq!(calls.lock().await.len(), 32);
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn retry_after_above_policy_fails_without_early_retry() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 1);
    provider.retry_after = Some(2_000);
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let mut bounded = request();
    bounded.policy.max_retries = 2;
    assert!(matches!(
        registry.execute("fixture", bounded, "fixture").await,
        Err(ProviderError::RateLimited {
            retry_after_ms: Some(2_000),
            ..
        })
    ));
    assert_eq!(calls.lock().await.len(), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn disabled_live_and_cache_replay_do_not_invoke_subrequests() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 2);
    let denied_factories = provider.factory_calls.clone();
    provider.live = true;
    let mut denied = ProviderRegistry::new();
    denied.register(Arc::new(provider))?;
    assert!(matches!(
        denied.execute("fixture", request(), "fixture").await,
        Err(ProviderError::LiveDisabled(_))
    ));
    assert!(calls.lock().await.is_empty());
    assert_eq!(denied_factories.load(Ordering::SeqCst), 0);
    let mut registry = ProviderRegistry::new()
        .with_cache(Arc::new(MemoryPageCache::new()), "fixture-authority")?;
    let provider = MultiCall::fixture("fixture", calls.clone(), 2);
    let factories = provider.factory_calls.clone();
    registry.register(Arc::new(provider))?;
    let mut cached = request();
    cached.policy.cache_write_enabled = true;
    registry
        .execute("fixture", cached.clone(), "fixture")
        .await?;
    let replay = registry.execute("fixture", cached, "fixture").await?;
    assert_eq!(replay.receipt.execution_mode, "replay");
    assert_eq!(calls.lock().await.len(), 2);
    assert_eq!(factories.load(Ordering::SeqCst), 2);
    Ok(())
}

#[test]
fn rate_group_names_fail_closed() {
    for name in ["", " ", "group/name", "secret@example.test", "\n"] {
        let mut registry = ProviderRegistry::new();
        assert!(
            registry
                .register_with_rate_group(
                    Arc::new(MultiCall::fixture(
                        "fixture",
                        Arc::new(Mutex::new(Vec::new())),
                        1
                    )),
                    name
                )
                .is_err()
        );
        assert!(registry.manifests().is_empty());
    }
}

#[tokio::test(start_paused = true)]
async fn delayed_wakeups_cannot_burst_queued_subrequests() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("first", calls.clone(), 2);
    provider.interval = 100;
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(Arc::new(provider), "shared")?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("second", calls.clone(), 2)),
        "shared",
    )?;
    let task = tokio::spawn(async move {
        let (first, second) = tokio::join!(
            registry.execute("first", request(), "fixture"),
            registry.execute("second", request(), "fixture"),
        );
        first?;
        second?;
        Ok::<(), ProviderError>(())
    });
    tokio::task::yield_now().await;
    assert_eq!(calls.lock().await.len(), 1);
    tokio::time::advance(Duration::from_secs(1)).await;
    task.await??;
    let observed = calls.lock().await.clone();
    assert_eq!(observed.len(), 4);
    assert!(observed.windows(2).all(
        |pair| matches!(pair, [first, second] if *second - *first >= Duration::from_millis(100))
    ));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn request_floor_cannot_be_weakened_but_other_groups_are_independent() -> TestResult {
    let shared_calls = Arc::new(Mutex::new(Vec::new()));
    let independent_calls = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("first", shared_calls.clone(), 1)),
        "shared",
    )?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("second", shared_calls.clone(), 1)),
        "shared",
    )?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture(
            "independent",
            independent_calls.clone(),
            1,
        )),
        "other",
    )?;
    let mut stronger = request();
    stronger.policy.min_interval_ms = 500;
    let start = Instant::now();
    registry.execute("first", stronger, "fixture").await?;
    registry
        .execute("independent", request(), "fixture")
        .await?;
    assert_eq!(independent_calls.lock().await.as_slice(), &[start]);
    registry.execute("second", request(), "fixture").await?;
    assert_eq!(
        shared_calls.lock().await.as_slice(),
        &[start, start + Duration::from_millis(500)]
    );
    Ok(())
}

struct LegacyCall(MultiCall);

#[async_trait]
impl SearchProvider for LegacyCall {
    fn manifest(&self) -> ProviderManifest {
        self.0.manifest()
    }
    fn mode(&self) -> ProviderMode {
        self.0.mode()
    }
    fn endpoint_label(&self) -> Option<String> {
        self.0.endpoint_label()
    }
    async fn execute_page(&self, _: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        self.0.calls.lock().await.push(Instant::now());
        Ok(page())
    }
}

#[tokio::test(start_paused = true)]
async fn legacy_default_retains_one_page_admission_and_total_deadline() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 1);
    provider.interval = 2_000;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(LegacyCall(provider)))?;
    registry.execute("fixture", request(), "fixture").await?;
    let mut bounded = request();
    bounded.policy.timeout_seconds = 1;
    bounded.policy.total_timeout_seconds = Some(1);
    assert!(matches!(
        registry.execute("fixture", bounded, "fixture").await,
        Err(ProviderError::BudgetExceeded {
            kind: "total_timeout_seconds",
            limit: 1
        })
    ));
    assert_eq!(calls.lock().await.len(), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn legacy_page_timeout_now_includes_rate_wait_even_with_larger_total_budget() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 1);
    provider.interval = 2_000;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(LegacyCall(provider)))?;
    registry.execute("fixture", request(), "fixture").await?;
    let mut bounded = request();
    bounded.policy.timeout_seconds = 1;
    assert!(matches!(
        registry.execute("fixture", bounded, "fixture").await,
        Err(ProviderError::Timeout {
            timeout_seconds: 1,
            ..
        })
    ));
    assert_eq!(calls.lock().await.len(), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn denied_request_does_not_raise_shared_rate_floor() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut denied = MultiCall::fixture("denied", calls.clone(), 1);
    denied.live = true;
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(Arc::new(denied), "shared")?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("fixture", calls.clone(), 2)),
        "shared",
    )?;
    let mut strong = request();
    strong.policy.min_interval_ms = 5_000;
    assert!(matches!(
        registry.execute("denied", strong, "fixture").await,
        Err(ProviderError::LiveDisabled(_))
    ));
    let mut normal = request();
    normal.policy.min_interval_ms = 100;
    let start = Instant::now();
    registry.execute("fixture", normal, "fixture").await?;
    assert_eq!(
        calls.lock().await.as_slice(),
        &[start, start + Duration::from_millis(100)]
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cache_only_request_does_not_raise_shared_rate_floor() -> TestResult {
    let cache = Arc::new(MemoryPageCache::new());
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut seeded = request();
    seeded.policy.cache_write_enabled = true;
    seeded.policy.min_interval_ms = 5_000;
    let mut seed_registry =
        ProviderRegistry::new().with_cache(cache.clone(), "fixture-authority")?;
    seed_registry.register(Arc::new(MultiCall::fixture("cached", calls.clone(), 1)))?;
    seed_registry
        .execute("cached", seeded.clone(), "fixture")
        .await?;
    calls.lock().await.clear();
    let mut registry = ProviderRegistry::new().with_cache(cache, "fixture-authority")?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("cached", calls.clone(), 1)),
        "shared",
    )?;
    registry.register_with_rate_group(
        Arc::new(MultiCall::fixture("fixture", calls.clone(), 2)),
        "shared",
    )?;
    let replay = registry.execute("cached", seeded, "fixture").await?;
    assert_eq!(replay.receipt.execution_mode, "replay");
    let mut normal = request();
    normal.policy.min_interval_ms = 100;
    let start = Instant::now();
    registry.execute("fixture", normal, "fixture").await?;
    assert_eq!(
        calls.lock().await.as_slice(),
        &[start, start + Duration::from_millis(100)]
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn lock_queued_cancellation_and_deadline_never_invoke_operation() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 1);
    let factories = provider.factory_calls.clone();
    provider.interval = 2_000;
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let registry = Arc::new(registry);
    registry.execute("fixture", request(), "fixture").await?;
    let holder_registry = registry.clone();
    let holder = tokio::spawn(async move {
        holder_registry
            .execute("fixture", request(), "fixture")
            .await
    });
    tokio::task::yield_now().await;
    let cancelled_registry = registry.clone();
    let cancelled = tokio::spawn(async move {
        cancelled_registry
            .execute("fixture", request(), "fixture")
            .await
    });
    tokio::task::yield_now().await;
    cancelled.abort();
    assert!(cancelled.await.is_err());
    let mut short = request();
    short.policy.timeout_seconds = 1;
    short.policy.total_timeout_seconds = Some(1);
    assert!(matches!(
        registry.execute("fixture", short, "fixture").await,
        Err(ProviderError::BudgetExceeded {
            kind: "total_timeout_seconds",
            limit: 1
        })
    ));
    assert_eq!(calls.lock().await.len(), 1);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    holder.await??;
    assert_eq!(calls.lock().await.len(), 2);
    assert_eq!(factories.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn overall_deadline_is_not_reset_between_page_retries() -> TestResult {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut provider = MultiCall::fixture("fixture", calls.clone(), 1);
    provider.retry_after = Some(600);
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(provider))?;
    let mut bounded = request();
    bounded.policy.timeout_seconds = 1;
    bounded.policy.total_timeout_seconds = Some(1);
    bounded.policy.max_retries = 3;
    bounded.policy.retry_max_delay_ms = Some(1_000);
    assert!(matches!(
        registry.execute("fixture", bounded, "fixture").await,
        Err(ProviderError::BudgetExceeded {
            kind: "total_timeout_seconds",
            limit: 1
        })
    ));
    assert_eq!(calls.lock().await.len(), 2);
    Ok(())
}
