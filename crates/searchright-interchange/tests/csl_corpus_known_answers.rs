//! Independent, synthetic known-answer checks for the Track 40.1 CSL identity boundary.
//!
//! Intentionally does not change the implementation or claim CSL schema,
//! source-rights, live-provider or downstream-consumer conformance.
//! Pinned digest inputs are tiny and independently calculated from the
//! documented domain prefix and canonical ASCII/UTF-8 payload bytes.

use searchright_interchange::csl_corpus::{CslCorpusError, canonical_csl_bibliography};

#[test]
fn empty_bibliography_matches_independent_blake3_known_answer() -> Result<(), CslCorpusError> {
    let output = canonical_csl_bibliography("[]")?;
    assert!(output.references.is_empty());
    assert_eq!(
        output.revision,
        "blake3:bf238d126a29605d14feea5c9c4e7a572e349723defe8101cfc7df6c537247e5"
    );
    Ok(())
}

#[test]
fn minimal_bibliography_matches_independent_blake3_known_answer() -> Result<(), CslCorpusError> {
    // Hashed bytes: b"searchright/csl-bibliography/v1\n" followed by
    // b'[{"id":"a","type":"report"}]' (no terminator).
    let output = canonical_csl_bibliography(r#"[{"type":"report","id":"a"}]"#)?;
    assert_eq!(
        output.revision,
        "blake3:298900b34fce5da126e622698ac294be6041c535220e1bfc676f626d69510962"
    );
    assert_eq!(
        output.references,
        vec![serde_json::json!({"id": "a", "type": "report"})]
    );
    Ok(())
}

#[test]
fn unicode_and_control_escapes_have_stable_distinct_revisions() -> Result<(), CslCorpusError> {
    let escaped = canonical_csl_bibliography(
        r#"[{"type":"report","title":"Caf\u00e9\n\"review\"","id":"unicode"}]"#,
    )?;
    let literal = canonical_csl_bibliography(
        r#"[{"id":"unicode","title":"Café\n\"review\"","type":"report"}]"#,
    )?;
    assert_eq!(escaped, literal);
    assert_eq!(
        escaped.revision,
        "blake3:17823b33cd008aaa329f9c14eee4a02eb7f6748336aacc1ada7356f06550517e"
    );
    assert_eq!(
        escaped.references,
        vec![serde_json::json!({"id":"unicode","title":"Caf\u{e9}\n\"review\"","type":"report"})]
    );

    // Preserve decomposed Unicode; the identity boundary does not normalise text.
    let decomposed = canonical_csl_bibliography(
        r#"[{"id":"unicode","title":"Cafe\u0301\n\"review\"","type":"report"}]"#,
    )?;
    assert_eq!(
        decomposed.revision,
        "blake3:eec6ef72a9188fc50aacc1ceeeec9ae2618cf21781abb61ce935b158471f2301"
    );
    assert_ne!(escaped.revision, decomposed.revision);
    assert_eq!(
        decomposed.references,
        vec![serde_json::json!({"id":"unicode","title":"Cafe\u{301}\n\"review\"","type":"report"})]
    );
    Ok(())
}

#[test]
fn nested_order_invariance_preserves_fields() -> Result<(), CslCorpusError> {
    let first = r#"[{"type":"report","id":"z","custom":{"beta":2,"alpha":1},"author":[{"given":"Ana","family":"Lee"},{"literal":"Collective"}]},{"id":"a","type":"article-journal","issued":{"date-parts":[[2025,3,7]]}}]"#;
    let second = r#"[{"issued":{"date-parts":[[2025,3,7]]},"type":"article-journal","id":"a"},{"author":[{"family":"Lee","given":"Ana"},{"literal":"Collective"}],"custom":{"alpha":1,"beta":2},"id":"z","type":"report"}]"#;
    let a = canonical_csl_bibliography(first)?;
    let b = canonical_csl_bibliography(second)?;
    assert_eq!(a.revision, b.revision);
    assert_eq!(a.references, b.references);
    assert_eq!(
        a.references,
        vec![
            serde_json::json!({"id":"a","type":"article-journal","issued":{"date-parts":[[2025,3,7]]}}),
            serde_json::json!({"id":"z","type":"report","custom":{"alpha":1,"beta":2},"author":[{"family":"Lee","given":"Ana"},{"literal":"Collective"}]})
        ]
    );
    Ok(())
}

#[test]
fn nested_array_order_is_part_of_bibliography_revision() -> Result<(), CslCorpusError> {
    let one = canonical_csl_bibliography(
        r#"[{"id":"a","type":"report","author":[{"family":"Lee"},{"family":"Doe"}]}]"#,
    )?;
    let reversed = canonical_csl_bibliography(
        r#"[{"id":"a","type":"report","author":[{"family":"Doe"},{"family":"Lee"}]}]"#,
    )?;
    assert_ne!(one.revision, reversed.revision);
    Ok(())
}

#[test]
fn malformed_identity_errors_have_precise_second_item_index() {
    assert!(matches!(
        canonical_csl_bibliography(r#"[{"id":"ok","type":"report"},{"id":7,"type":"report"}]"#),
        Err(CslCorpusError::InvalidId(1))
    ));
    assert!(matches!(
        canonical_csl_bibliography(r#"[{"id":"ok","type":"report"},{"id":"next","type":null}]"#),
        Err(CslCorpusError::InvalidType(1))
    ));
    assert!(matches!(
        canonical_csl_bibliography(r#"[{"id":"ok","type":"report"},{"id":"ok","type":"article-journal"}]"#),
        Err(CslCorpusError::DuplicateId(id)) if id == "ok"
    ));
    assert!(matches!(
        canonical_csl_bibliography(r#"[{"id":"ok","type":"report"},false]"#),
        Err(CslCorpusError::NotObject(1))
    ));
}

#[test]
fn full_schema_validation_is_not_claimed() -> Result<(), CslCorpusError> {
    let output = canonical_csl_bibliography(r#"[{"id":"a","type":"synthetic-unsupported-type"}]"#)?;
    assert_eq!(
        output.references,
        vec![serde_json::json!({"id": "a", "type": "synthetic-unsupported-type"})]
    );
    Ok(())
}
