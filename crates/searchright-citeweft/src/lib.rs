//! Optional one-way adapter from CiteWeft's neutral extraction contracts.
//!
//! Dependency direction is deliberately `Searchright -> CiteWeft`. CiteWeft
//! does not depend on Searchright, Sourceright, CSL, screening or MCP types.

#![forbid(unsafe_code)]

use citeweft::{
    citeweft::{DiagnosticSeverity, ScholarlyDocument},
    reference_model::{ExtractionStatus, ReferenceModelReport, SourceSpan},
};
use searchright_contracts::{
    CitationCalloutEvidence, DocumentEvidence, DocumentExtractionProvenance, DocumentSpan,
    ExtractedFieldEvidence, ExtractedReferenceEvidence, ExtractionDiagnostic,
    DOCUMENT_EVIDENCE_SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};

/// Convert CiteWeft's full scholarly-document output into neutral Searchright evidence.
#[must_use]
pub fn from_scholarly_document(document_id: &str, document: &ScholarlyDocument) -> DocumentEvidence {
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
                fields.push(field(&format!("identifier:{}", identifier.scheme), &identifier.value));
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

/// Convert CiteWeft's deterministic reference/callout report into Searchright evidence.
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

fn reference_fields(reference: &citeweft::reference_model::ReferenceCandidate) -> Vec<ExtractedFieldEvidence> {
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
    format!("{:x}", Sha256::digest(input))
}

#[cfg(test)]
mod tests {
    use citeweft::reference_model::DeterministicReferenceModel;
    use searchright_contracts::Validate;

    use super::*;

    #[test]
    fn deterministic_report_preserves_spans_and_requires_review_where_upstream_does() {
        let input = b"Body [1].\n\nReferences\n1. Smith J. 2024. Example title. 10.1000/test";
        let result = DeterministicReferenceModel::default().extract(input);
        assert!(result.is_ok());
        if let Ok(report) = result {
            let evidence = from_reference_model_report("doc-1", input, &report);
            assert!(evidence.validate().is_ok());
            assert!(!evidence.canonical_write_permitted);
            assert!(!evidence.retained_full_text);
            assert_eq!(evidence.citation_callouts.len(), 1);
            assert!(evidence.references.first().is_some_and(|value| value.span.is_some()));
        }
    }
}
