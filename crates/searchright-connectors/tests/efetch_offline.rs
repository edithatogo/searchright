//! Synthetic, offline PubMed `EFetch` contract tests, never live-provider evidence.
use evidence_search_contracts::Validate;
use searchright_connectors::{PubMedFetchRequest, parse_pubmed_fetch_page};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const FIXTURE: &str = include_str!("fixtures/pubmed-efetch.xml");

#[test]
fn efetch_scalar_descendant_content_is_rejected() {
    for (original, replacement) in [
        (
            "<PMID Version=\"1\">123</PMID>",
            "<PMID><Unexpected>123</Unexpected></PMID>",
        ),
        (
            "<Year>2026</Year>",
            "<Year><Unexpected>2026</Unexpected></Year>",
        ),
        (
            "<LastName>Example</LastName>",
            "<LastName><b>Example</b></LastName>",
        ),
        (
            "<Language>eng</Language>",
            "<Language><Unknown>eng</Unknown></Language>",
        ),
    ] {
        assert!(
            parse_pubmed_fetch_page(&FIXTURE.replace(original, replacement), &["123".to_owned()])
                .is_err()
        );
    }
}

#[test]
fn efetch_declaration_grammar_is_strict() {
    let declaration = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";
    for replacement in [
        "<?xml version='1.0' version='1.0'?>",
        "<?xml encoding='UTF-8' version='1.0'?>",
        "<?xml version='1.0' unknown='yes'?>",
        "<?xml version='1.0' standalone='maybe'?>",
        "<?xml version='1.0' standalone='yes' encoding='UTF-8'?>",
        " <?xml version='1.0'?>",
        "<?xml version='1.0'encoding='UTF-8'?>",
        "<?xml version='1.1'?>",
    ] {
        assert!(
            parse_pubmed_fetch_page(
                &FIXTURE.replace(declaration, replacement),
                &["123".to_owned()]
            )
            .is_err()
        );
    }
}

#[test]
fn efetch_endpoint_is_explicit_pubmed_xml_and_encodes_identity() -> TestResult {
    let endpoint = PubMedFetchRequest {
        pmids: vec!["123".to_owned(), "456".to_owned()],
        tool: Some("fixture tool".to_owned()),
        email: Some("fixture@example.invalid".to_owned()),
    }
    .endpoint()?;
    assert_eq!(endpoint.host_str(), Some("eutils.ncbi.nlm.nih.gov"));
    assert_eq!(endpoint.path(), "/entrez/eutils/efetch.fcgi");
    let pairs = endpoint
        .query_pairs()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        pairs.get("db").map(std::borrow::Cow::as_ref),
        Some("pubmed")
    );
    assert_eq!(
        pairs.get("retmode").map(std::borrow::Cow::as_ref),
        Some("xml")
    );
    assert_eq!(
        pairs.get("id").map(std::borrow::Cow::as_ref),
        Some("123,456")
    );
    assert_eq!(
        pairs.get("tool").map(std::borrow::Cow::as_ref),
        Some("fixture tool")
    );
    Ok(())
}

#[test]
fn efetch_request_rejects_invalid_or_duplicate_pmids() {
    for pmids in [
        vec![],
        vec!["123", "123"],
        vec![""],
        vec!["12&db=pmc"],
        vec![" "],
    ] {
        assert!(
            PubMedFetchRequest {
                pmids: pmids.into_iter().map(str::to_owned).collect(),
                tool: None,
                email: None
            }
            .endpoint()
            .is_err()
        );
    }
}

#[test]
fn efetch_fixture_preserves_bibliographic_abstract_semantics() -> TestResult {
    let page = parse_pubmed_fetch_page(FIXTURE, &["123".to_owned()])?;
    page.validate()?;
    let record = page.records.first().ok_or("missing fixture record")?;
    assert_eq!(record.record_id, "pubmed-123");
    assert_eq!(record.native_id, "123");
    assert_eq!(record.title, "Synthetic mixed title & evidence");
    assert_eq!(
        record.abstract_text.as_deref(),
        Some("BACKGROUND: First & clear.\nRESULTS: Second result.")
    );
    assert_eq!(record.authors, ["Example Ada", "Synthetic Consortium"]);
    assert_eq!(record.languages, ["eng"]);
    assert_eq!(record.subjects, ["Synthetic Subject"]);
    assert_eq!(
        record.identifiers.doi.as_deref(),
        Some("10.1000/efetch.fixture")
    );
    assert_eq!(record.identifiers.pmcid.as_deref(), Some("PMC123"));
    assert_eq!(record.publication_year, Some(2026));
    assert_eq!(record.publication_date.as_deref(), Some("2026 Spring"));
    assert_eq!(
        record.provider_metadata.pointer("/abstract_sections/0"),
        Some(
            &json!({"label":"BACKGROUND", "nlm_category":"BACKGROUND", "text":"First & clear.","attributes":{"Label":"BACKGROUND","NlmCategory":"BACKGROUND"}})
        )
    );
    assert_eq!(page.next_cursor, None);
    assert_eq!(page.total_available, Some(1));
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/pubmed-efetch-page.json"))?;
    assert_eq!(
        golden.pointer("/diagnostics/raw_response_digest"),
        Some(&json!(
            blake3::hash(FIXTURE.as_bytes()).to_hex().to_string()
        )),
        "independently computed raw fixture hash"
    );
    assert_eq!(serde_json::to_value(&page)?, golden);
    Ok(())
}

#[test]
fn efetch_rejects_duplicate_singletons_and_conflicting_identifiers() {
    for xml in [
        FIXTURE.replace(
            "<PMID Version=\"1\">123</PMID>",
            "<PMID>123</PMID><PMID>123</PMID>",
        ),
        FIXTURE.replace(
            "</ArticleTitle>",
            "</ArticleTitle><ArticleTitle>Other</ArticleTitle>",
        ),
        FIXTURE.replace(
            "<ArticleId IdType=\"pubmed\">123</ArticleId>",
            "<ArticleId IdType=\"pubmed\">456</ArticleId>",
        ),
        FIXTURE.replace(
            "</ArticleIdList>",
            "<ArticleId IdType=\"doi\">10.1000/conflict</ArticleId></ArticleIdList>",
        ),
    ] {
        assert!(parse_pubmed_fetch_page(&xml, &["123".to_owned()]).is_err());
    }
}

#[test]
fn efetch_medline_date_has_no_inferred_year() -> TestResult {
    let xml = FIXTURE.replace(
        "<Year>2026</Year><Season>Spring</Season>",
        "<MedlineDate>2025 Dec-2026 Jan</MedlineDate>",
    );
    let page = parse_pubmed_fetch_page(&xml, &["123".to_owned()])?;
    let record = page.records.first().ok_or("record missing")?;
    assert_eq!(record.publication_year, None);
    assert_eq!(
        record.publication_date.as_deref(),
        Some("2025 Dec-2026 Jan")
    );
    Ok(())
}

#[test]
fn efetch_preserves_inline_word_boundaries_without_inventing_spaces() -> TestResult {
    let xml = FIXTURE.replace(
        "Synthetic <i>mixed</i> title &amp; evidence",
        "pre<i>fix</i> &amp; <![CDATA[existing text]]>",
    );
    let page = parse_pubmed_fetch_page(&xml, &["123".to_owned()])?;
    assert_eq!(
        page.records.first().ok_or("record missing")?.title,
        "prefix & existing text"
    );
    Ok(())
}

#[test]
fn efetch_mapped_field_limit_is_inclusive() -> TestResult {
    let original = "Synthetic <i>mixed</i> title &amp; evidence";
    let boundary = FIXTURE.replace(original, &"x".repeat(64 * 1024));
    parse_pubmed_fetch_page(&boundary, &["123".to_owned()])?;
    let oversized = FIXTURE.replace(original, &"x".repeat(64 * 1024 + 1));
    assert!(parse_pubmed_fetch_page(&oversized, &["123".to_owned()]).is_err());
    Ok(())
}

#[test]
fn efetch_byte_depth_and_record_limits_accept_boundaries() -> TestResult {
    let at_bytes = format!("{FIXTURE}{}", " ".repeat(8 * 1024 * 1024 - FIXTURE.len()));
    parse_pubmed_fetch_page(&at_bytes, &["123".to_owned()])?;
    let at_depth = FIXTURE.replace(
        "<Article>",
        &format!(
            "<Article>{}x{}",
            "<Unused>".repeat(60),
            "</Unused>".repeat(60)
        ),
    );
    parse_pubmed_fetch_page(&at_depth, &["123".to_owned()])?;
    let too_deep = FIXTURE.replace(
        "<Article>",
        &format!(
            "<Article>{}x{}",
            "<Unused>".repeat(61),
            "</Unused>".repeat(61)
        ),
    );
    assert!(parse_pubmed_fetch_page(&too_deep, &["123".to_owned()]).is_err());
    let ids = (1..=1_000).map(|id| id.to_string()).collect::<Vec<_>>();
    let records = ids.iter().map(|id| format!("<PubmedArticle><MedlineCitation><PMID>{id}</PMID><Article><ArticleTitle>Synthetic</ArticleTitle></Article></MedlineCitation></PubmedArticle>")).collect::<String>();
    let at_records = format!("<PubmedArticleSet>{records}</PubmedArticleSet>");
    assert_eq!(
        parse_pubmed_fetch_page(&at_records, &ids)?.records.len(),
        1_000
    );
    let too_many = at_records.replace("</PubmedArticleSet>", "<PubmedArticle/></PubmedArticleSet>");
    assert!(parse_pubmed_fetch_page(&too_many, &ids).is_err());
    Ok(())
}

#[test]
fn efetch_elocation_doi_agrees_with_article_ids_and_preserves_source() -> TestResult {
    let matching = FIXTURE.replace("<ArticleTitle>", "<ELocationID EIdType=\"doi\" ValidYN=\"Y\">10.1000/efetch.fixture</ELocationID><ArticleTitle>");
    let page = parse_pubmed_fetch_page(&matching, &["123".to_owned()])?;
    assert_eq!(
        page.records
            .first()
            .ok_or("record missing")?
            .provider_metadata
            .pointer("/elocation_ids/0/value"),
        Some(&json!("10.1000/efetch.fixture"))
    );
    let conflict = matching.replace(
        "ValidYN=\"Y\">10.1000/efetch.fixture",
        "ValidYN=\"Y\">10.1000/conflict",
    );
    assert!(parse_pubmed_fetch_page(&conflict, &["123".to_owned()]).is_err());
    Ok(())
}

#[test]
fn efetch_present_malformed_containers_are_not_absent_metadata() {
    for (open, close) in [
        ("<Abstract>", "</Abstract>"),
        ("<AuthorList>", "</AuthorList>"),
        ("<MeshHeadingList>", "</MeshHeadingList>"),
        ("<PubDate>", "</PubDate>"),
    ] {
        let before = FIXTURE.split(open).next().unwrap_or_default();
        let after = FIXTURE.split(close).nth(1).unwrap_or_default();
        let malformed = format!("{before}{open}<Unexpected/>{close}{after}");
        assert!(parse_pubmed_fetch_page(&malformed, &["123".to_owned()]).is_err());
    }
}

#[test]
fn efetch_rejects_malformed_suffix_after_valid_record_and_redacts_errors() {
    let secret = "private-synthetic-marker";
    for xml in [
        format!("{FIXTURE}<{secret}>"),
        FIXTURE.replace("&amp;", &format!("&{secret};")),
        FIXTURE.replace("<PubmedArticleSet>", &format!("<PubmedArticleSet>{secret}")),
    ] {
        let result = parse_pubmed_fetch_page(&xml, &["123".to_owned()]);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(!error.to_string().contains(secret));
        }
    }
}

#[test]
fn efetch_requires_exact_requested_identity_set() {
    for ids in [vec![], vec!["456"], vec!["123", "456"], vec!["123", "123"]] {
        assert!(
            parse_pubmed_fetch_page(
                FIXTURE,
                &ids.into_iter().map(str::to_owned).collect::<Vec<_>>()
            )
            .is_err()
        );
    }
    let duplicate = FIXTURE.replace(
        "</PubmedArticleSet>",
        &format!(
            "{} </PubmedArticleSet>",
            FIXTURE
                .split("<PubmedArticle>")
                .nth(1)
                .unwrap_or_default()
                .split("</PubmedArticle>")
                .next()
                .map(|body| format!("<PubmedArticle>{body}</PubmedArticle>"))
                .unwrap_or_default()
        ),
    );
    assert!(parse_pubmed_fetch_page(&duplicate, &["123".to_owned()]).is_err());
}

#[test]
fn efetch_rejects_unsupported_records_and_hostile_xml() {
    for xml in [
        "<PubmedArticleSet><PubmedBookArticle/></PubmedArticleSet>",
        "<PubmedArticleSet><Unknown/></PubmedArticleSet>",
        "<!DOCTYPE PubmedArticleSet [<!ENTITY secret SYSTEM 'file:///etc/passwd'>]><PubmedArticleSet/>",
        "<PubmedArticleSet><PubmedArticle></PubmedArticleSet>",
        "<PubmedArticleSet/><!-- unterminated",
        "<PubmedArticleSet/>&custom;",
    ] {
        assert!(parse_pubmed_fetch_page(xml, &["123".to_owned()]).is_err());
    }
}

#[test]
fn efetch_enforces_depth_and_input_size_limits() {
    let deep = format!(
        "<PubmedArticleSet>{}{}{}</PubmedArticleSet>",
        "<x>".repeat(65),
        "text",
        "</x>".repeat(65)
    );
    assert!(parse_pubmed_fetch_page(&deep, &["123".to_owned()]).is_err());
    let large = " ".repeat(8 * 1024 * 1024 + 1);
    assert!(parse_pubmed_fetch_page(&large, &["123".to_owned()]).is_err());
}
