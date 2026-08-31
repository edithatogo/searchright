//! No network: live-shaped adapters replay only synthetic, version-bound cache entries.
#![cfg(feature = "live")]

use async_trait::async_trait;
use evidence_search_contracts::{
    CompiledStrategy, ExecutionPolicy, ProviderManifest, ProviderPage, SearchDialect,
    SearchRequest, TranslationFidelity,
};
use evidence_search_core::{
    MemoryPageCache, ProviderError, ProviderMode, ProviderRegistry, SearchProvider,
};
use searchright_connectors::{
    CrossrefProvider, EuropePmcProvider, FixtureProvider, OpenAlexProvider,
    PROVIDER_PARSER_VERSION, PubMedProvider, parse_openalex_page,
};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request() -> SearchRequest {
    SearchRequest {
        review_id: "synthetic-review".into(),
        run_id: "synthetic-run".into(),
        strategy: CompiledStrategy {
            schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.into(),
            strategy_id: "synthetic-strategy".into(),
            dialect: SearchDialect::GenericBoolean,
            query: "synthetic".into(),
            warnings: Vec::new(),
            fidelity: TranslationFidelity::Exact,
            review_required: false,
            loss_codes: Vec::new(),
            compilation_hash: "synthetic".into(),
            compiler_version: "fixture".into(),
        },
        cursor: None,
        page_size: 1,
        policy: ExecutionPolicy {
            live_enabled: false,
            max_records: 1,
            max_pages: 1,
            timeout_seconds: 5,
            total_timeout_seconds: Some(10),
            max_retries: 0,
            min_interval_ms: 0,
            retry_base_delay_ms: Some(100),
            retry_max_delay_ms: Some(1_000),
            max_response_bytes: Some(10_000),
            replay_enabled: true,
            cache_write_enabled: true,
        },
    }
}

struct SyntheticLive {
    manifest: ProviderManifest,
    endpoint: Option<String>,
    page: ProviderPage,
}

#[async_trait]
impl SearchProvider for SyntheticLive {
    fn manifest(&self) -> ProviderManifest {
        self.manifest.clone()
    }
    fn mode(&self) -> ProviderMode {
        ProviderMode::Live
    }
    fn endpoint_label(&self) -> Option<String> {
        self.endpoint.clone()
    }
    async fn execute_page(&self, _: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        Ok(self.page.clone())
    }
}

fn registry(
    cache: Arc<MemoryPageCache>,
    provider: Arc<dyn SearchProvider>,
) -> Result<ProviderRegistry, ProviderError> {
    let mut registry = ProviderRegistry::new().with_cache(cache, "synthetic-authority")?;
    registry.register(provider)?;
    Ok(registry)
}

#[tokio::test]
async fn old_parser_cache_cannot_bypass_current_live_adapter_but_is_preserved() -> TestResult {
    let providers: Vec<Arc<dyn SearchProvider>> = vec![
        Arc::new(PubMedProvider::new(None, None)?),
        Arc::new(EuropePmcProvider::new()?),
        Arc::new(CrossrefProvider::new(None)?),
        Arc::new(OpenAlexProvider::new(None)?),
    ];
    for provider in providers {
        let cache = Arc::new(MemoryPageCache::new());
        let current_manifest = provider.manifest();
        assert_eq!(current_manifest.version, PROVIDER_PARSER_VERSION);
        assert_ne!(current_manifest.version, env!("CARGO_PKG_VERSION"));
        let id = current_manifest.provider_id.clone();
        let mut old_manifest = current_manifest.clone();
        old_manifest.version = env!("CARGO_PKG_VERSION").into();
        let mut old_page = parse_openalex_page(
            &json!({"results":[{"id":"https://openalex.org/W1","title":"Synthetic historical payload"}]}),
        )?;
        old_page
            .records
            .first_mut()
            .ok_or("missing synthetic record")?
            .record_id = "historical-positional-id".into();
        old_page.diagnostics = BTreeMap::new();
        let old_stub = Arc::new(SyntheticLive {
            manifest: old_manifest,
            endpoint: provider.endpoint_label(),
            page: old_page,
        });
        let mut seed = request();
        seed.policy.live_enabled = true; // Only the in-memory stub can execute.
        registry(cache.clone(), old_stub.clone())?
            .execute(&id, seed.clone(), "synthetic")
            .await?;
        let current = registry(cache.clone(), provider.clone())?;
        let miss = current.execute(&id, request(), "synthetic").await;
        assert!(
            matches!(miss, Err(ProviderError::LiveDisabled(_))),
            "old normalized pages must not replay through the new adapter: {id}"
        );
        let retained = registry(cache.clone(), old_stub)?
            .execute(&id, request(), "synthetic")
            .await?;
        assert_eq!(retained.receipt.execution_mode, "replay");
        assert_eq!(
            retained
                .records
                .first()
                .ok_or("old record missing")?
                .record_id,
            "historical-positional-id"
        );
        let current_page = parse_openalex_page(
            &json!({"results":[{"id":"https://openalex.org/W2","title":"Synthetic current payload"}]}),
        )?;
        let expected_id = current_page
            .records
            .first()
            .ok_or("current record missing")?
            .record_id
            .clone();
        let current_stub = Arc::new(SyntheticLive {
            manifest: current_manifest.clone(),
            endpoint: provider.endpoint_label(),
            page: current_page,
        });
        registry(cache.clone(), current_stub)?
            .execute(&id, seed, "synthetic")
            .await?;
        let replay = current.execute(&id, request(), "synthetic").await?;
        assert_eq!(replay.receipt.execution_mode, "replay");
        assert_eq!(replay.receipt.provider_version, current_manifest.version);
        assert_eq!(
            replay
                .records
                .first()
                .ok_or("new record missing")?
                .record_id,
            expected_id
        );
        assert_eq!(cache.len().await, 2);
    }
    Ok(())
}

#[test]
fn fixture_version_is_explicit_and_legacy_constructor_does_not_claim_parser_provenance() {
    let legacy = FixtureProvider::new("fixture", "Synthetic", BTreeMap::new());
    assert_eq!(legacy.manifest().version, env!("CARGO_PKG_VERSION"));
    let current = legacy.with_version(PROVIDER_PARSER_VERSION);
    assert_eq!(current.manifest().version, PROVIDER_PARSER_VERSION);
}
