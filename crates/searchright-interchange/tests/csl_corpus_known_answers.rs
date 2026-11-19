//! Independent, synthetic known-answer checks for the Track 40.1 CSL identity boundary.
//!
//! Intentionally does not change the implementation or claim CSL schema,
//! source-rights, live-provider or downstream-consumer conformance.
//! Pinned digest inputs are tiny, ASCII-only and independently calculated
//! from the documented domain prefix and BLAKE3 test-vector algorithm.

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
    // Hashed bytes: b"searchright/csl-bibliography/v1\\n" followed by
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
fn nested_key_and_record_permutation_preserve_revision_and_full_fields()
-> Result<(), CslCorpusError> {
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
fn identity_validation_does_not_pretend_to_be_a_full_csl_schema_validator()
-> Result<(), CslCorpusError> {
    let output = canonical_csl_bibliography(r#"[{"id":"a","type":"synthetic-unsupported-type"}]"#)?;
    assert_eq!(
        output.references,
        vec![serde_json::json!({"id": "a", "type": "synthetic-unsupported-type"})]
    );
    Ok(())
}
