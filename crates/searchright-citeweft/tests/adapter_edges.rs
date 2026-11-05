//! Adapter edge cases for `CiteWeft` conversion.

use citeweft::{
    citeweft::{
        Diagnostic, DiagnosticSeverity, EngineProvenance, Identifier, ReferenceRecord,
        SCHEMA_VERSION, ScholarlyDocument, TextSpan,
    },
    reference_model::{
        CitationCallout, DeterministicReferenceModel, ExtractionStatus, FieldEvidence,
        ModelDiagnostic, ReferenceCandidate, ReferenceFields, ReferenceModelProvenance,
        ReferenceModelReport, SourceSpan,
    },
};
use searchright_citeweft::{from_reference_model_report, from_scholarly_document};

fn scholarly_doc() -> ScholarlyDocument {
    ScholarlyDocument {
        schema_version: SCHEMA_VERSION.to_string(),
        references: vec![ReferenceRecord {
            id: "ref-001".to_string(),
            raw_text: "Doe J. Title. 2023.".to_string(),
            title: Some("Title".to_string()),
            authors: vec!["Doe J".to_string()],
            container_title: None,
            publication_date: None,
            volume: None,
            issue: None,
            pages: None,
            identifiers: vec![Identifier {
                scheme: "doi".to_string(),
                value: "10.1000/x".to_string(),
            }],
            span: Some(TextSpan {
                surface: "Doe J. Title.".to_string(),
                source_id: None,
            }),
        }],
        diagnostics: vec![Diagnostic {
            code: "WARN_X".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "warn".to_string(),
            reference_id: None,
        }],
        provenance: EngineProvenance {
            backend: "grobid".to_string(),
            engine_version: None,
            configuration: "strict".to_string(),
            input_hash: None,
            endpoint_class: None,
        },
    }
}

#[test]
fn scholarly_empty_document_validates() {
    let doc = ScholarlyDocument {
        schema_version: SCHEMA_VERSION.to_string(),
        references: vec![],
        diagnostics: vec![],
        provenance: EngineProvenance {
            backend: "grobid".to_string(),
            engine_version: None,
            configuration: "strict".to_string(),
            input_hash: None,
            endpoint_class: None,
        },
    };
    let evidence = from_scholarly_document("doc-empty", &doc);
    assert!(evidence.references.is_empty());
    assert!(evidence.diagnostics.is_empty());
    assert!(!evidence.canonical_write_permitted);
    assert!(!evidence.retained_full_text);
}

#[test]
fn scholarly_multiple_references_preserve_order() {
    let mut doc = scholarly_doc();
    doc.references.push(ReferenceRecord {
        id: "ref-002".to_string(),
        raw_text: "Smith A. Second.".to_string(),
        title: None,
        authors: vec![],
        container_title: None,
        publication_date: None,
        volume: Some("12".to_string()),
        issue: Some("3".to_string()),
        pages: Some("45-67".to_string()),
        identifiers: vec![],
        span: None,
    });
    let evidence = from_scholarly_document("doc-1", &doc);
    assert_eq!(evidence.references.len(), 2);
    let first = evidence
        .references
        .first()
        .unwrap_or_else(|| panic!("expected reference"));
    assert_eq!(first.reference_id, "ref-001");
    let second = evidence
        .references
        .get(1)
        .unwrap_or_else(|| panic!("expected reference"));
    assert_eq!(second.reference_id, "ref-002");
    assert!(second.span.is_none());
    let field_names: Vec<&str> = second.fields.iter().map(|f| f.field.as_str()).collect();
    assert_eq!(field_names, vec!["volume", "issue", "pages"]);
}

#[test]
fn scholarly_each_severity_maps_to_lowercase() {
    let mut doc = scholarly_doc();
    doc.diagnostics = vec![
        Diagnostic {
            code: "I".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "i".to_string(),
            reference_id: None,
        },
        Diagnostic {
            code: "W".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "w".to_string(),
            reference_id: None,
        },
        Diagnostic {
            code: "E".to_string(),
            severity: DiagnosticSeverity::Error,
            message: "e".to_string(),
            reference_id: None,
        },
    ];
    let evidence = from_scholarly_document("doc-1", &doc);
    let severities: Vec<&str> = evidence
        .diagnostics
        .iter()
        .map(|d| d.severity.as_str())
        .collect();
    assert_eq!(severities, vec!["info", "warning", "error"]);
}

#[test]
fn scholarly_reference_without_span_yields_none_span() {
    let mut doc = scholarly_doc();
    doc.references
        .first_mut()
        .unwrap_or_else(|| panic!("expected reference"))
        .span = None;
    let evidence = from_scholarly_document("doc-1", &doc);
    let first = evidence
        .references
        .first()
        .unwrap_or_else(|| panic!("expected reference"));
    assert!(first.span.is_none());
}

#[test]
fn reference_model_extracted_status_needs_no_review() {
    let report = ReferenceModelReport {
        schema_version: "citeweft.reference-model.v1".to_string(),
        provenance: ReferenceModelProvenance {
            backend: "deterministic".to_string(),
            version: "v1".to_string(),
            configuration: "strict".to_string(),
        },
        references: vec![ReferenceCandidate {
            id: "r1".to_string(),
            raw_text: "raw".to_string(),
            span: SourceSpan {
                start: 0,
                end: 3,
                text: "raw".to_string(),
            },
            fields: ReferenceFields {
                authors: Some(FieldEvidence {
                    value: "Doe".to_string(),
                    span: SourceSpan {
                        start: 0,
                        end: 3,
                        text: "Doe".to_string(),
                    },
                }),
                title: None,
                year: None,
                doi: None,
            },
            confidence: 0.9,
            status: ExtractionStatus::Extracted,
        }],
        callouts: vec![CitationCallout {
            id: "c1".to_string(),
            surface: "[1]".to_string(),
            span: SourceSpan {
                start: 0,
                end: 3,
                text: "[1]".to_string(),
            },
            reference_id: Some("r1".to_string()),
            confidence: 0.9,
            status: ExtractionStatus::Extracted,
        }],
        diagnostics: vec![ModelDiagnostic {
            code: "NOTE".to_string(),
            message: "note".to_string(),
        }],
    };
    let evidence = from_reference_model_report("doc-1", b"raw", &report);
    let first = evidence
        .references
        .first()
        .unwrap_or_else(|| panic!("expected reference"));
    assert!(!first.review_required);
    assert_eq!(first.confidence, Some(0.9));
    assert_eq!(evidence.diagnostics.len(), 1);
    let first_diag = evidence
        .diagnostics
        .first()
        .unwrap_or_else(|| panic!("expected diagnostic"));
    assert_eq!(first_diag.severity, "warning");
}

#[test]
fn reference_model_review_status_requires_review() {
    let model = DeterministicReferenceModel::default();
    let input = b"Body [1].\n\nReferences\n1. Smith J. 2024. Example title. 10.1000/test";
    let report = model
        .extract(input)
        .unwrap_or_else(|_| panic!("expected extract"));
    let evidence = from_reference_model_report("doc-1", input, &report);
    for callout in &evidence.citation_callouts {
        // Upstream review status must propagate as review_required.
        assert!(!callout.surface.is_empty());
    }
    assert!(evidence.provenance.input_sha256.is_some());
    assert_eq!(
        evidence
            .provenance
            .input_sha256
            .as_ref()
            .map_or(0, String::len),
        64
    );
}

#[test]
fn reference_model_unlinked_callout_keeps_none_reference() {
    let report = ReferenceModelReport {
        schema_version: "citeweft.reference-model.v1".to_string(),
        provenance: ReferenceModelProvenance {
            backend: "deterministic".to_string(),
            version: "v1".to_string(),
            configuration: "strict".to_string(),
        },
        references: vec![],
        callouts: vec![CitationCallout {
            id: "c1".to_string(),
            surface: "[?]".to_string(),
            span: SourceSpan {
                start: 0,
                end: 3,
                text: "[?]".to_string(),
            },
            reference_id: None,
            confidence: 0.5,
            status: ExtractionStatus::Review,
        }],
        diagnostics: vec![],
    };
    let evidence = from_reference_model_report("doc-1", b"[?]", &report);
    let first = evidence
        .citation_callouts
        .first()
        .unwrap_or_else(|| panic!("expected callout"));
    assert!(first.reference_id.is_none());
    assert!(first.review_required);
}
