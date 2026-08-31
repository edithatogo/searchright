//! Test-first acceptance for explicit `EFetch` registration. No network is requested.
#![cfg(feature = "live")]

use evidence_search_contracts::{
    CompiledStrategy, ExecutionPolicy, SearchDialect, SearchRequest, TranslationFidelity,
};
use evidence_search_core::{ProviderMode, ProviderRegistry, SearchProvider};
use searchright_connectors::{
    LiveProviderConfig, PubMedEfetchProvider, PubMedProvider, register_mvp_live_providers,
    register_pubmed_efetch_provider,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn efetch_identity_is_distinct_from_unchanged_summary_provider() -> TestResult {
    let summary = PubMedProvider::new(None, None)?;
    let efetch = PubMedEfetchProvider::new(None, None)?;
    assert_eq!(summary.manifest().provider_id, "pubmed");
    assert_eq!(efetch.manifest().provider_id, "pubmed-efetch");
    assert_eq!(efetch.mode(), ProviderMode::Live);
    assert_eq!(efetch.manifest().allowed_hosts, ["eutils.ncbi.nlm.nih.gov"]);
    Ok(())
}

#[test]
fn default_live_registration_does_not_enable_efetch() -> TestResult {
    let mut registry = ProviderRegistry::new();
    register_mvp_live_providers(&mut registry, LiveProviderConfig::default())?;
    let ids = registry
        .manifests()
        .into_iter()
        .map(|manifest| manifest.provider_id)
        .collect::<Vec<_>>();
    assert!(ids.iter().any(|id| id == "pubmed"));
    assert!(!ids.iter().any(|id| id == "pubmed-efetch"));
    assert_eq!(ids.len(), 4);
    register_pubmed_efetch_provider(&mut registry, None, None)?;
    assert_eq!(registry.manifests().len(), 5);
    Ok(())
}

#[tokio::test]
async fn direct_unscoped_efetch_is_denied_before_network() -> TestResult {
    let mut request = SearchRequest {
        review_id: "synthetic-review".to_owned(),
        run_id: "synthetic-run".to_owned(),
        strategy: CompiledStrategy {
            schema_version: evidence_search_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.to_owned(),
            strategy_id: "synthetic-strategy".to_owned(),
            dialect: SearchDialect::PubMed,
            query: "synthetic".to_owned(),
            warnings: Vec::new(),
            fidelity: TranslationFidelity::Exact,
            review_required: false,
            loss_codes: Vec::new(),
            compilation_hash: "synthetic".to_owned(),
            compiler_version: "test".to_owned(),
        },
        cursor: None,
        page_size: 1,
        policy: ExecutionPolicy {
            live_enabled: false,
            max_records: 1,
            max_pages: 1,
            timeout_seconds: 1,
            total_timeout_seconds: Some(1),
            max_retries: 0,
            min_interval_ms: 1_000,
            retry_base_delay_ms: Some(100),
            retry_max_delay_ms: Some(1_000),
            max_response_bytes: Some(8 * 1024 * 1024),
            replay_enabled: false,
            cache_write_enabled: false,
        },
    };
    let provider = PubMedEfetchProvider::new(None, None)?;
    assert!(provider.execute_page(&request).await.is_err());
    request.policy.live_enabled = true;
    assert!(provider.execute_page(&request).await.is_err());
    assert!(
        PubMedProvider::new(None, None)?
            .execute_page(&request)
            .await
            .is_err()
    );
    Ok(())
}
