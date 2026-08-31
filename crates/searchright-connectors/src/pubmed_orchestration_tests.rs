use super::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::Mutex;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct ScriptedBytes {
    replies: Mutex<VecDeque<Vec<u8>>>,
    calls: std::sync::Mutex<Vec<(String, tokio::time::Instant)>>,
    factories: AtomicUsize,
    fail_second: AtomicBool,
}

impl ByteTransport for ScriptedBytes {
    fn fetch<'a>(
        &'a self,
        provider: &'a str,
        endpoint: url::Url,
        _: &'a SearchRequest,
        _: u64,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<u8>, ProviderError>> + Send + 'a>> {
        let number = self.factories.fetch_add(1, Ordering::SeqCst) + 1;
        match self.calls.lock() {
            Ok(mut calls) => calls.push((endpoint.to_string(), tokio::time::Instant::now())),
            Err(_) => {
                return Box::pin(async {
                    Err(ProviderError::InvalidRequest(
                        "poisoned scripted calls".into(),
                    ))
                });
            }
        }
        Box::pin(async move {
            if number == 2 && self.fail_second.load(Ordering::SeqCst) {
                return Err(ProviderError::RateLimited {
                    provider: provider.to_owned(),
                    retry_after_ms: Some(1000),
                });
            }
            let bytes = self.replies.lock().await.pop_front().ok_or_else(|| {
                ProviderError::InvalidRequest("unexpected scripted request".into())
            })?;
            // Deliberately bypass transport accumulation: orchestration must
            // independently reject oversized bytes from this injected boundary.
            Ok(bytes)
        })
    }
}

struct ScriptedPubmed {
    adapter: PubMedProvider,
    transport: Arc<ScriptedBytes>,
    efetch: bool,
    version: Option<String>,
    observed: Mutex<Option<ProviderPage>>,
}

#[async_trait]
impl SearchProvider for ScriptedPubmed {
    fn manifest(&self) -> ProviderManifest {
        let mut manifest = if self.efetch {
            PubMedEfetchProvider {
                inner: self.adapter.clone(),
            }
            .manifest()
        } else {
            self.adapter.manifest()
        };
        if let Some(version) = &self.version {
            manifest.version.clone_from(version);
        }
        manifest
    }
    fn mode(&self) -> ProviderMode {
        ProviderMode::Live
    }
    fn endpoint_label(&self) -> Option<String> {
        self.adapter.endpoint_label()
    }
    async fn execute_page(&self, _: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        Err(ProviderError::InvalidRequest("context required".into()))
    }
    async fn execute_page_with_context(
        &self,
        request: &SearchRequest,
        context: &mut PageExecutionContext<'_>,
    ) -> Result<ProviderPage, ProviderError> {
        let page = self
            .adapter
            .execute_with(request, context, self.efetch, self.transport.as_ref())
            .await?;
        *self.observed.lock().await = Some(page.clone());
        Ok(page)
    }
}

fn request() -> SearchRequest {
    SearchRequest {
        review_id: "synthetic".into(),
        run_id: "synthetic".into(),
        strategy: evidence_search_contracts::CompiledStrategy {
            schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.into(),
            strategy_id: "synthetic".into(),
            dialect: evidence_search_contracts::SearchDialect::PubMed,
            query: "secret-query".into(),
            warnings: Vec::new(),
            fidelity: evidence_search_contracts::TranslationFidelity::Exact,
            review_required: false,
            loss_codes: Vec::new(),
            compilation_hash: "synthetic".into(),
            compiler_version: "test".into(),
        },
        cursor: None,
        page_size: 1,
        policy: evidence_search_contracts::ExecutionPolicy {
            live_enabled: true,
            max_records: 1,
            max_pages: 1,
            timeout_seconds: 10,
            total_timeout_seconds: Some(20),
            max_retries: 0,
            min_interval_ms: 1000,
            retry_base_delay_ms: Some(100),
            retry_max_delay_ms: Some(1000),
            max_response_bytes: Some(8_388_608),
            replay_enabled: false,
            cache_write_enabled: false,
        },
    }
}

fn search(count: u64, ids: &[&str]) -> Vec<u8> {
    serde_json::json!({"esearchresult":{"count":count.to_string(),"idlist":ids}})
        .to_string()
        .into_bytes()
}

fn xml() -> Vec<u8> {
    b"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article><ArticleTitle>Synthetic</ArticleTitle><Abstract><AbstractText Label=\"METHODS\">Offline.</AbstractText></Abstract></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>".to_vec()
}

fn adapter(efetch: bool, replies: Vec<Vec<u8>>) -> Result<Arc<ScriptedPubmed>, ProviderError> {
    Ok(Arc::new(ScriptedPubmed {
        adapter: PubMedProvider::new(None, None)?,
        efetch,
        version: None,
        transport: Arc::new(ScriptedBytes {
            replies: Mutex::new(replies.into()),
            calls: std::sync::Mutex::new(Vec::new()),
            factories: AtomicUsize::new(0),
            fail_second: AtomicBool::new(false),
        }),
        observed: Mutex::new(None),
    }))
}

#[tokio::test(start_paused = true)]
async fn empty_result_and_lower_policy_limit_are_enforced_offline() -> TestResult {
    let provider = adapter(true, vec![search(0, &[])])?;
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone())?;
    let result = registry
        .execute("pubmed-efetch", request(), "synthetic")
        .await?;
    assert!(result.records.is_empty());
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 1);
    let provider = adapter(true, vec![search(1, &["1"]), xml()])?;
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone())?;
    let mut bounded = request();
    bounded.policy.max_response_bytes = Some(100);
    assert!(matches!(
        registry
            .execute("pubmed-efetch", bounded, "synthetic")
            .await,
        Err(ProviderError::BudgetExceeded {
            kind: "response_bytes",
            limit: 100
        })
    ));
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn actual_orchestration_cache_partitions_ids_and_previous_behavior_versions() -> TestResult {
    let cache = Arc::new(evidence_search_core::MemoryPageCache::new());
    let summary_bytes =
        serde_json::json!({"result":{"uids":["1"],"1":{"uid":"1","title":"Synthetic"}}})
            .to_string()
            .into_bytes();
    let mut old = adapter(false, vec![search(1, &["1"]), summary_bytes.clone()])?;
    Arc::get_mut(&mut old)
        .ok_or("unexpected shared stub")?
        .version = Some(super::super::PROVIDER_PARSER_VERSION.into());
    let mut old_registry =
        ProviderRegistry::new().with_cache(cache.clone(), "synthetic-authority")?;
    old_registry.register(old.clone())?;
    let mut seed = request();
    seed.policy.replay_enabled = true;
    seed.policy.cache_write_enabled = true;
    old_registry
        .execute("pubmed", seed.clone(), "synthetic")
        .await?;

    let summary = adapter(false, vec![search(1, &["1"]), summary_bytes])?;
    let efetch = adapter(true, vec![search(1, &["1"]), xml()])?;
    let mut registry = ProviderRegistry::new().with_cache(cache.clone(), "synthetic-authority")?;
    registry.register_with_rate_group(summary.clone(), NCBI_RATE_GROUP)?;
    registry.register_with_rate_group(efetch.clone(), NCBI_RATE_GROUP)?;
    let mut replay = seed.clone();
    replay.policy.live_enabled = false;
    assert!(matches!(
        registry
            .execute("pubmed", replay.clone(), "synthetic")
            .await,
        Err(ProviderError::LiveDisabled(_))
    ));
    assert_eq!(summary.transport.factories.load(Ordering::SeqCst), 0);
    registry
        .execute("pubmed", seed.clone(), "synthetic")
        .await?;
    assert!(matches!(
        registry
            .execute("pubmed-efetch", replay.clone(), "synthetic")
            .await,
        Err(ProviderError::LiveDisabled(_))
    ));
    assert_eq!(efetch.transport.factories.load(Ordering::SeqCst), 0);
    let first = registry.execute("pubmed-efetch", seed, "synthetic").await?;
    let second = registry
        .execute("pubmed-efetch", replay.clone(), "synthetic")
        .await?;
    assert_eq!(first.receipt.execution_mode, "live"); // Synthetic live-shaped byte boundary only.
    assert_eq!(second.receipt.execution_mode, "replay");
    assert_eq!(second.receipt.provider_id, "pubmed-efetch");
    assert_eq!(
        second.receipt.provider_version,
        format!("{}.subrequests.1", super::super::PROVIDER_PARSER_VERSION)
    );
    assert_eq!(second.receipt.cache_hits, 1);
    assert_eq!(second.receipt.cache_writes, 0);
    assert_eq!(efetch.transport.factories.load(Ordering::SeqCst), 2);
    let mut first_records = first.records;
    let mut second_records = second.records;
    for record in first_records.iter_mut().chain(second_records.iter_mut()) {
        record.source_receipt_id.clear();
    }
    assert_eq!(first_records, second_records);
    assert_eq!(
        second_records.first().ok_or("missing record")?.record_id,
        "pubmed-1"
    );
    let retained = old_registry.execute("pubmed", replay, "synthetic").await?;
    assert_eq!(retained.receipt.execution_mode, "replay");
    assert_eq!(
        retained.receipt.provider_version,
        super::super::PROVIDER_PARSER_VERSION
    );
    assert_eq!(old.transport.factories.load(Ordering::SeqCst), 2);
    assert_eq!(cache.len().await, 3);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn efetch_uses_two_admitted_requests_and_retains_both_raw_hashes() -> TestResult {
    let first = search(2, &["1"]);
    let second = xml();
    let hashes = vec![
        blake3::hash(&first).to_hex().to_string(),
        blake3::hash(&second).to_hex().to_string(),
    ];
    let provider = adapter(true, vec![first, second])?;
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(provider.clone(), NCBI_RATE_GROUP)?;
    let result = registry
        .execute("pubmed-efetch", request(), "synthetic")
        .await?;
    assert_eq!(
        result.records.first().ok_or("missing record")?.record_id,
        "pubmed-1"
    );
    let calls = provider
        .transport
        .calls
        .lock()
        .map_err(|_| "poisoned calls")?
        .clone();
    let [first, second] = calls.as_slice() else {
        return Err("expected two calls".into());
    };
    assert!(first.0.contains("/esearch.fcgi?"));
    assert!(second.0.contains("/efetch.fcgi?"));
    assert!(!second.0.contains("esummary"));
    let search_url = url::Url::parse(&first.0)?;
    let fetch_url = url::Url::parse(&second.0)?;
    let search_pairs = search_url
        .query_pairs()
        .into_owned()
        .collect::<BTreeMap<_, _>>();
    let fetch_pairs = fetch_url
        .query_pairs()
        .into_owned()
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        search_pairs,
        BTreeMap::from([
            ("db".into(), "pubmed".into()),
            ("term".into(), "secret-query".into()),
            ("retmode".into(), "json".into()),
            ("retstart".into(), "0".into()),
            ("retmax".into(), "1".into()),
        ])
    );
    assert_eq!(
        fetch_pairs,
        BTreeMap::from([
            ("db".into(), "pubmed".into()),
            ("id".into(), "1".into()),
            ("retmode".into(), "xml".into()),
        ])
    );
    assert!(second.1.duration_since(first.1) >= std::time::Duration::from_secs(1));
    let page = provider
        .observed
        .lock()
        .await
        .clone()
        .ok_or("missing page")?;
    assert_eq!(page.next_cursor.as_deref(), Some("1"));
    assert_eq!(page.total_available, Some(2));
    assert_eq!(
        page.diagnostics.get("raw_response_digests"),
        Some(&serde_json::json!(hashes))
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn invalid_efetch_never_falls_back_to_summary() -> TestResult {
    for body in [
        vec![0xff],
        b"<!DOCTYPE x><PubmedArticleSet/>".to_vec(),
        String::from_utf8(xml())?
            .replace("<PMID>1</PMID>", "<PMID>2</PMID>")
            .into_bytes(),
        b"<PubmedArticleSet><PubmedBookArticle/></PubmedArticleSet>".to_vec(),
        vec![b'x'; 8 * 1024 * 1024 + 1],
    ] {
        let provider = adapter(true, vec![search(1, &["1"]), body])?;
        let mut registry = ProviderRegistry::new();
        registry.register(provider.clone())?;
        let error = registry
            .execute("pubmed-efetch", request(), "synthetic")
            .await
            .err()
            .ok_or("expected rejection")?;
        assert!(!error.to_string().contains("secret-query"));
        assert_eq!(
            provider
                .transport
                .calls
                .lock()
                .map_err(|_| "poisoned calls")?
                .len(),
            2
        );
        assert!(provider.observed.lock().await.is_none());
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn invalid_search_or_deadline_prevents_second_request() -> TestResult {
    for first in [search(1, &[]), search(1, &["1", "1"]), search(1, &["bad"])] {
        let provider = adapter(true, vec![first, xml()])?;
        let mut registry = ProviderRegistry::new();
        registry.register(provider.clone())?;
        assert!(
            registry
                .execute("pubmed-efetch", request(), "synthetic")
                .await
                .is_err()
        );
        assert_eq!(
            provider
                .transport
                .calls
                .lock()
                .map_err(|_| "poisoned calls")?
                .len(),
            1
        );
    }
    let provider = adapter(true, vec![search(1, &["1"]), xml()])?;
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone())?;
    let mut short = request();
    short.policy.timeout_seconds = 1;
    short.policy.min_interval_ms = 2000;
    assert!(
        registry
            .execute("pubmed-efetch", short, "synthetic")
            .await
            .is_err()
    );
    assert_eq!(
        provider
            .transport
            .calls
            .lock()
            .map_err(|_| "poisoned calls")?
            .len(),
        1
    );
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn second_request_rate_limit_uses_core_page_retry_and_readmits_search() -> TestResult {
    let provider = adapter(true, vec![search(1, &["1"]), search(1, &["1"]), xml()])?;
    provider.transport.fail_second.store(true, Ordering::SeqCst);
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone())?;
    let mut retry = request();
    retry.policy.max_retries = 1;
    registry
        .execute("pubmed-efetch", retry, "synthetic")
        .await?;
    let calls = provider
        .transport
        .calls
        .lock()
        .map_err(|_| "poisoned calls")?
        .clone();
    assert_eq!(calls.len(), 4);
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 4);
    let paths = calls
        .iter()
        .map(|call| url::Url::parse(&call.0).map(|url| url.path().to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        paths,
        [
            "/entrez/eutils/esearch.fcgi",
            "/entrez/eutils/efetch.fcgi",
            "/entrez/eutils/esearch.fcgi",
            "/entrez/eutils/efetch.fcgi"
        ]
    );
    for pair in calls.windows(2) {
        let [first, second] = pair else {
            return Err("invalid pair".into());
        };
        assert!(second.1.duration_since(first.1) >= std::time::Duration::from_secs(1));
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_before_second_admission_never_constructs_its_future() -> TestResult {
    let provider = adapter(true, vec![search(1, &["1"]), xml()])?;
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone())?;
    let task = tokio::spawn(async move {
        registry
            .execute("pubmed-efetch", request(), "synthetic")
            .await
    });
    tokio::task::yield_now().await;
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 1);
    task.abort();
    assert!(task.await.is_err());
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    assert_eq!(provider.transport.factories.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn summary_and_efetch_share_admission_group() -> TestResult {
    let summary = adapter(
        false,
        vec![
            search(1, &["1"]),
            serde_json::json!({"result":{"uids":["1"],"1":{"uid":"1","title":"Synthetic"}}})
                .to_string()
                .into_bytes(),
        ],
    )?;
    let efetch = adapter(true, vec![search(1, &["1"]), xml()])?;
    let mut registry = ProviderRegistry::new();
    registry.register_with_rate_group(summary.clone(), NCBI_RATE_GROUP)?;
    registry.register_with_rate_group(efetch.clone(), NCBI_RATE_GROUP)?;
    let (a, b) = tokio::join!(
        registry.execute("pubmed", request(), "synthetic"),
        registry.execute("pubmed-efetch", request(), "synthetic")
    );
    a?;
    b?;
    let mut calls = summary
        .transport
        .calls
        .lock()
        .map_err(|_| "poisoned calls")?
        .clone();
    assert!(calls.iter().any(|call| call.0.contains("/esummary.fcgi?")));
    calls.extend(
        efetch
            .transport
            .calls
            .lock()
            .map_err(|_| "poisoned calls")?
            .clone(),
    );
    calls.sort_by_key(|call| call.1);
    assert_eq!(calls.len(), 4);
    for pair in calls.windows(2) {
        let [first, second] = pair else {
            return Err("invalid pair".into());
        };
        assert!(second.1.duration_since(first.1) >= std::time::Duration::from_secs(1));
    }
    Ok(())
}
