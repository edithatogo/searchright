//! Complete, manually reviewed synthetic-page expectations, not live-provider evidence.
//!
//! Goldens are authored from the checked-in source fixtures and field mappings;
//! they must not be refreshed automatically from parser output.

use evidence_search_contracts::{ProviderPage, Validate};
use evidence_search_core::ProviderError;
use searchright_connectors::{
    parse_crossref_page, parse_europe_pmc_page, parse_openalex_page, parse_pubmed_summary_page,
};
use serde_json::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn assert_canonical_page(
    raw: &str,
    expected: &str,
    parser: fn(&Value) -> Result<ProviderPage, ProviderError>,
) -> TestResult {
    let payload: Value = serde_json::from_str(raw)?;
    let golden: Value = serde_json::from_str(expected)?;
    let page = parser(&payload)?;
    page.validate()?;
    assert_eq!(serde_json::to_value(&page)?, golden);
    // A second parse must preserve the entire page, not just its record count.
    assert_eq!(serde_json::to_value(parser(&payload)?)?, golden);
    Ok(())
}

#[test]
fn pubmed_summary_matches_complete_synthetic_golden() -> TestResult {
    assert_canonical_page(
        include_str!("../../../provider-fixtures/mvp/pubmed-esummary.json"),
        include_str!("fixtures/pubmed-summary-page.json"),
        parse_pubmed_summary_page,
    )
}

#[test]
fn europe_pmc_matches_complete_synthetic_golden() -> TestResult {
    assert_canonical_page(
        include_str!("../../../provider-fixtures/mvp/europe-pmc.json"),
        include_str!("fixtures/europe-pmc-page.json"),
        parse_europe_pmc_page,
    )
}

#[test]
fn crossref_matches_complete_synthetic_golden() -> TestResult {
    assert_canonical_page(
        include_str!("../../../provider-fixtures/mvp/crossref.json"),
        include_str!("fixtures/crossref-page.json"),
        parse_crossref_page,
    )
}

#[test]
fn openalex_matches_complete_synthetic_golden() -> TestResult {
    assert_canonical_page(
        include_str!("../../../provider-fixtures/mvp/openalex.json"),
        include_str!("fixtures/openalex-page.json"),
        parse_openalex_page,
    )
}
