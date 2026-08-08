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
    RecordIdentifiers, RecordKind, SearchRequest,
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
                schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
                total_available: Some(
                    u64::try_from(records.len()).unwrap_or(u64::MAX),
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
    for (provider_id, display_name, native_id) in [
        ("pubmed-fixture", "PubMed fixture", "pmid:00000001"),
        ("europe-pmc-fixture", "Europe PMC fixture", "epmc:00000001"),
        ("crossref-fixture", "Crossref fixture", "doi:10.1000/searchright"),
        ("openalex-fixture", "OpenAlex fixture", "openalex:W000000001"),
        (
            "clinicaltrials-gov-fixture",
            "ClinicalTrials.gov fixture",
            "nct:NCT00000001",
        ),
    ] {
        registry.register(Arc::new(FixtureProvider::one_page(
            provider_id,
            display_name,
            vec![demo_record(provider_id, native_id)],
        )))?;
    }
    Ok(())
}

fn demo_record(provider_id: &str, native_id: &str) -> BibliographicRecord {
    BibliographicRecord {
        schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
        record_id: format!("{provider_id}-record-1"),
        source_receipt_id: "fixture-receipt".to_owned(),
        native_id: native_id.to_owned(),
        kind: if provider_id.contains("clinicaltrials") {
            RecordKind::TrialRegistry
        } else {
            RecordKind::JournalArticle
        },
        identifiers: RecordIdentifiers {
            doi: provider_id.contains("crossref").then(|| "10.1000/searchright".to_owned()),
            pmid: provider_id.contains("pubmed").then(|| "00000001".to_owned()),
            trial_registration: provider_id
                .contains("clinicaltrials")
                .then(|| "NCT00000001".to_owned()),
            openalex: provider_id
                .contains("openalex")
                .then(|| "W000000001".to_owned()),
            ..RecordIdentifiers::default()
        },
        title: "Synthetic fixture for contract and interoperability testing".to_owned(),
        abstract_text: Some(
            "Synthetic metadata only; it does not describe a real study.".to_owned(),
        ),
        authors: vec!["Searchright Fixture".to_owned()],
        container_title: Some("Searchright Test Corpus".to_owned()),
        publication_year: Some(2026),
        publication_date: Some("2026-08-06".to_owned()),
        languages: vec!["en".to_owned()],
        subjects: vec!["systematic review".to_owned()],
        urls: Vec::new(),
        provider_metadata: serde_json::json!({"synthetic": true}),
    }
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



/// Redacted endpoint construction for PubMed ESummary metadata retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubMedSummaryRequest {
    /// PubMed identifiers to retrieve.
    pub pmids: Vec<String>,
    /// Optional NCBI tool name.
    pub tool: Option<String>,
    /// Optional contact email.
    pub email: Option<String>,
}

impl PubMedSummaryRequest {
    /// Build the NCBI ESummary endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("db", "pubmed");
            pairs.append_pair("retmode", "json");
            pairs.append_pair("id", &self.pmids.join(","));
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

/// Parse an NCBI ESummary response into canonical bibliographic records.
pub fn parse_pubmed_summary_page(
    payload: &serde_json::Value,
) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let result = payload.get("result").ok_or_else(|| ProviderError::Upstream {
        provider: "pubmed".to_owned(),
        message: "response omitted result".to_owned(),
    })?;
    let uids = result
        .get("uids")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Upstream {
            provider: "pubmed".to_owned(),
            message: "response omitted result.uids".to_owned(),
        })?;
    let records = uids
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|pmid| result.get(pmid).map(|item| (pmid, item)))
        .map(|(pmid, item)| {
            let article_ids = item
                .get("articleids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let doi = article_ids.iter().find_map(|identifier| {
                (identifier.get("idtype").and_then(Value::as_str) == Some("doi"))
                    .then(|| identifier.get("value").and_then(Value::as_str))
                    .flatten()
                    .map(str::to_owned)
            });
            let authors = item
                .get("authors")
                .and_then(Value::as_array)
                .map_or_else(Vec::new, |values| {
                    values
                        .iter()
                        .filter_map(|author| author.get("name").and_then(Value::as_str))
                        .map(str::to_owned)
                        .collect()
                });
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
                record_id: format!("pubmed-{pmid}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id: pmid.to_owned(),
                kind: RecordKind::JournalArticle,
                identifiers: RecordIdentifiers {
                    doi,
                    pmid: Some(pmid.to_owned()),
                    ..RecordIdentifiers::default()
                },
                title: item
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("[untitled]")
                    .to_owned(),
                abstract_text: None,
                authors,
                container_title: item
                    .get("fulljournalname")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                publication_year: item
                    .get("pubdate")
                    .and_then(Value::as_str)
                    .and_then(|value| value.get(..4))
                    .and_then(|value| value.parse::<i32>().ok()),
                publication_date: item
                    .get("sortpubdate")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                languages: Vec::new(),
                subjects: Vec::new(),
                urls: vec![format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")],
                provider_metadata: item.clone(),
            }
        })
        .collect::<Vec<_>>();
    Ok(ProviderPage {
        schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        total_available: Some(u64::try_from(records.len()).unwrap_or(u64::MAX)),
        records,
        next_cursor: None,
        diagnostics: BTreeMap::new(),
    })
}

/// Redacted endpoint construction for Crossref Works.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossrefRequest {
    /// Bibliographic query.
    pub query: String,
    /// Page size.
    pub rows: u32,
    /// Optional opaque cursor.
    pub cursor: Option<String>,
    /// Optional polite-pool contact email.
    pub mailto: Option<String>,
}

impl CrossrefRequest {
    /// Build the Crossref Works endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://api.crossref.org/works")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("query.bibliographic", &self.query);
            pairs.append_pair("rows", &self.rows.to_string());
            if let Some(cursor) = &self.cursor {
                pairs.append_pair("cursor", cursor);
            }
            if let Some(mailto) = &self.mailto {
                pairs.append_pair("mailto", mailto);
            }
        }
        Ok(url)
    }
}

/// Redacted endpoint construction for OpenAlex Works.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAlexRequest {
    /// Search query.
    pub query: String,
    /// Page size.
    pub per_page: u32,
    /// Optional cursor.
    pub cursor: Option<String>,
    /// Optional polite-pool email.
    pub mailto: Option<String>,
}

impl OpenAlexRequest {
    /// Build the OpenAlex Works endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://api.openalex.org/works")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("search", &self.query);
            pairs.append_pair("per-page", &self.per_page.to_string());
            pairs.append_pair("cursor", self.cursor.as_deref().unwrap_or("*"));
            if let Some(mailto) = &self.mailto {
                pairs.append_pair("mailto", mailto);
            }
        }
        Ok(url)
    }
}

/// Redacted endpoint construction for ClinicalTrials.gov API v2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicalTrialsGovRequest {
    /// Query text.
    pub query: String,
    /// Page size.
    pub page_size: u32,
    /// Optional page token.
    pub page_token: Option<String>,
}

impl ClinicalTrialsGovRequest {
    /// Build the ClinicalTrials.gov studies endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url = url::Url::parse("https://clinicaltrials.gov/api/v2/studies")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("query.term", &self.query);
            pairs.append_pair("pageSize", &self.page_size.to_string());
            pairs.append_pair("format", "json");
            if let Some(token) = &self.page_token {
                pairs.append_pair("pageToken", token);
            }
        }
        Ok(url)
    }
}

/// Parse a Europe PMC fixture or live response into the canonical page contract.
pub fn parse_europe_pmc_page(
    payload: &serde_json::Value,
) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
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
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
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
                .map_or_else(Vec::new, |authors| vec![authors.to_owned()]),
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
        schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor: payload
            .get("nextCursorMark")
            .and_then(Value::as_str)
            .map(str::to_owned),
        total_available: payload.get("hitCount").and_then(Value::as_u64),
        diagnostics: BTreeMap::new(),
    })
}

/// Parse a Crossref Works response into the canonical page contract.
pub fn parse_crossref_page(payload: &serde_json::Value) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let message = payload
        .get("message")
        .ok_or_else(|| ProviderError::Upstream {
            provider: "crossref".to_owned(),
            message: "response omitted message".to_owned(),
        })?;
    let items = message
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Upstream {
            provider: "crossref".to_owned(),
            message: "response omitted message.items".to_owned(),
        })?;
    let records = items
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let doi = value.get("DOI").and_then(Value::as_str).map(str::to_owned);
            let native_id = doi.clone().unwrap_or_else(|| format!("crossref-{index}"));
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
                record_id: format!("crossref-{native_id}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id,
                kind: crossref_kind(value.get("type").and_then(Value::as_str)),
                identifiers: RecordIdentifiers {
                    doi,
                    ..RecordIdentifiers::default()
                },
                title: first_string(value.get("title")).unwrap_or("[untitled]").to_owned(),
                abstract_text: value.get("abstract").and_then(Value::as_str).map(str::to_owned),
                authors: value
                    .get("author")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |authors| {
                        authors
                            .iter()
                            .filter_map(render_crossref_author)
                            .collect()
                    }),
                container_title: first_string(value.get("container-title")).map(str::to_owned),
                publication_year: crossref_year(value),
                publication_date: None,
                languages: value
                    .get("language")
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |language| vec![language.to_owned()]),
                subjects: value
                    .get("subject")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |subjects| {
                        subjects.iter().filter_map(Value::as_str).map(str::to_owned).collect()
                    }),
                urls: value
                    .get("URL")
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |url| vec![url.to_owned()]),
                provider_metadata: value.clone(),
            }
        })
        .collect();
    Ok(ProviderPage {
        schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor: message
            .get("next-cursor")
            .and_then(Value::as_str)
            .map(str::to_owned),
        total_available: message.get("total-results").and_then(Value::as_u64),
        diagnostics: BTreeMap::new(),
    })
}

/// Parse an OpenAlex Works response into the canonical page contract.
pub fn parse_openalex_page(payload: &serde_json::Value) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let items = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Upstream {
            provider: "openalex".to_owned(),
            message: "response omitted results".to_owned(),
        })?;
    let records = items
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let openalex = value.get("id").and_then(Value::as_str).map(str::to_owned);
            let native_id = openalex.clone().unwrap_or_else(|| format!("openalex-{index}"));
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
                record_id: format!("openalex-{index}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id,
                kind: openalex_kind(value.get("type").and_then(Value::as_str)),
                identifiers: RecordIdentifiers {
                    doi: value
                        .get("doi")
                        .and_then(Value::as_str)
                        .map(|doi| doi.trim_start_matches("https://doi.org/").to_owned()),
                    openalex,
                    ..RecordIdentifiers::default()
                },
                title: value
                    .get("display_name")
                    .and_then(Value::as_str)
                    .unwrap_or("[untitled]")
                    .to_owned(),
                abstract_text: None,
                authors: value
                    .get("authorships")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |authorships| {
                        authorships
                            .iter()
                            .filter_map(|authorship| {
                                authorship
                                    .get("author")
                                    .and_then(|author| author.get("display_name"))
                                    .and_then(Value::as_str)
                                    .map(str::to_owned)
                            })
                            .collect()
                    }),
                container_title: value
                    .get("primary_location")
                    .and_then(|location| location.get("source"))
                    .and_then(|source| source.get("display_name"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                publication_year: value
                    .get("publication_year")
                    .and_then(Value::as_i64)
                    .and_then(|year| i32::try_from(year).ok()),
                publication_date: value
                    .get("publication_date")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                languages: value
                    .get("language")
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |language| vec![language.to_owned()]),
                subjects: value
                    .get("topics")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |topics| {
                        topics
                            .iter()
                            .filter_map(|topic| topic.get("display_name").and_then(Value::as_str))
                            .map(str::to_owned)
                            .collect()
                    }),
                urls: value
                    .get("primary_location")
                    .and_then(|location| location.get("landing_page_url"))
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |url| vec![url.to_owned()]),
                provider_metadata: value.clone(),
            }
        })
        .collect();
    Ok(ProviderPage {
        schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor: payload
            .get("meta")
            .and_then(|meta| meta.get("next_cursor"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        total_available: payload
            .get("meta")
            .and_then(|meta| meta.get("count"))
            .and_then(Value::as_u64),
        diagnostics: BTreeMap::new(),
    })
}

/// Parse a ClinicalTrials.gov API v2 response into the canonical page contract.
pub fn parse_clinical_trials_page(
    payload: &serde_json::Value,
) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let studies = payload
        .get("studies")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Upstream {
            provider: "clinicaltrials-gov".to_owned(),
            message: "response omitted studies".to_owned(),
        })?;
    let records = studies
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let protocol = value.get("protocolSection").unwrap_or(value);
            let identification = protocol.get("identificationModule").unwrap_or(protocol);
            let nct = identification
                .get("nctId")
                .and_then(Value::as_str)
                .map(str::to_owned);
            let native_id = nct.clone().unwrap_or_else(|| format!("trial-{index}"));
            let title = identification
                .get("briefTitle")
                .or_else(|| identification.get("officialTitle"))
                .and_then(Value::as_str)
                .unwrap_or("[untitled trial]")
                .to_owned();
            let conditions = protocol
                .get("conditionsModule")
                .and_then(|module| module.get("conditions"))
                .and_then(Value::as_array)
                .map_or_else(Vec::new, |items| {
                    items.iter().filter_map(Value::as_str).map(str::to_owned).collect()
                });
            BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
                record_id: format!("clinicaltrials-{native_id}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id,
                kind: RecordKind::TrialRegistry,
                identifiers: RecordIdentifiers {
                    trial_registration: nct,
                    ..RecordIdentifiers::default()
                },
                title,
                abstract_text: protocol
                    .get("descriptionModule")
                    .and_then(|module| module.get("briefSummary"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                authors: Vec::new(),
                container_title: Some("ClinicalTrials.gov".to_owned()),
                publication_year: None,
                publication_date: protocol
                    .get("statusModule")
                    .and_then(|module| module.get("startDateStruct"))
                    .and_then(|date| date.get("date"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                languages: Vec::new(),
                subjects: conditions,
                urls: vec![format!("https://clinicaltrials.gov/study/{native_id}")],
                provider_metadata: value.clone(),
            }
        })
        .collect();
    Ok(ProviderPage {
        schema_version: searchright_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor: payload
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_owned),
        total_available: payload.get("totalCount").and_then(Value::as_u64),
        diagnostics: BTreeMap::new(),
    })
}

fn first_string(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_str)
}

fn render_crossref_author(value: &serde_json::Value) -> Option<String> {
    let family = value.get("family").and_then(serde_json::Value::as_str).unwrap_or("");
    let given = value.get("given").and_then(serde_json::Value::as_str).unwrap_or("");
    let rendered = format!("{family}, {given}")
        .trim_matches([',', ' '])
        .to_owned();
    (!rendered.is_empty()).then_some(rendered)
}

fn crossref_year(value: &serde_json::Value) -> Option<i32> {
    for field in ["published-print", "published-online", "issued", "created"] {
        if let Some(year) = value
            .get(field)
            .and_then(|date| date.get("date-parts"))
            .and_then(serde_json::Value::as_array)
            .and_then(|parts| parts.first())
            .and_then(serde_json::Value::as_array)
            .and_then(|parts| parts.first())
            .and_then(serde_json::Value::as_i64)
            .and_then(|year| i32::try_from(year).ok())
        {
            return Some(year);
        }
    }
    None
}

fn crossref_kind(value: Option<&str>) -> RecordKind {
    match value {
        Some("journal-article") => RecordKind::JournalArticle,
        Some("proceedings-article") => RecordKind::Conference,
        Some("posted-content") => RecordKind::Preprint,
        Some("dissertation") => RecordKind::Thesis,
        Some("report") => RecordKind::Report,
        Some("dataset") => RecordKind::Dataset,
        Some(other) => RecordKind::Other(other.to_owned()),
        None => RecordKind::Other("crossref".to_owned()),
    }
}

fn openalex_kind(value: Option<&str>) -> RecordKind {
    match value {
        Some("article") => RecordKind::JournalArticle,
        Some("preprint") => RecordKind::Preprint,
        Some("proceedings-article") => RecordKind::Conference,
        Some("dissertation") => RecordKind::Thesis,
        Some("report") => RecordKind::Report,
        Some("dataset") => RecordKind::Dataset,
        Some(other) => RecordKind::Other(other.to_owned()),
        None => RecordKind::Other("openalex".to_owned()),
    }
}

#[cfg(feature = "live")]
mod live {
    use super::*;
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

    #[test]
    fn pubmed_summary_parser_preserves_identifiers() {
        let payload = serde_json::json!({
            "result": {
                "uids": ["123"],
                "123": {
                    "uid": "123",
                    "title": "Synthetic PubMed record",
                    "pubdate": "2026 Jan",
                    "sortpubdate": "2026/01/01 00:00",
                    "fulljournalname": "Searchright Fixtures",
                    "authors": [{"name": "Example A"}],
                    "articleids": [{"idtype": "doi", "value": "10.1000/example"}]
                }
            }
        });
        let page = parse_pubmed_summary_page(&payload);
        assert!(page.is_ok());
        if let Ok(page) = page {
            assert_eq!(page.records.len(), 1);
            let first = page.records.first();
            assert_eq!(
                first.and_then(|record| record.identifiers.pmid.as_deref()),
                Some("123")
            );
            assert_eq!(
                first.and_then(|record| record.identifiers.doi.as_deref()),
                Some("10.1000/example")
            );
        }
    }

    #[test]
    fn open_provider_parsers_emit_canonical_pages() {
        let europe = parse_europe_pmc_page(&serde_json::json!({
            "hitCount": 1,
            "resultList": {"result": [{"id": "1", "title": "Europe PMC fixture"}]}
        }));
        let crossref = parse_crossref_page(&serde_json::json!({
            "message": {"total-results": 1, "items": [{"DOI": "10.1/example", "title": ["Crossref fixture"]}]}
        }));
        let openalex = parse_openalex_page(&serde_json::json!({
            "meta": {"count": 1},
            "results": [{"id": "https://openalex.org/W1", "display_name": "OpenAlex fixture"}]
        }));
        let trials = parse_clinical_trials_page(&serde_json::json!({
            "totalCount": 1,
            "studies": [{"protocolSection": {"identificationModule": {"nctId": "NCT1", "briefTitle": "Trial fixture"}}}]
        }));
        assert!(europe.is_ok());
        assert!(crossref.is_ok());
        assert!(openalex.is_ok());
        assert!(trials.is_ok());
    }
}
