//! XML import regressions: identifiers, literal text and record isolation.

use searchright_interchange::{import_endnote_xml, import_pubmed_xml};

#[test]
fn attributed_elements_preserve_pubmed_identity() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"<PubmedArticle Status="MEDLINE"><MedlineCitation><PMID Version="1">12345</PMID><Article><ArticleTitle>Alpha trial</ArticleTitle></Article><MeshHeadingList><MeshHeading><DescriptorName UI="D000001" MajorTopicYN="N">Alpha</DescriptorName></MeshHeading></MeshHeadingList></MedlineCitation></PubmedArticle>"#;
    let result = import_pubmed_xml(input, "receipt")?;
    let record = result.records.first().ok_or("missing record")?;
    assert_eq!(record.native_id, "12345");
    assert_eq!(record.identifiers.pmid.as_deref(), Some("12345"));
    assert_eq!(record.subjects, ["Alpha"]);
    assert!(result.quarantined.is_empty());
    Ok(())
}

#[test]
fn xml_references_are_decoded_once_and_cdata_remains_literal()
-> Result<(), Box<dyn std::error::Error>> {
    let input = "<PubmedArticle><PMID>1</PMID><ArticleTitle>A &amp;lt; B &#x3b1; <![CDATA[C < D &amp;]]></ArticleTitle></PubmedArticle>";
    let result = import_pubmed_xml(input, "receipt")?;
    assert_eq!(
        result.records.first().ok_or("missing record")?.title,
        "A &lt; B α C < D &amp;"
    );
    Ok(())
}

#[test]
fn unclosed_record_cannot_swallow_a_following_record() -> Result<(), Box<dyn std::error::Error>> {
    let first = "<PubmedArticle><PMID>1</PMID><ArticleTitle>First</ArticleTitle>\n";
    let second = "<PubmedArticle><PMID>2</PMID><ArticleTitle>Second</ArticleTitle></PubmedArticle>";
    let result = import_pubmed_xml(&format!("{first}{second}"), "receipt")?;
    assert_eq!(result.records.len(), 1);
    assert_eq!(
        result.records.first().ok_or("missing record")?.native_id,
        "2"
    );
    let bad = result.quarantined.first().ok_or("missing quarantine")?;
    assert_eq!(bad.raw_content, first);
    assert_eq!((bad.start_line, bad.end_line, bad.index), (1, 1, 1));
    Ok(())
}

#[test]
fn comments_and_cdata_do_not_create_record_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<PubmedArticle><PMID>1</PMID><!-- <PubmedArticle> --><ArticleTitle><![CDATA[A <PubmedArticle> B]]></ArticleTitle></PubmedArticle>";
    let result = import_pubmed_xml(input, "receipt")?;
    assert!(result.quarantined.is_empty());
    assert_eq!(
        result.records.first().ok_or("missing record")?.title,
        "A <PubmedArticle> B"
    );
    Ok(())
}

#[test]
fn malformed_inner_markup_is_quarantined() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<PubmedArticle><PMID>1</PMID><ArticleTitle>First</Wrong></PubmedArticle><PubmedArticle><PMID>2</PMID><ArticleTitle>Second</ArticleTitle></PubmedArticle>";
    let result = import_pubmed_xml(input, "receipt")?;
    assert_eq!(result.quarantined.len(), 1);
    assert_eq!(result.records.len(), 1);
    assert_eq!(
        result.records.first().ok_or("missing record")?.native_id,
        "2"
    );
    Ok(())
}

#[test]
fn endnote_record_isolation_uses_the_same_structural_parser()
-> Result<(), Box<dyn std::error::Error>> {
    let input = "<record><rec-number>1</rec-number><title>First</title>\n<record key='two'><rec-number>2</rec-number><title>A &#38; B</title></record>";
    let result = import_endnote_xml(input, "receipt")?;
    assert_eq!(result.quarantined.len(), 1);
    let record = result.records.first().ok_or("missing record")?;
    assert_eq!(record.native_id, "2");
    assert_eq!(record.title, "A & B");
    Ok(())
}

#[test]
fn invalid_references_attributes_and_embedded_dtd_are_quarantined()
-> Result<(), Box<dyn std::error::Error>> {
    let invalid = [
        "<PubmedArticle><PMID>1</PMID><ArticleTitle>&external;</ArticleTitle></PubmedArticle>",
        "<PubmedArticle><PMID>1</PMID><ArticleTitle>&#0;</ArticleTitle></PubmedArticle>",
        "<PubmedArticle duplicate='a' duplicate='b'><PMID>1</PMID></PubmedArticle>",
        "<PubmedArticle field='&external;'><PMID>1</PMID></PubmedArticle>",
        "<PubmedArticle key&jey='value'><PMID>1</PMID></PubmedArticle>",
        "<PubmedArticle field='a < b'><PMID>1</PMID></PubmedArticle>",
        "<PubmedArticle><!DOCTYPE article SYSTEM 'file:///unread-test-file'><PMID>1</PMID></PubmedArticle>",
        "<PubmedArticle><!-- invalid -- comment --><PMID>1</PMID></PubmedArticle>",
    ];
    let good = "<PubmedArticle><PMID>2</PMID><ArticleTitle>Valid</ArticleTitle></PubmedArticle>";
    for bad in invalid {
        let result = import_pubmed_xml(&format!("{bad}{good}"), "receipt")?;
        assert_eq!(result.quarantined.len(), 1, "{bad}");
        assert_eq!(
            result
                .quarantined
                .first()
                .ok_or("missing quarantine")?
                .raw_content,
            bad
        );
        assert_eq!(result.records.len(), 1, "{bad}");
        assert_eq!(
            result.records.first().ok_or("missing record")?.native_id,
            "2"
        );
    }
    Ok(())
}

#[test]
fn cited_identifiers_never_become_primary_identity() -> Result<(), Box<dyn std::error::Error>> {
    for primary in [
        "",
        "<ArticleId IdType='pubmed'>10.1000/wrong-type</ArticleId>",
        "<ArticleId IdType='doi'>10.1000/primary</ArticleId>",
    ] {
        let input = format!(
            "<PubmedArticle><MedlineCitation><PMID>1</PMID><Article><ArticleTitle>Main</ArticleTitle></Article></MedlineCitation><PubmedData><ReferenceList><Reference><PMID>999</PMID><ArticleIdList><ArticleId IdType='doi'>10.1000/cited</ArticleId></ArticleIdList></Reference></ReferenceList><ArticleIdList>{primary}</ArticleIdList></PubmedData></PubmedArticle>"
        );
        let result = import_pubmed_xml(&input, "receipt")?;
        let record = result.records.first().ok_or("missing record")?;
        assert_eq!(record.identifiers.pmid.as_deref(), Some("1"));
        assert_eq!(
            record.identifiers.doi.as_deref(),
            primary.contains("/primary").then_some("10.1000/primary")
        );
    }
    let input = "<PubmedArticle field='a &lt; b'><PubmedData><ReferenceList><Reference><PMID>999</PMID><ArticleTitle>Cited title</ArticleTitle></Reference></ReferenceList></PubmedData></PubmedArticle>";
    let result = import_pubmed_xml(input, "receipt")?;
    let record = result.records.first().ok_or("missing record")?;
    assert!(record.identifiers.pmid.is_none());
    assert_eq!(record.title, "Untitled imported record");
    Ok(())
}

#[test]
fn publication_date_keeps_its_year_scope() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<PubmedArticle><PMID>3</PMID><ArticleDate><Year>2025</Year></ArticleDate><PubDate Type='print'><Year>2024</Year></PubDate><ArticleTitle>Trial</ArticleTitle></PubmedArticle>";
    let result = import_pubmed_xml(input, "receipt")?;
    assert_eq!(
        result
            .records
            .first()
            .ok_or("missing record")?
            .publication_year,
        Some(2024)
    );
    Ok(())
}

#[test]
fn document_dtd_does_not_enable_external_entities() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<?xml version='1.0'?><!DOCTYPE PubmedArticleSet [<!ENTITY external SYSTEM 'file:///unread-test-file'>]><PubmedArticleSet><PubmedArticle><PMID>1</PMID><ArticleTitle>&external;</ArticleTitle></PubmedArticle><PubmedArticle><PMID>2</PMID><ArticleTitle>Safe</ArticleTitle></PubmedArticle></PubmedArticleSet>";
    let result = import_pubmed_xml(input, "receipt")?;
    assert_eq!(result.quarantined.len(), 1);
    assert_eq!(result.records.len(), 1);
    assert_eq!(
        result.records.first().ok_or("missing record")?.native_id,
        "2"
    );
    Ok(())
}
