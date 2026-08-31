//! Offline parser-to-runtime receipt/cache evidence, not live transport evidence.
//! The `live` feature supplies Tokio only; no live adapter is constructed here.
#![cfg(feature = "live")]

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use evidence_search_contracts::{
    CompiledStrategy, ExecutionPolicy, ProviderManifest, ProviderPage, SearchDialect,
    SearchRequest, TranslationFidelity, Validate,
};
use evidence_search_core::{
    CachedProviderPage, ExecutionResult, MemoryPageCache, PageCache, ProviderError, ProviderMode,
    ProviderRegistry, SearchProvider, canonical_record_digest,
};
use searchright_connectors::{
    FixtureProvider, parse_crossref_page, parse_europe_pmc_page, parse_openalex_page,
    parse_pubmed_fetch_page, parse_pubmed_summary_page,
};
use serde_json::{Value, json};
use tokio::sync::Mutex;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Parser = fn(&Value) -> Result<ProviderPage, ProviderError>;

fn request() -> SearchRequest {
    SearchRequest {
        review_id: "synthetic-review".to_owned(),
        run_id: "synthetic-run".to_owned(),
        strategy: CompiledStrategy {
            schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.to_owned(),
            strategy_id: "synthetic-strategy".to_owned(),
            dialect: SearchDialect::GenericBoolean,
            query: "synthetic fixture".to_owned(),
            warnings: Vec::new(),
            fidelity: TranslationFidelity::Exact,
            review_required: false,
            loss_codes: Vec::new(),
            compilation_hash: "synthetic-compilation-hash".to_owned(),
            compiler_version: "runtime-fixture-test".to_owned(),
        },
        cursor: None,
        page_size: 100,
        policy: ExecutionPolicy {
            live_enabled: false,
            max_records: 100,
            max_pages: 2,
            timeout_seconds: 5,
            total_timeout_seconds: Some(15),
            max_retries: 0,
            min_interval_ms: 0,
            retry_base_delay_ms: Some(100),
            retry_max_delay_ms: Some(1_000),
            max_response_bytes: Some(1_000_000),
            replay_enabled: true,
            cache_write_enabled: true,
        },
    }
}

#[derive(Default)]
struct ObservedCache {
    inner: MemoryPageCache,
    writes: Mutex<Vec<(String, CachedProviderPage)>>,
    missing: Mutex<Option<String>>,
    corrupt: Mutex<Option<String>>,
    wrong_key: Mutex<Option<String>>,
    provider_calls: AtomicUsize,
}

struct CountedFixture {
    fixture: FixtureProvider,
    observations: Arc<ObservedCache>,
}

#[async_trait]
impl SearchProvider for CountedFixture {
    fn manifest(&self) -> ProviderManifest {
        self.fixture.manifest()
    }
    fn mode(&self) -> ProviderMode {
        self.fixture.mode()
    }
    fn endpoint_label(&self) -> Option<String> {
        self.fixture.endpoint_label()
    }
    async fn execute_page(&self, request: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        self.observations
            .provider_calls
            .fetch_add(1, Ordering::SeqCst);
        self.fixture.execute_page(request).await
    }
}

#[async_trait]
impl PageCache for ObservedCache {
    async fn get(&self, key: &str) -> Result<Option<CachedProviderPage>, ProviderError> {
        if self.missing.lock().await.as_deref() == Some(key) {
            return Ok(None);
        }
        let mut cached = self.inner.get(key).await?;
        if self.corrupt.lock().await.as_deref() == Some(key)
            && let Some(page) = &mut cached
        {
            page.page
                .diagnostics
                .insert("tampered".to_owned(), json!(true));
        }
        if self.wrong_key.lock().await.as_deref() == Some(key)
            && let Some(page) = &mut cached
        {
            "different-request-key".clone_into(&mut page.request_key);
        }
        Ok(cached)
    }

    async fn put(&self, key: &str, page: &CachedProviderPage) -> Result<(), ProviderError> {
        self.inner.put(key, page).await?;
        self.writes
            .lock()
            .await
            .push((key.to_owned(), page.clone()));
        Ok(())
    }
}

fn registry(
    cache: Arc<ObservedCache>,
    id: &str,
    pages: BTreeMap<Option<String>, ProviderPage>,
) -> Result<ProviderRegistry, ProviderError> {
    let mut runtime =
        ProviderRegistry::new().with_cache(cache.clone(), "synthetic-authority".to_owned())?;
    runtime.register(Arc::new(CountedFixture {
        fixture: FixtureProvider::new(id, "Synthetic runtime fixture", pages),
        observations: cache,
    }))?;
    Ok(runtime)
}

fn assert_receipt(
    result: &ExecutionResult,
    provider: &str,
    count: u64,
    pages: u32,
    mode: &str,
) -> TestResult {
    result.receipt.validate()?;
    assert_eq!(result.receipt.provider_id, provider);
    assert_eq!(result.receipt.records_retrieved, count);
    assert_eq!(result.receipt.pages_retrieved, pages);
    assert_eq!(result.receipt.execution_mode, mode);
    assert!(!result.receipt.policy.live_enabled);
    assert_eq!(result.receipt.endpoint, None);
    assert_eq!(
        result.receipt.result_digest,
        canonical_record_digest(&result.records)?
    );
    assert_ne!(result.receipt.receipt_id, "pending-receipt");
    for record in &result.records {
        record.validate()?;
        assert_eq!(record.source_receipt_id, result.receipt.receipt_id);
        assert_ne!(record.source_receipt_id, "pending-receipt");
    }
    Ok(())
}

// An independent test comparator for the documented canonical JSON cache envelope.
fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(fields) => {
            let ordered = fields
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(ordered.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        scalar => scalar.clone(),
    }
}

async fn prove_one_page(provider: &str, mut page: ProviderPage, raw: &str) -> TestResult {
    // Fixture scheduling ends this page explicitly; source cursor values are tested
    // separately by canonical parser goldens, not interpreted as available fixtures.
    page.next_cursor = None;
    let raw_digest = blake3::hash(raw.as_bytes()).to_hex().to_string();
    // JSON parsers do not produce raw digests. This is a fixture-envelope binding,
    // not evidence that the live transport propagates raw hashes into final receipts.
    page.diagnostics
        .insert("raw_response_digest".to_owned(), json!(raw_digest));
    let count = u64::try_from(page.records.len())?;
    let expected_records = page.records.clone();
    let cache = Arc::new(ObservedCache::default());
    let runtime = registry(
        cache.clone(),
        provider,
        BTreeMap::from([(None, page.clone())]),
    )?;
    let first = runtime
        .execute(provider, request(), "synthetic parsed page")
        .await?;
    assert_receipt(&first, provider, count, 1, "fixture")?;
    assert_eq!(first.receipt.cache_hits, 0);
    assert_eq!(first.receipt.cache_writes, 1);
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 1);
    let stored = cache.writes.lock().await;
    let (_, envelope) = stored.first().ok_or("missing cached page")?;
    assert_eq!(envelope.page, page);
    assert_eq!(
        envelope.page.diagnostics.get("raw_response_digest"),
        Some(&json!(raw_digest))
    );
    let expected_page_digest = blake3::hash(&serde_json::to_vec(&canonical(
        &serde_json::to_value(&page)?,
    ))?)
    .to_hex()
    .to_string();
    assert_eq!(envelope.response_digest, expected_page_digest);
    assert!(
        envelope
            .page
            .records
            .iter()
            .all(|record| record.source_receipt_id == "pending-receipt")
    );
    drop(stored);
    // An empty fixture provider would error if invoked: success proves cache-only replay.
    let replay_runtime = registry(cache.clone(), provider, BTreeMap::new())?;
    let replay = replay_runtime
        .execute(provider, request(), "synthetic parsed page")
        .await?;
    assert_receipt(&replay, provider, count, 1, "replay")?;
    assert_eq!(replay.receipt.cache_hits, 1);
    assert_eq!(replay.receipt.cache_writes, 0);
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 1);
    assert_ne!(replay.receipt.receipt_id, first.receipt.receipt_id);
    for result in [&first, &replay] {
        let mut normalised = result.records.clone();
        for record in &mut normalised {
            "pending-receipt".clone_into(&mut record.source_receipt_id);
        }
        assert_eq!(normalised, expected_records);
    }
    Ok(())
}

#[tokio::test]
async fn four_json_parsers_bind_real_runtime_receipts_and_replay() -> TestResult {
    let cases: [(&str, &str, Parser); 4] = [
        (
            "pubmed",
            include_str!("../../../provider-fixtures/mvp/pubmed-esummary.json"),
            parse_pubmed_summary_page,
        ),
        (
            "europe-pmc",
            include_str!("../../../provider-fixtures/mvp/europe-pmc.json"),
            parse_europe_pmc_page,
        ),
        (
            "crossref",
            include_str!("../../../provider-fixtures/mvp/crossref.json"),
            parse_crossref_page,
        ),
        (
            "openalex",
            include_str!("../../../provider-fixtures/mvp/openalex.json"),
            parse_openalex_page,
        ),
    ];
    for (provider, raw, parser) in cases {
        prove_one_page(provider, parser(&serde_json::from_str(raw)?)?, raw).await?;
    }
    Ok(())
}

#[tokio::test]
async fn efetch_parser_binds_real_runtime_receipt_and_replay() -> TestResult {
    let raw = include_str!("fixtures/pubmed-efetch.xml");
    let page = parse_pubmed_fetch_page(raw, &["123".to_owned()])?;
    assert_eq!(
        page.diagnostics.get("raw_response_digest"),
        Some(&json!(blake3::hash(raw.as_bytes()).to_hex().to_string()))
    );
    prove_one_page("pubmed-efetch-fixture", page, raw).await
}

fn two_pages() -> Result<BTreeMap<Option<String>, ProviderPage>, ProviderError> {
    let mut first = parse_openalex_page(
        &json!({"results":[{"id":"https://openalex.org/W1","title":"First synthetic"}]}),
    )?;
    let second = parse_openalex_page(
        &json!({"results":[{"id":"https://openalex.org/W2","title":"Second synthetic"}]}),
    )?;
    first.next_cursor = Some("second-page".to_owned());
    Ok(BTreeMap::from([
        (None, first),
        (Some("second-page".to_owned()), second),
    ]))
}

#[tokio::test]
async fn two_page_runtime_preserves_order_and_supports_mixed_cache_hit_miss() -> TestResult {
    let pages = two_pages()?;
    let cache = Arc::new(ObservedCache::default());
    let first = registry(cache.clone(), "openalex", pages.clone())?
        .execute("openalex", request(), "synthetic two-page")
        .await?;
    assert_receipt(&first, "openalex", 2, 2, "fixture")?;
    assert_eq!(first.receipt.cache_writes, 2);
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        first
            .records
            .iter()
            .map(|record| record.native_id.as_str())
            .collect::<Vec<_>>(),
        ["https://openalex.org/W1", "https://openalex.org/W2"]
    );
    let second_key = cache
        .writes
        .lock()
        .await
        .get(1)
        .ok_or("second cache write missing")?
        .0
        .clone();
    *cache.missing.lock().await = Some(second_key);
    let second_page = pages
        .get(&Some("second-page".to_owned()))
        .ok_or("second page missing")?
        .clone();
    let mixed = registry(
        cache.clone(),
        "openalex",
        BTreeMap::from([(Some("second-page".to_owned()), second_page)]),
    )?
    .execute("openalex", request(), "synthetic two-page")
    .await?;
    assert_receipt(&mixed, "openalex", 2, 2, "mixed-fixture-replay")?;
    assert_eq!(mixed.receipt.cache_hits, 1);
    assert_eq!(mixed.receipt.cache_writes, 1);
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 3);
    // Each run owns a fresh receipt; every other field and record order must
    // remain identical across the fixture and mixed-cache paths.
    let mut primed_records = first.records.clone();
    let mut mixed_records = mixed.records;
    for record in primed_records.iter_mut().chain(mixed_records.iter_mut()) {
        "pending-receipt".clone_into(&mut record.source_receipt_id);
    }
    assert_eq!(mixed_records, primed_records);
    Ok(())
}

#[tokio::test]
async fn modified_cached_page_is_rejected_without_fixture_fallback() -> TestResult {
    let cache = Arc::new(ObservedCache::default());
    registry(cache.clone(), "openalex", two_pages()?)?
        .execute("openalex", request(), "synthetic tamper")
        .await?;
    let key = cache
        .writes
        .lock()
        .await
        .first()
        .ok_or("cache write missing")?
        .0
        .clone();
    *cache.corrupt.lock().await = Some(key);
    let result = registry(cache.clone(), "openalex", BTreeMap::new())?
        .execute("openalex", request(), "synthetic tamper")
        .await;
    assert!(matches!(result, Err(ProviderError::Cache(_))));
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 2);
    assert_eq!(cache.writes.lock().await.len(), 2);
    Ok(())
}

#[tokio::test]
async fn mismatched_cached_request_key_is_rejected_without_fallback_or_overwrite() -> TestResult {
    let cache = Arc::new(ObservedCache::default());
    registry(cache.clone(), "openalex", two_pages()?)?
        .execute("openalex", request(), "synthetic key mismatch")
        .await?;
    let key = cache
        .writes
        .lock()
        .await
        .first()
        .ok_or("cache write missing")?
        .0
        .clone();
    *cache.wrong_key.lock().await = Some(key);
    let result = registry(cache.clone(), "openalex", BTreeMap::new())?
        .execute("openalex", request(), "synthetic key mismatch")
        .await;
    assert!(matches!(result, Err(ProviderError::Cache(_))));
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 2);
    assert_eq!(cache.writes.lock().await.len(), 2);
    Ok(())
}

#[tokio::test]
async fn page_and_record_budgets_are_visible_in_runtime_receipts() -> TestResult {
    let runtime = registry(Arc::new(ObservedCache::default()), "openalex", two_pages()?)?;
    let mut page_limited = request();
    page_limited.policy.max_pages = 1;
    let result = runtime
        .execute("openalex", page_limited, "synthetic page budget")
        .await?;
    assert_receipt(&result, "openalex", 1, 1, "fixture")?;
    assert!(
        result
            .receipt
            .warnings
            .iter()
            .any(|warning| warning.contains("max_pages"))
    );
    let mut record_limited = request();
    record_limited.policy.max_records = 1;
    let mut page = parse_openalex_page(
        &json!({"results":[{"id":"https://openalex.org/W1"},{"id":"https://openalex.org/W2"}]}),
    )?;
    page.next_cursor = None;
    let runtime = registry(
        Arc::new(ObservedCache::default()),
        "openalex",
        BTreeMap::from([(None, page)]),
    )?;
    let result = runtime
        .execute("openalex", record_limited, "synthetic record budget")
        .await?;
    assert_receipt(&result, "openalex", 1, 1, "fixture")?;
    assert!(
        result
            .receipt
            .warnings
            .iter()
            .any(|warning| warning.contains("max_records"))
    );
    Ok(())
}

#[tokio::test]
async fn exact_record_budget_with_continuation_is_explicitly_partial() -> TestResult {
    let cache = Arc::new(ObservedCache::default());
    let runtime = registry(cache.clone(), "openalex", two_pages()?)?;
    let mut limited = request();
    limited.policy.max_records = 1;
    let result = runtime
        .execute("openalex", limited, "synthetic exact-boundary budget")
        .await?;
    assert_receipt(&result, "openalex", 1, 1, "fixture")?;
    assert_eq!(cache.provider_calls.load(Ordering::SeqCst), 1);
    assert!(
        result
            .receipt
            .warnings
            .iter()
            .any(|warning| warning.contains("max_records")),
        "a continuation cursor at the exact record limit must not look exhaustive"
    );
    Ok(())
}
