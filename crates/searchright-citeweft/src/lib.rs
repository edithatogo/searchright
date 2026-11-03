//! Optional one-way adapter from `CiteWeft`'s neutral extraction contracts.
//!
//! Dependency direction is deliberately `Searchright -> CiteWeft`. `CiteWeft`
//! does not depend on Searchright, Sourceright, CSL, screening or MCP types.

#![forbid(unsafe_code)]

use citeweft::{
    citeweft::{DiagnosticSeverity, ScholarlyDocument},
    reference_model::{ExtractionStatus, ReferenceModelReport, SourceSpan},
};
use searchright_contracts::{
    CitationCalloutEvidence, DOCUMENT_EVIDENCE_SCHEMA_VERSION, DocumentEvidence,
    DocumentExtractionProvenance, DocumentSpan, ExtractedFieldEvidence, ExtractedReferenceEvidence,
    ExtractionDiagnostic,
};
use sha2::{Digest, Sha256};

/// Convert `CiteWeft`'s full scholarly-document output into neutral Searchright evidence.
#[must_use]
pub fn from_scholarly_document(
    document_id: &str,
    document: &ScholarlyDocument,
) -> DocumentEvidence {
    let references = document
        .references
        .iter()
        .map(|reference| {
            let mut fields = Vec::new();
            if let Some(title) = &reference.title {
                fields.push(field("title", title));
            }
            for author in &reference.authors {
                fields.push(field("author", author));
            }
            if let Some(value) = &reference.container_title {
                fields.push(field("container_title", value));
            }
            if let Some(value) = &reference.publication_date {
                fields.push(field("publication_date", value));
            }
            if let Some(value) = &reference.volume {
                fields.push(field("volume", value));
            }
            if let Some(value) = &reference.issue {
                fields.push(field("issue", value));
            }
            if let Some(value) = &reference.pages {
                fields.push(field("pages", value));
            }
            for identifier in &reference.identifiers {
                fields.push(field(
                    &format!("identifier:{}", identifier.scheme),
                    &identifier.value,
                ));
            }
            ExtractedReferenceEvidence {
                reference_id: reference.id.clone(),
                raw_citation: reference.raw_text.clone(),
                fields,
                span: reference.span.as_ref().map(|span| DocumentSpan {
                    start_byte: None,
                    end_byte: None,
                    surface: span.surface.clone(),
                    source_id: span.source_id.clone(),
                    page: None,
                    bounding_box: Vec::new(),
                }),
                confidence: None,
                review_required: true,
            }
        })
        .collect();
    let diagnostics = document
        .diagnostics
        .iter()
        .map(|diagnostic| ExtractionDiagnostic {
            code: diagnostic.code.clone(),
            severity: match &diagnostic.severity {
                DiagnosticSeverity::Info => "info",
                DiagnosticSeverity::Warning => "warning",
                DiagnosticSeverity::Error => "error",
            }
            .to_owned(),
            message: diagnostic.message.clone(),
            subject_id: diagnostic.reference_id.clone(),
        })
        .collect();
    DocumentEvidence {
        schema_version: DOCUMENT_EVIDENCE_SCHEMA_VERSION.to_owned(),
        document_id: document_id.to_owned(),
        upstream_schema_version: document.schema_version.clone(),
        references,
        citation_callouts: Vec::new(),
        diagnostics,
        provenance: DocumentExtractionProvenance {
            backend: document.provenance.backend.clone(),
            backend_version: document.provenance.engine_version.clone(),
            configuration: document.provenance.configuration.clone(),
            input_sha256: document.provenance.input_hash.clone(),
            endpoint_class: document.provenance.endpoint_class.clone(),
            route_trace_digest: None,
        },
        canonical_write_permitted: false,
        retained_full_text: false,
    }
}

/// Convert `CiteWeft`'s deterministic reference/callout report into Searchright evidence.
#[must_use]
pub fn from_reference_model_report(
    document_id: &str,
    input_bytes: &[u8],
    report: &ReferenceModelReport,
) -> DocumentEvidence {
    let references = report
        .references
        .iter()
        .map(|reference| ExtractedReferenceEvidence {
            reference_id: reference.id.clone(),
            raw_citation: reference.raw_text.clone(),
            fields: reference_fields(reference),
            span: Some(source_span(&reference.span)),
            confidence: Some(reference.confidence),
            review_required: reference.status == ExtractionStatus::Review,
        })
        .collect();
    let citation_callouts = report
        .callouts
        .iter()
        .map(|callout| CitationCalloutEvidence {
            callout_id: callout.id.clone(),
            surface: callout.surface.clone(),
            span: source_span(&callout.span),
            reference_id: callout.reference_id.clone(),
            confidence: Some(callout.confidence),
            review_required: callout.status == ExtractionStatus::Review,
        })
        .collect();
    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| ExtractionDiagnostic {
            code: diagnostic.code.clone(),
            severity: "warning".to_owned(),
            message: diagnostic.message.clone(),
            subject_id: None,
        })
        .collect();
    DocumentEvidence {
        schema_version: DOCUMENT_EVIDENCE_SCHEMA_VERSION.to_owned(),
        document_id: document_id.to_owned(),
        upstream_schema_version: report.schema_version.clone(),
        references,
        citation_callouts,
        diagnostics,
        provenance: DocumentExtractionProvenance {
            backend: report.provenance.backend.clone(),
            backend_version: Some(report.provenance.version.clone()),
            configuration: report.provenance.configuration.clone(),
            input_sha256: Some(hex_digest(input_bytes)),
            endpoint_class: None,
            route_trace_digest: None,
        },
        canonical_write_permitted: false,
        retained_full_text: false,
    }
}

fn reference_fields(
    reference: &citeweft::reference_model::ReferenceCandidate,
) -> Vec<ExtractedFieldEvidence> {
    let mut values = Vec::new();
    for (name, evidence) in [
        ("authors", reference.fields.authors.as_ref()),
        ("title", reference.fields.title.as_ref()),
        ("year", reference.fields.year.as_ref()),
        ("doi", reference.fields.doi.as_ref()),
    ] {
        if let Some(evidence) = evidence {
            values.push(ExtractedFieldEvidence {
                field: name.to_owned(),
                value: evidence.value.clone(),
                span: Some(source_span(&evidence.span)),
            });
        }
    }
    values
}

fn field(name: &str, value: &str) -> ExtractedFieldEvidence {
    ExtractedFieldEvidence {
        field: name.to_owned(),
        value: value.to_owned(),
        span: None,
    }
}

fn source_span(span: &SourceSpan) -> DocumentSpan {
    DocumentSpan {
        start_byte: Some(span.start as u64),
        end_byte: Some(span.end as u64),
        surface: span.text.clone(),
        source_id: None,
        page: None,
        bounding_box: Vec::new(),
    }
}

fn hex_digest(input: &[u8]) -> String {
    Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use citeweft::{
        citeweft::{
            Diagnostic, DiagnosticSeverity, EngineProvenance, Identifier, ReferenceRecord,
            ScholarlyDocument, TextSpan, SCHEMA_VERSION as CITEWEFT_SCHEMA_VERSION,
        },
        reference_model::DeterministicReferenceModel,
    };
    use searchright_contracts::Validate;

    use super::*;

    #[test]
    fn deterministic_report_preserves_spans_and_requires_review_where_upstream_does() {
        let input = b"Body [1].\n\nReferences\n1. Smith J. 2024. Example title. 10.1000/test";
        let result = DeterministicReferenceModel::default().extract(input);
        let Ok(report) = result else {
            panic!("the deterministic CiteWeft fixture must extract successfully");
        };
        let evidence = from_reference_model_report("doc-1", input, &report);
        assert!(evidence.validate().is_ok());
        assert!(!evidence.canonical_write_permitted);
        assert!(!evidence.retained_full_text);
        assert_eq!(evidence.citation_callouts.len(), 1);
        assert_eq!(evidence.provenance.input_sha256, Some(hex_digest(input)));

        let Some(reference) = evidence.references.first() else {
            panic!("the deterministic CiteWeft fixture must contain a reference");
        };
        let Some(span) = reference.span.as_ref() else {
            panic!("the extracted reference must retain its source span");
        };
        assert_span_matches(input, span);

        let Some(callout) = evidence.citation_callouts.first() else {
            panic!("the deterministic CiteWeft fixture must contain a callout");
        };
        assert_span_matches(input, &callout.span);
    }

    #[test]
    fn scholarly_document_adapter_preserves_fields_spans_diagnostics_and_provenance() {
        let scholarly = ScholarlyDocument {
            schema_version: CITEWEFT_SCHEMA_VERSION.to_owned(),
            references: vec![ReferenceRecord {
                id: "ref-001".to_owned(),
                raw_text: "Doe J, Smith A. 2023. Advanced Search Methodologies. Journal of Testing, 12(3), 45-67. doi:10.1000/182".to_owned(),
                title: Some("Advanced Search Methodologies".to_owned()),
                authors: vec!["Doe J".to_owned(), "Smith A".to_owned()],
                container_title: Some("Journal of Testing".to_owned()),
                publication_date: Some("2023".to_owned()),
                volume: Some("12".to_owned()),
                issue: Some("3".to_owned()),
                pages: Some("45-67".to_owned()),
                identifiers: vec![Identifier {
                    scheme: "doi".to_owned(),
                    value: "10.1000/182".to_owned(),
                }],
                span: Some(TextSpan {
                    surface: "Doe J, Smith A. 2023.".to_owned(),
                    source_id: Some("grobid-p1-b2".to_owned()),
                }),
            }],
            diagnostics: vec![
                Diagnostic {
                    code: "INFO_NOTE".to_owned(),
                    severity: DiagnosticSeverity::Info,
                    message: "Info message".to_owned(),
                    reference_id: Some("ref-001".to_owned()),
                },
                Diagnostic {
                    code: "WARN_HEADER".to_owned(),
                    severity: DiagnosticSeverity::Warning,
                    message: "Warning message".to_owned(),
                    reference_id: None,
                },
                Diagnostic {
                    code: "ERR_UNRESOLVED".to_owned(),
                    severity: DiagnosticSeverity::Error,
                    message: "Error message".to_owned(),
                    reference_id: Some("ref-001".to_owned()),
                },
            ],
            provenance: EngineProvenance {
                backend: "grobid".to_owned(),
                engine_version: Some("0.8.0".to_owned()),
                configuration: "tei-header-and-citations".to_owned(),
                input_hash: Some(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned(),
                ),
                endpoint_class: Some("sandboxed_local".to_owned()),
            },
        };

        let evidence = from_scholarly_document("doc-scholarly-1", &scholarly);
        let Ok(()) = evidence.validate() else {
            panic!("scholarly document evidence must validate");
        };

        assert_eq!(evidence.document_id, "doc-scholarly-1");
        assert_eq!(evidence.upstream_schema_version, CITEWEFT_SCHEMA_VERSION);
        assert!(!evidence.canonical_write_permitted);
        assert!(!evidence.retained_full_text);
        assert_eq!(evidence.references.len(), 1);

        let Some(reference) = evidence.references.first() else {
            panic!("expected reference to be present");
        };
        assert_eq!(reference.reference_id, "ref-001");
        assert!(reference.review_required);
        assert_eq!(
            reference.raw_citation,
            "Doe J, Smith A. 2023. Advanced Search Methodologies. Journal of Testing, 12(3), 45-67. doi:10.1000/182"
        );

        let Some(span) = reference.span.as_ref() else {
            panic!("expected reference span to be present");
        };
        assert_eq!(span.surface, "Doe J, Smith A. 2023.");
        assert_eq!(span.source_id.as_deref(), Some("grobid-p1-b2"));
        assert!(span.start_byte.is_none());
        assert!(span.end_byte.is_none());

        let field_names: Vec<&str> = reference
            .fields
            .iter()
            .map(|f| f.field.as_str())
            .collect();
        assert_eq!(
            field_names,
            vec![
                "title",
                "author",
                "author",
                "container_title",
                "publication_date",
                "volume",
                "issue",
                "pages",
                "identifier:doi",
            ]
        );

        let doi_field = reference
            .fields
            .iter()
            .find(|f| f.field == "identifier:doi");
        let Some(doi) = doi_field else {
            panic!("expected identifier:doi field to be present");
        };
        assert_eq!(doi.value, "10.1000/182");

        assert_eq!(evidence.diagnostics.len(), 3);
        let Some(d0) = evidence.diagnostics.first() else {
            panic!("expected first diagnostic");
        };
        assert_eq!(d0.severity, "info");
        assert_eq!(d0.code, "INFO_NOTE");
        assert_eq!(d0.subject_id.as_deref(), Some("ref-001"));

        let Some(d1) = evidence.diagnostics.get(1) else {
            panic!("expected second diagnostic");
        };
        assert_eq!(d1.severity, "warning");
        assert_eq!(d1.code, "WARN_HEADER");
        assert!(d1.subject_id.is_none());

        let Some(d2) = evidence.diagnostics.get(2) else {
            panic!("expected third diagnostic");
        };
        assert_eq!(d2.severity, "error");
        assert_eq!(d2.code, "ERR_UNRESOLVED");
        assert_eq!(d2.subject_id.as_deref(), Some("ref-001"));

        assert_eq!(evidence.provenance.backend, "grobid");
        assert_eq!(evidence.provenance.backend_version.as_deref(), Some("0.8.0"));
        assert_eq!(evidence.provenance.configuration, "tei-header-and-citations");
        assert_eq!(
            evidence.provenance.input_sha256.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
        assert_eq!(
            evidence.provenance.endpoint_class.as_deref(),
            Some("sandboxed_local")
        );
    }

    #[test]
    fn adversarial_spans_diagnostics_and_write_permission_fail_closed() {
        let scholarly = ScholarlyDocument {
            schema_version: CITEWEFT_SCHEMA_VERSION.to_owned(),
            references: vec![ReferenceRecord {
                id: "ref-adv".to_owned(),
                raw_text: "Raw citation text".to_owned(),
                title: None,
                authors: Vec::new(),
                container_title: None,
                publication_date: None,
                volume: None,
                issue: None,
                pages: None,
                identifiers: Vec::new(),
                span: Some(TextSpan {
                    surface: "Raw citation text".to_owned(),
                    source_id: None,
                }),
            }],
            diagnostics: Vec::new(),
            provenance: EngineProvenance {
                backend: "adversarial-check".to_owned(),
                engine_version: None,
                configuration: "strict".to_owned(),
                input_hash: None,
                endpoint_class: None,
            },
        };

        let baseline = from_scholarly_document("doc-adv", &scholarly);
        let Ok(()) = baseline.validate() else {
            panic!("baseline evidence must validate");
        };

        // 1. Inverted span offsets (start >= end) must fail validation.
        let mut inverted_span_evidence = baseline.clone();
        let Some(ref_mut) = inverted_span_evidence.references.first_mut() else {
            panic!("expected reference to be present");
        };
        ref_mut.span = Some(DocumentSpan {
            start_byte: Some(100),
            end_byte: Some(50),
            surface: "inverted span".to_owned(),
            source_id: None,
            page: None,
            bounding_box: Vec::new(),
        });
        assert!(inverted_span_evidence.validate().is_err());

        // 2. Unpaired span byte offsets must fail validation.
        let mut unpaired_span_evidence = baseline.clone();
        let Some(ref_mut) = unpaired_span_evidence.references.first_mut() else {
            panic!("expected reference to be present");
        };
        ref_mut.span = Some(DocumentSpan {
            start_byte: Some(10),
            end_byte: None,
            surface: "unpaired span".to_owned(),
            source_id: None,
            page: None,
            bounding_box: Vec::new(),
        });
        assert!(unpaired_span_evidence.validate().is_err());

        // 3. Blank span surface must fail validation.
        let mut empty_span_evidence = baseline.clone();
        let Some(ref_mut) = empty_span_evidence.references.first_mut() else {
            panic!("expected reference to be present");
        };
        ref_mut.span = Some(DocumentSpan {
            start_byte: Some(0),
            end_byte: Some(5),
            surface: "   ".to_owned(),
            source_id: None,
            page: None,
            bounding_box: Vec::new(),
        });
        assert!(empty_span_evidence.validate().is_err());

        // 4. Unsupported diagnostic severity must fail closed.
        let mut invalid_diag_evidence = baseline.clone();
        invalid_diag_evidence.diagnostics.push(ExtractionDiagnostic {
            code: "ERR_CUSTOM".to_owned(),
            severity: "catastrophic".to_owned(),
            message: "unexpected severity".to_owned(),
            subject_id: None,
        });
        assert!(invalid_diag_evidence.validate().is_err());

        // 5. Permitting canonical write must fail closed.
        let mut write_permitted_evidence = baseline.clone();
        write_permitted_evidence.canonical_write_permitted = true;
        assert!(write_permitted_evidence.validate().is_err());

        // 6. Duplicate reference IDs must fail closed.
        let mut dup_ref_evidence = baseline;
        dup_ref_evidence.references.push(ExtractedReferenceEvidence {
            reference_id: "ref-adv".to_owned(),
            raw_citation: "Another citation".to_owned(),
            fields: Vec::new(),
            span: None,
            confidence: None,
            review_required: true,
        });
        assert!(dup_ref_evidence.validate().is_err());
    }

    fn assert_span_matches(input: &[u8], span: &DocumentSpan) {
        let (Some(start), Some(end)) = (span.start_byte, span.end_byte) else {
            panic!("the extracted span must retain byte offsets");
        };
        let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
            panic!("the extracted span offsets must fit the current platform");
        };
        let Some(surface) = input.get(start..end) else {
            panic!("the extracted span offsets must fall within the source document");
        };
        assert_eq!(surface, span.surface.as_bytes());
    }
}
