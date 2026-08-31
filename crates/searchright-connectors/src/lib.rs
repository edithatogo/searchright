//! Provider adapters and deterministic fixtures.
//!
//! Live network adapters are feature-gated. The default build has no network
//! capability and is suitable for tests, replay and contract development.

#![forbid(unsafe_code)]

mod efetch;
pub use efetch::{PubMedFetchRequest, parse_pubmed_fetch_page};

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use evidence_search_contracts::{
    BibliographicRecord, ProviderCapability, ProviderManifest, ProviderPage, ProviderSupportLevel,
    RecordIdentifiers, RecordKind, SearchRequest,
};
use evidence_search_core::{ProviderError, ProviderMode, ProviderRegistry, SearchProvider};
use serde::{Deserialize, Serialize};

/// Source-owned behavior revision for current normalized provider pages.
///
/// Bump when parser identity, admission or mapping behavior changes so intact
/// cached pages from an older parser cannot bypass the current implementation.
pub const PROVIDER_PARSER_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ".parser.2");

/// A deterministic provider backed by checked-in or caller-supplied pages.
#[derive(Debug, Clone)]
pub struct FixtureProvider {
    manifest: ProviderManifest,
    pages: BTreeMap<Option<String>, ProviderPage>,
}

impl FixtureProvider {
    fn page_for_cursor(&self, cursor: Option<&String>) -> Result<ProviderPage, ProviderError> {
        self.pages
            .get(&cursor.cloned())
            .cloned()
            .ok_or_else(|| ProviderError::Upstream {
                provider: self.manifest.provider_id.clone(),
                message: "fixture has no page for requested cursor; cursor redacted".to_owned(),
            })
    }

    /// Construct a fixture provider. Page keys are request cursors; `None` is
    /// the first page. The legacy package-only version is retained; this does
    /// not assert which parser produced caller-supplied normalized pages.
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

    /// Explicitly bind caller-supplied pages to a declared behavior version.
    ///
    /// Registry admission validates the manifest. This declaration isolates cache
    /// identities; it is not evidence that the named parser generated the pages.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
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
                schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
                total_available: Some(u64::try_from(records.len()).unwrap_or(u64::MAX)),
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
        self.page_for_cursor(request.cursor.as_ref())
    }
}

/// Add the deterministic MVP fixtures to a registry.
pub fn register_mvp_fixtures(registry: &mut ProviderRegistry) -> Result<(), ProviderError> {
    for (provider_id, display_name, native_id) in [
        ("pubmed-fixture", "PubMed fixture", "pmid:00000001"),
        ("europe-pmc-fixture", "Europe PMC fixture", "epmc:00000001"),
        (
            "crossref-fixture",
            "Crossref fixture",
            "doi:10.1000/searchright",
        ),
        (
            "openalex-fixture",
            "OpenAlex fixture",
            "openalex:W000000001",
        ),
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
        schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
        record_id: format!("{provider_id}-record-1"),
        source_receipt_id: "fixture-receipt".to_owned(),
        native_id: native_id.to_owned(),
        kind: if provider_id.contains("clinicaltrials") {
            RecordKind::TrialRegistry
        } else {
            RecordKind::JournalArticle
        },
        identifiers: RecordIdentifiers {
            doi: provider_id
                .contains("crossref")
                .then(|| "10.1000/searchright".to_owned()),
            pmid: provider_id
                .contains("pubmed")
                .then(|| "00000001".to_owned()),
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

/// Redacted endpoint construction for PubMed `ESearch`.
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
    /// Build the NCBI `ESearch` endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url =
            url::Url::parse("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi")?;
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

/// Redacted endpoint construction for PubMed `ESummary` metadata retrieval.
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
    /// Build the NCBI `ESummary` endpoint without performing a request.
    pub fn endpoint(&self) -> Result<url::Url, url::ParseError> {
        let mut url =
            url::Url::parse("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi")?;
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

/// Construct a fixed, payload-independent malformed-response diagnostic.
fn malformed(provider: &str, message: &str) -> ProviderError {
    ProviderError::MalformedResponse {
        provider: provider.to_owned(),
        format: "JSON",
        message: message.to_owned(),
    }
}

fn identity<'a>(
    value: &'a serde_json::Value,
    field: &str,
    provider: &str,
) -> Result<&'a str, ProviderError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| malformed(provider, "record identity is missing or malformed"))
}

/// Stable provider-qualified identity: each raw UTF-8 component is prefixed by
/// its byte length and a colon. Europe PMC uses source then native ID. No page
/// index, canonicalisation, or historical record rewriting participates.
fn stable_record_id(provider: &str, components: &[&str]) -> String {
    let mut result = provider.to_owned();
    for component in components {
        result.push_str(&format!(":{}:{component}", component.len()));
    }
    result
}

fn validate_rows(
    rows: &[serde_json::Value],
    provider: &str,
    fields: &[&str],
) -> Result<(), ProviderError> {
    let mut seen = BTreeSet::new();
    for row in rows {
        if !row.is_object() || row.get("error").is_some() {
            return Err(malformed(
                provider,
                "record is not an object or reports an error",
            ));
        }
        let components = fields
            .iter()
            .map(|field| identity(row, field, provider))
            .collect::<Result<Vec<_>, _>>()?;
        if !seen.insert(stable_record_id(provider, &components)) {
            return Err(malformed(provider, "duplicate record identity in page"));
        }
    }
    Ok(())
}

fn optional_count(
    value: &serde_json::Value,
    field: &str,
    provider: &str,
) -> Result<Option<u64>, ProviderError> {
    value
        .get(field)
        .map(|count| {
            count
                .as_u64()
                .ok_or_else(|| malformed(provider, "present total count is malformed"))
        })
        .transpose()
}

fn optional_cursor(
    value: &serde_json::Value,
    field: &str,
    provider: &str,
) -> Result<Option<String>, ProviderError> {
    match value.get(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(cursor) => cursor
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .map(|text| Some(text.to_owned()))
            .ok_or_else(|| malformed(provider, "present pagination cursor is malformed")),
    }
}

/// Parse a complete PubMed summary page, rejecting missing or conflicting UIDs.
pub fn parse_pubmed_summary_page(
    payload: &serde_json::Value,
) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let result = payload
        .get("result")
        .ok_or_else(|| ProviderError::Upstream {
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
    let mut seen = BTreeSet::new();
    for uid in uids {
        let pmid = uid
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| malformed("pubmed", "summary UID is malformed"))?;
        if !seen.insert(pmid) {
            return Err(malformed("pubmed", "duplicate summary UID"));
        }
        let item = result
            .get(pmid)
            .ok_or_else(|| malformed("pubmed", "summary UID has no record"))?;
        if !item.is_object()
            || item.get("error").is_some()
            || identity(item, "uid", "pubmed")? != pmid
        {
            return Err(malformed(
                "pubmed",
                "summary record is invalid or has mismatched UID",
            ));
        }
    }
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
            let authors =
                item.get("authors")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |values| {
                        values
                            .iter()
                            .filter_map(|author| author.get("name").and_then(Value::as_str))
                            .map(str::to_owned)
                            .collect()
                    });
            BibliographicRecord {
                schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
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
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
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
pub fn parse_europe_pmc_page(payload: &serde_json::Value) -> Result<ProviderPage, ProviderError> {
    use serde_json::Value;
    let list = payload
        .get("resultList")
        .and_then(|value| value.get("result"))
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Upstream {
            provider: "europe-pmc".to_owned(),
            message: "response omitted resultList.result".to_owned(),
        })?;
    validate_rows(list, "europe-pmc", &["source", "id"])?;
    let total_available = optional_count(payload, "hitCount", "europe-pmc")?;
    let next_cursor = optional_cursor(payload, "nextCursorMark", "europe-pmc")?;
    let records = list
        .iter()
        .map(|value| BibliographicRecord {
            schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                .to_owned(),
            record_id: stable_record_id(
                "europe-pmc",
                &[
                    value
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                    value.get("id").and_then(Value::as_str).unwrap_or_default(),
                ],
            ),
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
                pmcid: value
                    .get("pmcid")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
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
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor,
        total_available,
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
    validate_rows(items, "crossref", &["DOI"])?;
    let total_available = optional_count(message, "total-results", "crossref")?;
    let next_cursor = optional_cursor(message, "next-cursor", "crossref")?;
    let records = items
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let doi = value.get("DOI").and_then(Value::as_str).map(str::to_owned);
            let native_id = doi.clone().unwrap_or_else(|| format!("crossref-{index}"));
            BibliographicRecord {
                schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: format!("crossref-{native_id}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id,
                kind: crossref_kind(value.get("type").and_then(Value::as_str)),
                identifiers: RecordIdentifiers {
                    doi,
                    ..RecordIdentifiers::default()
                },
                title: first_string(value.get("title"))
                    .unwrap_or("[untitled]")
                    .to_owned(),
                abstract_text: value
                    .get("abstract")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                authors: value
                    .get("author")
                    .and_then(Value::as_array)
                    .map_or_else(Vec::new, |authors| {
                        authors.iter().filter_map(render_crossref_author).collect()
                    }),
                container_title: first_string(value.get("container-title")).map(str::to_owned),
                publication_year: crossref_year(value),
                publication_date: None,
                languages: value
                    .get("language")
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |language| vec![language.to_owned()]),
                subjects: value.get("subject").and_then(Value::as_array).map_or_else(
                    Vec::new,
                    |subjects| {
                        subjects
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    },
                ),
                urls: value
                    .get("URL")
                    .and_then(Value::as_str)
                    .map_or_else(Vec::new, |url| vec![url.to_owned()]),
                provider_metadata: value.clone(),
            }
        })
        .collect();
    Ok(ProviderPage {
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor,
        total_available,
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
    validate_rows(items, "openalex", &["id"])?;
    let meta = payload.get("meta").unwrap_or(&Value::Null);
    if payload.get("meta").is_some() && !meta.is_object() {
        return Err(malformed("openalex", "present metadata is not an object"));
    }
    let total_available = optional_count(meta, "count", "openalex")?;
    let next_cursor = optional_cursor(meta, "next_cursor", "openalex")?;
    let records = items
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let openalex = value.get("id").and_then(Value::as_str).map(str::to_owned);
            let native_id = openalex
                .clone()
                .unwrap_or_else(|| format!("openalex-{index}"));
            BibliographicRecord {
                schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: stable_record_id("openalex", &[&native_id]),
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
                    .filter(|text| !text.trim().is_empty())
                    .or_else(|| {
                        value
                            .get("title")
                            .and_then(Value::as_str)
                            .filter(|text| !text.trim().is_empty())
                    })
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
                subjects: value.get("topics").and_then(Value::as_array).map_or_else(
                    Vec::new,
                    |topics| {
                        topics
                            .iter()
                            .filter_map(|topic| topic.get("display_name").and_then(Value::as_str))
                            .map(str::to_owned)
                            .collect()
                    },
                ),
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
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        records,
        next_cursor,
        total_available,
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
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                });
            BibliographicRecord {
                schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                    .to_owned(),
                record_id: format!("clinicaltrials-{native_id}"),
                source_receipt_id: "pending-receipt".to_owned(),
                native_id: native_id.clone(),
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
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
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
    let family = value
        .get("family")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let given = value
        .get("given")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
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
    use super::{
        Arc, BTreeMap, CrossrefRequest, Deserialize, EuropePmcRequest, OpenAlexRequest,
        ProviderCapability, ProviderError, ProviderManifest, ProviderMode, ProviderPage,
        ProviderRegistry, ProviderSupportLevel, PubMedSearchRequest, PubMedSummaryRequest,
        SearchProvider, SearchRequest, Serialize, async_trait, parse_crossref_page,
        parse_europe_pmc_page, parse_openalex_page, parse_pubmed_summary_page,
    };
    use evidence_search_core::validate_resolved_endpoint_addresses;
    use reqwest::header::{HeaderMap, RETRY_AFTER};
    use serde_json::Value;
    use std::net::SocketAddr;

    const DEFAULT_MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;

    fn parse_search_ids(payload: &Value) -> Result<(u64, Vec<String>), ProviderError> {
        let result = payload
            .get("esearchresult")
            .filter(|value| value.is_object())
            .ok_or_else(|| super::malformed("pubmed", "missing ESearch result"))?;
        if payload.get("error").is_some()
            || result.get("error").is_some()
            || result.get("errorlist").is_some()
        {
            return Err(super::malformed("pubmed", "ESearch reports an error"));
        }
        let count = result
            .get("count")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|text| text.parse::<u64>().ok())
            .ok_or_else(|| super::malformed("pubmed", "ESearch count is malformed"))?;
        let ids = result
            .get("idlist")
            .and_then(Value::as_array)
            .ok_or_else(|| super::malformed("pubmed", "ESearch ID list is malformed"))?;
        let mut seen = std::collections::BTreeSet::new();
        let mut output = Vec::new();
        for id in ids {
            let id = id
                .as_str()
                .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
                .ok_or_else(|| super::malformed("pubmed", "ESearch ID is malformed"))?;
            if !seen.insert(id) {
                return Err(super::malformed("pubmed", "duplicate ESearch ID"));
            }
            output.push(id.to_owned());
        }
        if u64::try_from(output.len()).unwrap_or(u64::MAX) > count {
            return Err(super::malformed(
                "pubmed",
                "ESearch count is smaller than returned ID list",
            ));
        }
        Ok((count, output))
    }

    fn validate_search_progress(
        count: u64,
        ids: &[String],
        offset: u32,
    ) -> Result<(), ProviderError> {
        if ids.is_empty() && u64::from(offset) < count {
            return Err(super::malformed(
                "pubmed",
                "ESearch returned no IDs before the reported result count was exhausted",
            ));
        }
        Ok(())
    }

    fn reconcile_summary(requested: &[String], page: &ProviderPage) -> Result<(), ProviderError> {
        let expected = requested
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let observed = page
            .records
            .iter()
            .map(|record| record.native_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if expected.len() != requested.len()
            || observed.len() != page.records.len()
            || expected != observed
        {
            return Err(super::malformed(
                "pubmed",
                "ESummary identities differ from requested IDs",
            ));
        }
        Ok(())
    }

    /// Non-secret configuration for the four open MVP providers.
    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LiveProviderConfig {
        /// NCBI tool identifier.
        pub ncbi_tool: Option<String>,
        /// NCBI contact email supplied by the operator.
        pub ncbi_email: Option<String>,
        /// Crossref polite-pool contact email.
        pub crossref_mailto: Option<String>,
        /// OpenAlex polite-pool contact email.
        pub openalex_mailto: Option<String>,
    }

    fn build_pinned_client(
        provider: &str,
        host: &str,
        addresses: &[SocketAddr],
    ) -> Result<reqwest::Client, ProviderError> {
        let resolved = addresses.iter().map(SocketAddr::ip).collect::<Vec<_>>();
        validate_resolved_endpoint_addresses(provider, host, &resolved)?;
        reqwest::Client::builder()
            .https_only(true)
            // A system proxy would bypass origin-address pinning by resolving
            // the CONNECT hostname outside this process.
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("searchright/", env!("CARGO_PKG_VERSION")))
            .resolve_to_addrs(host, addresses)
            .build()
            .map_err(|error| ProviderError::Upstream {
                provider: provider.to_owned(),
                message: format!("could not construct HTTP client: {error}"),
            })
    }

    fn retry_after_ms(headers: &HeaderMap) -> Option<u64> {
        headers
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|seconds| seconds.saturating_mul(1_000))
    }

    fn append_bounded_chunk(
        bytes: &mut Vec<u8>,
        chunk: &[u8],
        maximum: u64,
    ) -> Result<(), ProviderError> {
        let chunk_size = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
        let next_size = u64::try_from(bytes.len())
            .unwrap_or(u64::MAX)
            .saturating_add(chunk_size);
        if next_size > maximum {
            return Err(ProviderError::BudgetExceeded {
                kind: "response_bytes",
                limit: maximum,
            });
        }
        bytes.extend_from_slice(chunk);
        Ok(())
    }

    async fn fetch_json(
        provider: &str,
        endpoint: url::Url,
        request: &SearchRequest,
    ) -> Result<(Value, String), ProviderError> {
        let host = endpoint.host_str().ok_or_else(|| ProviderError::Upstream {
            provider: provider.to_owned(),
            message: "approved endpoint omitted a DNS host".to_owned(),
        })?;
        let port = endpoint
            .port_or_known_default()
            .ok_or_else(|| ProviderError::Upstream {
                provider: provider.to_owned(),
                message: "approved endpoint omitted a transport port".to_owned(),
            })?;
        let mut addresses = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| ProviderError::Upstream {
                provider: provider.to_owned(),
                message: "DNS resolution failed; endpoint details were redacted".to_owned(),
            })?
            .collect::<Vec<_>>();
        addresses.sort_unstable();
        addresses.dedup();
        let client = build_pinned_client(provider, host, &addresses)?;
        let mut response =
            client
                .get(endpoint)
                .send()
                .await
                .map_err(|_| ProviderError::Upstream {
                    provider: provider.to_owned(),
                    message: concat!(
                        "network request failed before a response was available; ",
                        "endpoint and query details were redacted"
                    )
                    .to_owned(),
                })?;
        let status = response.status();
        let retry_after_ms = retry_after_ms(response.headers());
        if status.as_u16() == 429 {
            return Err(ProviderError::RateLimited {
                provider: provider.to_owned(),
                retry_after_ms,
            });
        }
        if !status.is_success() {
            return Err(ProviderError::HttpStatus {
                provider: provider.to_owned(),
                status: status.as_u16(),
                retry_after_ms,
                message: status.to_string(),
            });
        }
        let maximum = request
            .policy
            .max_response_bytes
            .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES);
        if response.content_length().is_some_and(|size| size > maximum) {
            return Err(ProviderError::BudgetExceeded {
                kind: "response_bytes",
                limit: maximum,
            });
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| ProviderError::Upstream {
                provider: provider.to_owned(),
                message: concat!(
                    "response body retrieval failed; ",
                    "endpoint and query details were redacted"
                )
                .to_owned(),
            })?
        {
            append_bounded_chunk(&mut bytes, &chunk, maximum)?;
        }
        let digest = blake3::hash(&bytes).to_hex().to_string();
        let payload =
            serde_json::from_slice(&bytes).map_err(|error| ProviderError::MalformedResponse {
                provider: provider.to_owned(),
                format: "JSON",
                message: error.to_string(),
            })?;
        Ok((payload, digest))
    }

    fn open_manifest(
        provider_id: &str,
        display_name: &str,
        allowed_hosts: &[&str],
        interval_ms: u64,
    ) -> ProviderManifest {
        ProviderManifest {
            provider_id: provider_id.to_owned(),
            display_name: display_name.to_owned(),
            version: super::PROVIDER_PARSER_VERSION.to_owned(),
            support_level: ProviderSupportLevel::OptInLive,
            capabilities: vec![ProviderCapability::Search, ProviderCapability::Pagination],
            allowed_hosts: allowed_hosts.iter().map(|value| (*value).to_owned()).collect(),
            authentication_required: false,
            licensed: false,
            default_min_interval_ms: interval_ms,
            policy_notes: vec![
                "live execution is feature-gated and also requires explicit execution-policy approval"
                    .to_owned(),
                "responses are bounded, hashed before normalisation and never written unless a cache is explicitly configured"
                    .to_owned(),
            ],
        }
    }

    /// Opt-in PubMed `ESearch` plus `ESummary` adapter.
    #[derive(Debug, Clone)]
    pub struct PubMedProvider {
        tool: Option<String>,
        email: Option<String>,
    }

    impl PubMedProvider {
        /// Construct a PubMed adapter with optional NCBI identity fields.
        pub const fn new(
            tool: Option<String>,
            email: Option<String>,
        ) -> Result<Self, ProviderError> {
            Ok(Self { tool, email })
        }
    }

    #[async_trait]
    impl SearchProvider for PubMedProvider {
        fn manifest(&self) -> ProviderManifest {
            open_manifest(
                "pubmed",
                "PubMed",
                &["eutils.ncbi.nlm.nih.gov"],
                if self.email.is_some() { 350 } else { 1_000 },
            )
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi".to_owned())
        }

        async fn execute_page(
            &self,
            request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            let offset = request
                .cursor
                .as_deref()
                .unwrap_or("0")
                .parse::<u32>()
                .map_err(|error| {
                    ProviderError::InvalidRequest(format!(
                        "PubMed cursor must be a result offset: {error}"
                    ))
                })?;
            let endpoint = PubMedSearchRequest {
                query: request.strategy.query.clone(),
                page_size: request.page_size,
                offset,
                tool: self.tool.clone(),
                email: self.email.clone(),
            }
            .endpoint()
            .map_err(|error| ProviderError::Upstream {
                provider: "pubmed".to_owned(),
                message: error.to_string(),
            })?;
            let (search, search_digest) = fetch_json("pubmed", endpoint, request).await?;
            let (count, pmids) = parse_search_ids(&search)?;
            validate_search_progress(count, &pmids, offset)?;
            let count = Some(count);
            if pmids.is_empty() {
                return Ok(ProviderPage {
                    schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION
                        .to_owned(),
                    records: Vec::new(),
                    next_cursor: None,
                    total_available: count,
                    diagnostics: BTreeMap::from([(
                        "raw_response_digest".to_owned(),
                        Value::String(search_digest),
                    )]),
                });
            }
            let summary_endpoint = PubMedSummaryRequest {
                pmids: pmids.clone(),
                tool: self.tool.clone(),
                email: self.email.clone(),
            }
            .endpoint()
            .map_err(|error| ProviderError::Upstream {
                provider: "pubmed".to_owned(),
                message: error.to_string(),
            })?;
            let (summary, summary_digest) = fetch_json("pubmed", summary_endpoint, request).await?;
            let mut page = parse_pubmed_summary_page(&summary)?;
            reconcile_summary(&pmids, &page)?;
            let next = offset.saturating_add(u32::try_from(pmids.len()).unwrap_or(u32::MAX));
            page.total_available = count;
            page.next_cursor = count
                .filter(|total| u64::from(next) < *total)
                .map(|_| next.to_string());
            page.diagnostics.insert(
                "raw_response_digests".to_owned(),
                Value::Array(vec![
                    Value::String(search_digest),
                    Value::String(summary_digest),
                ]),
            );
            Ok(page)
        }
    }

    /// Opt-in Europe PMC live adapter.
    #[derive(Debug, Clone)]
    pub struct EuropePmcProvider;

    impl EuropePmcProvider {
        /// Construct a redirect-disabled HTTPS-only client for Europe PMC.
        pub const fn new() -> Result<Self, ProviderError> {
            Ok(Self)
        }
    }

    #[async_trait]
    impl SearchProvider for EuropePmcProvider {
        fn manifest(&self) -> ProviderManifest {
            open_manifest("europe-pmc", "Europe PMC", &["www.ebi.ac.uk"], 1_000)
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some("https://www.ebi.ac.uk/europepmc/webservices/rest/search".to_owned())
        }

        async fn execute_page(
            &self,
            request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
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
            let (payload, digest) = fetch_json("europe-pmc", endpoint, request).await?;
            let mut page = parse_europe_pmc_page(&payload)?;
            page.diagnostics
                .insert("raw_response_digest".to_owned(), Value::String(digest));
            Ok(page)
        }
    }

    /// Opt-in Crossref Works adapter.
    #[derive(Debug, Clone)]
    pub struct CrossrefProvider {
        mailto: Option<String>,
    }

    impl CrossrefProvider {
        /// Construct a Crossref adapter with an optional polite-pool contact.
        pub const fn new(mailto: Option<String>) -> Result<Self, ProviderError> {
            Ok(Self { mailto })
        }
    }

    #[async_trait]
    impl SearchProvider for CrossrefProvider {
        fn manifest(&self) -> ProviderManifest {
            open_manifest("crossref", "Crossref", &["api.crossref.org"], 1_000)
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some("https://api.crossref.org/works".to_owned())
        }

        async fn execute_page(
            &self,
            request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            let endpoint = CrossrefRequest {
                query: request.strategy.query.clone(),
                rows: request.page_size,
                cursor: request.cursor.clone().or_else(|| Some("*".to_owned())),
                mailto: self.mailto.clone(),
            }
            .endpoint()
            .map_err(|error| ProviderError::Upstream {
                provider: "crossref".to_owned(),
                message: error.to_string(),
            })?;
            let (payload, digest) = fetch_json("crossref", endpoint, request).await?;
            let mut page = parse_crossref_page(&payload)?;
            page.diagnostics
                .insert("raw_response_digest".to_owned(), Value::String(digest));
            Ok(page)
        }
    }

    /// Opt-in OpenAlex Works adapter.
    #[derive(Debug, Clone)]
    pub struct OpenAlexProvider {
        mailto: Option<String>,
    }

    impl OpenAlexProvider {
        /// Construct an OpenAlex adapter with an optional polite-pool contact.
        pub const fn new(mailto: Option<String>) -> Result<Self, ProviderError> {
            Ok(Self { mailto })
        }
    }

    #[async_trait]
    impl SearchProvider for OpenAlexProvider {
        fn manifest(&self) -> ProviderManifest {
            open_manifest("openalex", "OpenAlex", &["api.openalex.org"], 1_000)
        }

        fn mode(&self) -> ProviderMode {
            ProviderMode::Live
        }

        fn endpoint_label(&self) -> Option<String> {
            Some("https://api.openalex.org/works".to_owned())
        }

        async fn execute_page(
            &self,
            request: &SearchRequest,
        ) -> Result<ProviderPage, ProviderError> {
            let endpoint = OpenAlexRequest {
                query: request.strategy.query.clone(),
                per_page: request.page_size,
                cursor: request.cursor.clone(),
                mailto: self.mailto.clone(),
            }
            .endpoint()
            .map_err(|error| ProviderError::Upstream {
                provider: "openalex".to_owned(),
                message: error.to_string(),
            })?;
            let (payload, digest) = fetch_json("openalex", endpoint, request).await?;
            let mut page = parse_openalex_page(&payload)?;
            page.diagnostics
                .insert("raw_response_digest".to_owned(), Value::String(digest));
            Ok(page)
        }
    }

    /// Register the four open MVP live providers. Live execution still requires
    /// the Cargo feature and each request's explicit `live_enabled` policy.
    pub fn register_mvp_live_providers(
        registry: &mut ProviderRegistry,
        config: LiveProviderConfig,
    ) -> Result<(), ProviderError> {
        registry.register(Arc::new(PubMedProvider::new(
            config.ncbi_tool,
            config.ncbi_email,
        )?))?;
        registry.register(Arc::new(EuropePmcProvider::new()?))?;
        registry.register(Arc::new(CrossrefProvider::new(config.crossref_mailto)?))?;
        registry.register(Arc::new(OpenAlexProvider::new(config.openalex_mailto)?))?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::{append_bounded_chunk, build_pinned_client};
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        #[test]
        fn esearch_rejects_malformed_counts_ids_and_errors() {
            for count in [
                serde_json::Value::Null,
                serde_json::json!(1),
                serde_json::json!("-1"),
                serde_json::json!("invalid"),
                serde_json::json!(""),
            ] {
                assert!(
                    super::parse_search_ids(
                        &serde_json::json!({"esearchresult": {"count": count, "idlist": []}})
                    )
                    .is_err()
                );
            }
            for ids in [
                serde_json::json!([123]),
                serde_json::json!(["123", "123"]),
                serde_json::json!([""]),
                serde_json::json!(["abc"]),
                serde_json::Value::Null,
            ] {
                assert!(
                    super::parse_search_ids(
                        &serde_json::json!({"esearchresult": {"count": "2", "idlist": ids}})
                    )
                    .is_err()
                );
            }
            assert!(
                super::parse_search_ids(
                    &serde_json::json!({"esearchresult": {"count": "0", "idlist": ["123"]}})
                )
                .is_err()
            );
            assert!(super::parse_search_ids(&serde_json::json!({"esearchresult": {"count": "0", "idlist": [], "errorlist": {"phrasesnotfound": ["sensitive"]}}})).is_err());
        }

        #[test]
        fn esearch_and_summary_reconciliation_is_exact() -> Result<(), Box<dyn std::error::Error>> {
            let (count, ids) = super::parse_search_ids(
                &serde_json::json!({"esearchresult": {"count": "2", "idlist": ["123", "456"]}}),
            )?;
            assert_eq!(count, 2);
            let page = super::parse_pubmed_summary_page(
                &serde_json::json!({"result": {"uids": ["456", "123"], "123": {"uid": "123"}, "456": {"uid": "456"}}}),
            )?;
            super::reconcile_summary(&ids, &page)?;
            assert!(super::reconcile_summary(&["123".to_owned()], &page).is_err());
            assert!(
                super::reconcile_summary(&["123".to_owned(), "789".to_owned()], &page).is_err()
            );
            assert!(
                super::reconcile_summary(&["123".to_owned(), "123".to_owned()], &page).is_err()
            );
            Ok(())
        }

        #[test]
        fn esearch_empty_page_requires_exhausted_count_but_short_pages_are_allowed() {
            assert!(super::validate_search_progress(10, &[], 0).is_err());
            assert!(super::validate_search_progress(10, &[], 9).is_err());
            assert!(super::validate_search_progress(10, &[], 10).is_ok());
            assert!(super::validate_search_progress(10, &[], 11).is_ok());
            assert!(super::validate_search_progress(0, &[], 0).is_ok());
            assert!(super::validate_search_progress(10, &["123".to_owned()], 0).is_ok());
        }

        #[test]
        fn pinned_client_rejects_an_empty_dns_answer() {
            assert!(build_pinned_client("test", "example.test", &[]).is_err());
        }

        #[test]
        fn pinned_client_rejects_the_whole_answer_if_any_address_is_prohibited() {
            let public = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 443);
            let private = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 443);
            assert!(build_pinned_client("test", "example.test", &[public, private]).is_err());
        }

        #[test]
        fn pinned_client_accepts_and_pins_a_complete_public_answer() {
            let public = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 443);
            assert!(build_pinned_client("test", "example.test", &[public]).is_ok());
        }

        #[test]
        fn response_chunks_are_rejected_before_the_buffer_exceeds_its_budget() {
            let mut bytes = vec![1, 2, 3];
            assert!(append_bounded_chunk(&mut bytes, &[4], 4).is_ok());
            assert!(append_bounded_chunk(&mut bytes, &[5], 4).is_err());
            assert_eq!(bytes, vec![1, 2, 3, 4]);
        }
    }
}

#[cfg(feature = "live")]
pub use live::{
    CrossrefProvider, EuropePmcProvider, LiveProviderConfig, OpenAlexProvider, PubMedProvider,
    register_mvp_live_providers,
};

#[cfg(test)]
mod tests {
    use super::*;
    use evidence_search_contracts::Validate;

    #[test]
    fn fixture_missing_cursor_error_is_redacted() {
        let provider = FixtureProvider::one_page("test", "Test", Vec::new());
        let error = provider.page_for_cursor(Some(&"secret-cursor-token".to_owned()));
        assert!(error.is_err());
        if let Err(error) = error {
            assert!(!error.to_string().contains("secret-cursor-token"));
            assert!(error.to_string().contains("redacted"));
        }
    }

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
            "resultList": {"result": [{"id": "1", "source": "MED", "title": "Europe PMC fixture"}]}
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

    #[test]
    fn checked_in_open_provider_fixtures_compile_to_valid_canonical_pages()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixtures = [
            (
                include_str!("../../../provider-fixtures/mvp/pubmed-esummary.json"),
                parse_pubmed_summary_page
                    as fn(&serde_json::Value) -> Result<ProviderPage, ProviderError>,
                "10.1000/searchright.1",
            ),
            (
                include_str!("../../../provider-fixtures/mvp/europe-pmc.json"),
                parse_europe_pmc_page,
                "10.1000/searchright.3",
            ),
            (
                include_str!("../../../provider-fixtures/mvp/crossref.json"),
                parse_crossref_page,
                "10.1000/searchright.4",
            ),
            (
                include_str!("../../../provider-fixtures/mvp/openalex.json"),
                parse_openalex_page,
                "10.1000/searchright.5",
            ),
        ];

        for (fixture, parser, expected_doi) in fixtures {
            let payload: serde_json::Value = serde_json::from_str(fixture)?;
            let page = parser(&payload)?;
            page.validate()?;
            assert!(
                page.records
                    .iter()
                    .any(|record| { record.identifiers.doi.as_deref() == Some(expected_doi) })
            );
        }
        Ok(())
    }
}
