//! Offline parser integrity regressions; no provider requests or live claims.

use evidence_search_contracts::{ProviderPage, Validate};
use evidence_search_core::ProviderError;
use searchright_connectors::{
    parse_crossref_page, parse_europe_pmc_page, parse_openalex_page, parse_pubmed_summary_page,
};
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn openalex(items: Value) -> Result<ProviderPage, ProviderError> {
    parse_openalex_page(&json!({"results": items, "meta": {"count": 2}}))
}

fn record_id(page: &ProviderPage, native_id: &str) -> Result<String, &'static str> {
    page.records
        .iter()
        .find(|record| record.native_id == native_id)
        .map(|record| record.record_id.clone())
        .ok_or("expected native record missing")
}

#[test]
fn openalex_identity_is_stable_across_order_and_page_position() -> TestResult {
    let a = json!({"id": "https://openalex.org/W1", "display_name": "First"});
    let b = json!({"id": "https://openalex.org/W2", "display_name": "Second"});
    let forward = openalex(json!([a, b]))?;
    let reverse = openalex(json!([b, a]))?;
    let separate_page = openalex(json!([b]))?;
    assert_eq!(
        record_id(&forward, "https://openalex.org/W1")?,
        record_id(&reverse, "https://openalex.org/W1")?
    );
    assert_eq!(
        record_id(&forward, "https://openalex.org/W2")?,
        record_id(&separate_page, "https://openalex.org/W2")?
    );
    Ok(())
}

#[test]
fn openalex_different_native_ids_have_distinct_record_ids_across_pages() -> TestResult {
    let first = openalex(json!([{"id": "https://openalex.org/W1"}]))?;
    let second = openalex(json!([{"id": "https://openalex.org/W2"}]))?;
    assert_ne!(
        record_id(&first, "https://openalex.org/W1")?,
        record_id(&second, "https://openalex.org/W2")?
    );
    Ok(())
}

#[test]
fn openalex_missing_and_malformed_identity_is_rejected() {
    assert!(openalex(json!([{}])).is_err());
    for id in [
        Value::Null,
        json!(1),
        json!([]),
        json!({}),
        json!(""),
        json!(" "),
    ] {
        assert!(openalex(json!([{"id": id}])).is_err());
    }
}

#[test]
fn crossref_missing_and_malformed_identity_is_rejected() {
    assert!(parse_crossref_page(&json!({"message": {"items": [{}]}})).is_err());
    for doi in [
        Value::Null,
        json!(1),
        json!([]),
        json!({}),
        json!(""),
        json!(" "),
    ] {
        assert!(parse_crossref_page(&json!({"message": {"items": [{"DOI": doi}]}})).is_err());
    }
}

#[test]
fn europe_pmc_missing_and_malformed_identity_is_rejected() {
    assert!(
        parse_europe_pmc_page(&json!({"resultList": {"result": [{"source": "MED"}]}})).is_err()
    );
    for id in [
        Value::Null,
        json!(1),
        json!([]),
        json!({}),
        json!(""),
        json!(" "),
    ] {
        assert!(
            parse_europe_pmc_page(
                &json!({"resultList": {"result": [{"id": id, "source": "MED"}]}})
            )
            .is_err()
        );
    }
}

#[test]
fn europe_pmc_source_namespace_prevents_identifier_collisions() -> TestResult {
    let page = parse_europe_pmc_page(&json!({"resultList": {"result": [
        {"id": "123", "source": "MED", "title": "Journal record"},
        {"id": "123", "source": "PPR", "title": "Preprint record"}
    ]}}))?;
    page.validate()?;
    let mut records = page.records.iter();
    let first = records.next().ok_or("first record missing")?;
    let second = records.next().ok_or("second record missing")?;
    assert_ne!(first.record_id, second.record_id);
    Ok(())
}

#[test]
fn europe_pmc_source_namespace_is_required() {
    assert!(parse_europe_pmc_page(&json!({"resultList": {"result": [{"id": "123"}]}})).is_err());
    for source in [Value::Null, json!(1), json!(""), json!(" ")] {
        assert!(
            parse_europe_pmc_page(
                &json!({"resultList": {"result": [{"id": "123", "source": source}]}})
            )
            .is_err()
        );
    }
}

#[test]
fn pubmed_nonstring_and_blank_uids_are_rejected() {
    for uid in [
        Value::Null,
        json!(123),
        json!([]),
        json!({}),
        json!(""),
        json!(" "),
    ] {
        assert!(parse_pubmed_summary_page(&json!({"result": {"uids": [uid]}})).is_err());
    }
}

#[test]
fn pubmed_missing_summary_is_not_silently_dropped() {
    assert!(
        parse_pubmed_summary_page(&json!({"result": {
            "uids": ["123", "456"], "123": {"uid": "123", "title": "Present"}
        }}))
        .is_err()
    );
}

#[test]
fn pubmed_duplicate_uids_are_rejected() {
    assert!(
        parse_pubmed_summary_page(&json!({"result": {
            "uids": ["123", "123"], "123": {"uid": "123", "title": "Duplicate"}
        }}))
        .is_err()
    );
}

#[test]
fn pubmed_summary_uid_must_match_requested_entry_key() {
    for uid in [json!("456"), json!(123), Value::Null, json!("")] {
        assert!(
            parse_pubmed_summary_page(&json!({"result": {
                "uids": ["123"], "123": {"uid": uid, "title": "Mismatched"}
            }}))
            .is_err()
        );
    }
}

#[test]
fn duplicate_native_identities_are_rejected_in_each_provider_page() {
    assert!(
        openalex(json!([
            {"id": "https://openalex.org/W1", "display_name": "First"},
            {"id": "https://openalex.org/W1", "display_name": "Conflicting second"}
        ]))
        .is_err()
    );
    assert!(
        parse_crossref_page(&json!({"message": {"items": [
            {"DOI": "10.1000/example", "title": ["First"]},
            {"DOI": "10.1000/example", "title": ["Conflicting second"]}
        ]}}))
        .is_err()
    );
    assert!(
        parse_europe_pmc_page(&json!({"resultList": {"result": [
            {"source": "MED", "id": "123", "title": "First"},
            {"source": "MED", "id": "123", "title": "Conflicting second"}
        ]}}))
        .is_err()
    );
}

#[test]
fn present_malformed_total_counts_are_not_silently_discarded() {
    for count in [
        Value::Null,
        json!(-1),
        json!(1.5),
        json!("invalid"),
        json!([]),
        json!({}),
        json!(true),
    ] {
        assert!(
            parse_europe_pmc_page(&json!({"hitCount": count, "resultList": {"result": []}}))
                .is_err()
        );
        assert!(
            parse_crossref_page(&json!({"message": {"total-results": count, "items": []}}))
                .is_err()
        );
        assert!(parse_openalex_page(&json!({"meta": {"count": count}, "results": []})).is_err());
    }
}

#[test]
fn present_malformed_cursors_are_not_silently_treated_as_exhaustion() {
    for cursor in [
        json!(1),
        json!(true),
        json!([]),
        json!({}),
        json!(""),
        json!(" "),
    ] {
        assert!(
            parse_europe_pmc_page(&json!({"nextCursorMark": cursor, "resultList": {"result": []}}))
                .is_err()
        );
        assert!(
            parse_crossref_page(&json!({"message": {"next-cursor": cursor, "items": []}})).is_err()
        );
        assert!(
            parse_openalex_page(&json!({"meta": {"next_cursor": cursor}, "results": []})).is_err()
        );
    }
}

#[test]
fn absent_pagination_and_explicit_openalex_exhaustion_remain_valid() -> TestResult {
    parse_europe_pmc_page(&json!({"resultList": {"result": []}}))?.validate()?;
    parse_crossref_page(&json!({"message": {"items": []}}))?.validate()?;
    let page =
        parse_openalex_page(&json!({"meta": {"count": 0, "next_cursor": null}, "results": []}))?;
    page.validate()?;
    assert_eq!(page.next_cursor, None);
    Ok(())
}

#[test]
fn pubmed_per_record_errors_and_nonobjects_are_rejected() {
    for item in [
        json!({"uid": "123", "error": "cannot retrieve record"}),
        Value::Null,
        json!([]),
        json!("error"),
    ] {
        assert!(
            parse_pubmed_summary_page(&json!({"result": {"uids": ["123"], "123": item}})).is_err()
        );
    }
}

#[test]
fn openalex_title_fixture_is_not_lost_without_display_name() -> TestResult {
    let payload: Value =
        serde_json::from_str(include_str!("../../../provider-fixtures/mvp/openalex.json"))?;
    let page = parse_openalex_page(&payload)?;
    assert_eq!(
        page.records.first().ok_or("fixture record missing")?.title,
        "Synthetic OpenAlex fixture"
    );
    Ok(())
}

#[test]
fn valid_records_allow_absent_optional_metadata() -> TestResult {
    let pages = [
        parse_pubmed_summary_page(
            &json!({"result": {"uids": ["123"], "123": {"uid": "123", "title": "Title"}}}),
        )?,
        parse_europe_pmc_page(
            &json!({"resultList": {"result": [{"id": "123", "source": "MED", "title": "Title"}]}}),
        )?,
        parse_crossref_page(
            &json!({"message": {"items": [{"DOI": "10.1000/example", "title": ["Title"]}]}}),
        )?,
        openalex(json!([{"id": "https://openalex.org/W1", "display_name": "Title"}]))?,
    ];
    for page in pages {
        page.validate()?;
        assert_eq!(page.records.len(), 1);
    }
    Ok(())
}
