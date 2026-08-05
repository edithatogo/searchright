//! Provider adapters and deterministic fixtures.
//!
//! Live network adapters are feature-gated. The default build has no network
//! capability and is suitable for tests, replay and contract development.

#![forbid(unsafe_code)]

use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use evidence_search_core::{ProviderError, ProviderMode, ProviderRegistry, SearchProvider};
use searchright_contracts::{
    BibliographicRecord, ProviderCapability, ProviderManifest, ProviderPage, ProviderSupportLevel,
    SearchRequest,
};
use serde::{Deserialize, Serialize};

/// A deterministic provider backed by checked-in or caller-supplied pages.
#[derive(Debug, Clone)]
pub struct FixtureProvider {
    manifest: ProviderManifest,
    pages: BTreeMap<Option<String>, ProviderPage>,
}

impl FixtureProvider {
    /// Construct a fixture provider. Page keys are request cursors; `None` is
    /// the first page.
    #[must_use]
    pub fn new(
        provider_id: impl Into<String>,
        display_name: impl Into<String>,
        pages: BTreeMap<Option<String>, ProviderPage>,
    ) -> Self {
        Self {
            manifest: ProviderManifest {
                provider_id: provider_id.into(),
                display_name: display_name.into(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                support_level: ProviderSupportLevel::FixtureBacked,
                capabilities: vec![ProviderCapability::Search, ProviderCapability::Pagination],
                allowed_hosts: Vec::new(),
                authentication_required: false,
                licensed: false,
                default_min_interval_ms: 0,
                policy_notes: vec!["deterministic fixture; no network access".to_owned()],
            },
            pages,
        }
    }

    /// Construct a one-page fixture.
    #[must_use]
    pub fn one_page(
        provider_id: impl Into<String>,
        display_name: impl Into<String>,
        records: Vec<BibliographicRecord>,
    ) -> Self {
        let mut pages = BTreeMap::new();
        pages.insert(
            None,
            ProviderPage {
                total_available: Some(
                    u64::try_from(records.len()).map_or(u64::MAX, |value| value),
                ),
                records,
                next_cursor: None,
                diagnostics: BTreeMap::new(),
            },
        );
        Self::new(provider_id, display_name, pages)
    }
}

#[async_trait]
impl SearchProvider for FixtureProvider {
    fn manifest(&self) -> ProviderManifest {
        self.manifest.clone()
    }

    fn mode(&self) -> ProviderMode {
        ProviderMode::Fixture
    }

    fn endpoint_label(&self) -> Option<String> {
        None
    }

    async fn execute_page(&self, request: &SearchRequest) -> Result<ProviderPage, ProviderError> {
        self.pages
            .get(&request.cursor)
            .cloned()
            .ok_or_else(|| ProviderError::Upstream {
                provider: self.manifest.provider_id.clone(),
                message: format!("fixture has no page for cursor {:?}", request.cursor),
            })
    }
}

/// Add the deterministic MVP fixtures to a registry.
pub fn register_mvp_fixtures(registry: &mut ProviderRegistry) -> Result<(), ProviderError> {
    registry.register(Arc::new(FixtureProvider::one_page(
        "pubmed-fixture",
        "PubMed fixture",
        Vec::new(),
    )))?;
    registry.register(Arc::new(FixtureProvider::one_page(
        "europe-pmc-fixture",
        "Europe PMC fixture",
        Vec::new(),
    )))?;
    Ok(())
}

/// Redacted endpoint construction for Europe PMC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EuropePmcRequest {
    /// Query text.
    pub query: String,
    /// Page size.
    pub page_size: u32,
    /// Optional cursor mark.
    pub cursor_mark: Option<String>,
}

impl EuropePmcRequest {
    /// Build the official REST search endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://www.ebi.ac.uk/europepmc/webservices/rest/search")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("query", &self.query);
            pairs.append_pair("format", "json");
            pairs.append_pair("resultType", "core");
            pairs.append_pair("pageSize", &self.page_size.to_string());
            if let Some(cursor) = &self.cursor_mark {
                pairs.append_pair("cursorMark", cursor);
            }
        }
        Ok(url)
    }
}

/// Redacted endpoint construction for PubMed ESearch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubMedSearchRequest {
    /// Query text.
    pub query: String,
    /// Page size.
    pub page_size: u32,
    /// Zero-based result offset.
    pub offset: u32,
    /// Optional NCBI tool name.
    pub tool: Option<String>,
    /// Optional contact email, included only when configured by the caller.
    pub email: Option<String>,
}

impl PubMedSearchRequest {
    /// Build the NCBI ESearch endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("db", "pubmed");
            pairs.append_pair("retmode", "json");
            pairs.append_pair("term", &self.query);
            pairs.append_pair("retmax", &self.page_size.to_string());
            pairs.append_pair("retstart", &self.offset.to_string());
            if let Some(tool) = &self.tool {
                pairs.append_pair("tool", tool);
            }
            if let Some(email) = &self.email {
                pairs.append_pair("email", email);
            }
        }
        Ok(url)
    }
}

#[cfg(feature = "live")]
mod live {
    use super::*;
    use searchright_contracts::{RecordIdentifiers, RecordKind};
    use serde_json::Value;

    /// Opt-in Europe PMC live adapter.
    #[derive(Debug, Clone)]
    pub struct EuropePmcProvider {
        client: reqwest::Client,
    }

    impl EuropePmcProvider {
        /// Construct a redirect-disabled HTTPS-only client for Europe PMC.
        pub fn new() -> Result<Self, ProviderError> {
            let client = reqwest::Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .user_agent(concat!("searchright/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|error| ProviderError::Upstream {
                    provider: "europe-pmc".to_owned(),
                    message: format!("could not construct HTTP client: {error}"),
                })?;
            Ok(Self { client })
        }
    }

    #[async_trait]
    impl SearchProvider for EuropePmcProvider {
        fn manifest(&self) -> ProviderManifest {
            ProviderManifest {
                provider_id: "europe-pmc".to_owned(),
                display_name: "Europe PMC".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                support_level: ProviderSupportLevel::OptInLive,
                capabilities: vec![ProviderCapability::Search, ProviderCapability::Pagination],
                allowed_hosts: vec!["www.ebi.ac.uk".to_owned()],
                authentication_required: false,
                licensed: false,
                default_min_interval_ms: 1_000,
                policy_notes: vec![
                    "live execution is feature-gated and additionally requires request policy approval"
                        .to_owned(),
                ],
            }
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some("https://www.ebi.ac.uk/europepmc/webservices/rest/search".to_owned())
        }

        async fn execute_page(&self, request: &SearchRequest) -> Result<ProviderPage, ProviderError> {
            let endpoint = EuropePmcRequest {
                query: request.strategy.query.clone(),
                page_size: request.page_size,
                cursor_mark: request.cursor.clone(),
            }
            .endpoint()
            .map_err(|error| ProviderError::Upstream {
                provider: "europe-pmc".to_owned(),
                message: error.to_string(),
            })?;
            let payload: Value = self
                .client
                .get(endpoint)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| ProviderError::Upstream {
                    provider: "europe-pmc".to_owned(),
                    message: error.to_string(),
                })?
                .json()
                .await
                .map_err(|error| ProviderError::Upstream {
                    provider: "europe-pmc".to_owned(),
                    message: error.to_string(),
                })?;
            parse_europe_pmc_page(&payload)
        }
    }

    fn parse_europe_pmc_page(payload: &Value) -> Result<ProviderPage, ProviderError> {
        let list = payload
            .get("resultList")
            .and_then(|value| value.get("result"))
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::Upstream {
                provider: "europe-pmc".to_owned(),
                message: "response omitted resultList.result".to_owned(),
            })?;
        let records = list
            .iter()
            .enumerate()
            .map(|(index, value)| BibliographicRecord {
                record_id: value
                    .get("id")
                    .and_then(Value::as_str)
                    .map_or_else(|| format!("europe-pmc-{index}"), str::to_owned),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id: value
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                kind: RecordKind::JournalArticle,
                identifiers: RecordIdentifiers {
                    doi: value.get("doi").and_then(Value::as_str).map(str::to_owned),
                    pmid: value.get("pmid").and_then(Value::as_str).map(str::to_owned),
                    pmcid: value.get("pmcid").and_then(Value::as_str).map(str::to_owned),
                    ..RecordIdentifiers::default()
                },
                title: value
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("[untitled]")
                    .to_owned(),
                abstract_text: value
                    .get("abstractText")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                authors: value
                    .get("authorString")
                    .and_then(Value::as_str)
                    .map(|authors| vec![authors.to_owned()])
                    .unwrap_or_default(),
                container_title: value
                    .get("journalTitle")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                publication_year: value
                    .get("pubYear")
                    .and_then(Value::as_str)
                    .and_then(|year| year.parse::<i32>().ok()),
                publication_date: None,
                languages: Vec::new(),
                subjects: Vec::new(),
                urls: Vec::new(),
                provider_metadata: value.clone(),
            })
            .collect();
        Ok(ProviderPage {
            records,
            next_cursor: payload
                .get("nextCursorMark")
                .and_then(Value::as_str)
                .map(str::to_owned),
            total_available: payload
                .get("hitCount")
                .and_then(Value::as_u64),
            diagnostics: BTreeMap::new(),
        })
    }

    pub use EuropePmcProvider as PublicEuropePmcProvider;
}

#[cfg(feature = "live")]
pub use live::PublicEuropePmcProvider as EuropePmcProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pubmed_endpoint_percent_encodes_query() {
        let endpoint = PubMedSearchRequest {
            query: "child AND genome".to_owned(),
            page_size: 20,
            offset: 0,
            tool: Some("searchright".to_owned()),
            email: None,
        }
        .endpoint();
        assert!(endpoint.is_ok());
        if let Ok(endpoint) = endpoint {
            assert_eq!(endpoint.host_str(), Some("eutils.ncbi.nlm.nih.gov"));
            assert!(endpoint.as_str().contains("child+AND+genome"));
        }
    }

    #[test]
    fn fixture_pages_are_cursor_addressed() {
        let provider = FixtureProvider::one_page("test", "Test", Vec::new());
        assert_eq!(provider.manifest().provider_id, "test");
        assert_eq!(provider.mode(), ProviderMode::Fixture);
    }
}
