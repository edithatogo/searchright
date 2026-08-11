//! Neutral, provenance-rich scholarly document evidence contracts.

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, DOCUMENT_EVIDENCE_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Source span preserved from a document-extraction backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DocumentSpan {
    /// Optional byte offset in the original source.
    pub start_byte: Option<u64>,
    /// Optional exclusive byte offset in the original source.
    pub end_byte: Option<u64>,
    /// Exact surface text represented by this span.
    pub surface: String,
    /// Optional backend/source identifier when offsets are unavailable.
    pub source_id: Option<String>,
    /// Optional one-based page number.
    pub page: Option<u32>,
    /// Optional backend-neutral bounding box `[x, y, width, height]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounding_box: Vec<f32>,
}

/// One extracted bibliographic field with source-grounded evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedFieldEvidence {
    /// Stable field name such as `title`, `author`, `year` or `doi`.
    pub field: String,
    /// Extracted field value; this is evidence, not canonical truth.
    pub value: String,
    /// Optional source span supporting this value.
    pub span: Option<DocumentSpan>,
}

/// One extracted reference candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedReferenceEvidence {
    /// Backend-stable candidate identifier.
    pub reference_id: String,
    /// Raw citation text, not whole-document text.
    pub raw_citation: String,
    /// Extracted field evidence.
    #[serde(default)]
    pub fields: Vec<ExtractedFieldEvidence>,
    /// Optional candidate-level source span.
    pub span: Option<DocumentSpan>,
    /// Optional calibrated confidence between zero and one.
    pub confidence: Option<f32>,
    /// Whether human review is required before downstream canonicalisation.
    pub review_required: bool,
}

/// One in-text citation callout and its possible reference link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CitationCalloutEvidence {
    /// Backend-stable callout identifier.
    pub callout_id: String,
    /// Exact callout surface.
    pub surface: String,
    /// Source span.
    pub span: DocumentSpan,
    /// Optional extracted-reference identifier.
    pub reference_id: Option<String>,
    /// Optional calibrated confidence between zero and one.
    pub confidence: Option<f32>,
    /// Whether human review is required.
    pub review_required: bool,
}

/// Typed diagnostic emitted by the extraction layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: String,
    /// `info`, `warning` or `error`.
    pub severity: String,
    /// Human-readable explanation.
    pub message: String,
    /// Optional reference/callout identifier.
    pub subject_id: Option<String>,
}

/// Provenance for one neutral extraction result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocumentExtractionProvenance {
    /// Backend name, for example `citeweft-deterministic-reference-model`.
    pub backend: String,
    /// Optional backend version.
    pub backend_version: Option<String>,
    /// Configuration string or fingerprint.
    pub configuration: String,
    /// SHA-256 digest of the input document when available.
    pub input_sha256: Option<String>,
    /// Optional endpoint class; never store credentials or sensitive URLs.
    pub endpoint_class: Option<String>,
    /// Optional routing-trace digest.
    pub route_trace_digest: Option<String>,
}

/// Neutral document evidence consumed by Searchright and optionally Sourceright.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DocumentEvidence {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable review-local document identifier.
    pub document_id: String,
    /// Upstream extraction contract version.
    pub upstream_schema_version: String,
    /// Extracted reference candidates.
    #[serde(default)]
    pub references: Vec<ExtractedReferenceEvidence>,
    /// Extracted citation callouts.
    #[serde(default)]
    pub citation_callouts: Vec<CitationCalloutEvidence>,
    /// Extraction diagnostics.
    #[serde(default)]
    pub diagnostics: Vec<ExtractionDiagnostic>,
    /// Backend and input provenance.
    pub provenance: DocumentExtractionProvenance,
    /// Explicit statement that extraction output is non-canonical evidence.
    pub canonical_write_permitted: bool,
    /// Explicit full-text retention status; defaults should be metadata-only.
    pub retained_full_text: bool,
}

fn validate_confidence(value: Option<f32>, field: &'static str) -> Result<(), ContractError> {
    if value.is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score)) {
        return Err(ContractError::Invariant(format!(
            "`{field}` must be finite and between zero and one"
        )));
    }
    Ok(())
}

impl Validate for DocumentSpan {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.surface, "document_span.surface")?;
        match (self.start_byte, self.end_byte) {
            (Some(start), Some(end)) if start >= end => {
                return Err(ContractError::Invariant(
                    "document span start must precede end".to_owned(),
                ));
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(ContractError::Invariant(
                    "document span byte offsets must be supplied together".to_owned(),
                ));
            }
            _ => {}
        }
        if !self.bounding_box.is_empty()
            && (self.bounding_box.len() != 4
                || self.bounding_box.iter().any(|value| !value.is_finite()))
        {
            return Err(ContractError::Invariant(
                "document span bounding_box must contain four finite values".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for DocumentEvidence {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DOCUMENT_EVIDENCE_SCHEMA_VERSION,
            "document_evidence.schema_version",
        )?;
        require_text(&self.document_id, "document_evidence.document_id")?;
        require_text(
            &self.upstream_schema_version,
            "document_evidence.upstream_schema_version",
        )?;
        require_text(
            &self.provenance.backend,
            "document_evidence.provenance.backend",
        )?;
        require_text(
            &self.provenance.configuration,
            "document_evidence.provenance.configuration",
        )?;
        if let Some(digest) = &self.provenance.input_sha256
            && (digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        {
            return Err(ContractError::Invariant(
                "document evidence input_sha256 must be 64 lowercase hexadecimal characters"
                    .to_owned(),
            ));
        }
        if self.canonical_write_permitted {
            return Err(ContractError::Invariant(
                "document evidence must never permit canonical writes".to_owned(),
            ));
        }
        let mut reference_ids = BTreeSet::new();
        for reference in &self.references {
            require_text(
                &reference.reference_id,
                "document_evidence.reference.reference_id",
            )?;
            require_text(
                &reference.raw_citation,
                "document_evidence.reference.raw_citation",
            )?;
            if !reference_ids.insert(reference.reference_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "duplicate document reference identifier `{}`",
                    reference.reference_id
                )));
            }
            validate_confidence(
                reference.confidence,
                "document_evidence.reference.confidence",
            )?;
            if let Some(span) = &reference.span {
                span.validate()?;
            }
            for field in &reference.fields {
                require_text(&field.field, "document_evidence.reference.field")?;
                require_text(&field.value, "document_evidence.reference.value")?;
                if let Some(span) = &field.span {
                    span.validate()?;
                }
            }
        }
        let mut callout_ids = BTreeSet::new();
        for callout in &self.citation_callouts {
            require_text(&callout.callout_id, "document_evidence.callout.callout_id")?;
            require_text(&callout.surface, "document_evidence.callout.surface")?;
            if !callout_ids.insert(callout.callout_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "duplicate citation-callout identifier `{}`",
                    callout.callout_id
                )));
            }
            callout.span.validate()?;
            validate_confidence(callout.confidence, "document_evidence.callout.confidence")?;
            if let Some(reference_id) = &callout.reference_id
                && !reference_ids.contains(reference_id.as_str())
            {
                return Err(ContractError::Invariant(format!(
                    "citation callout refers to unknown reference `{reference_id}`"
                )));
            }
        }
        for diagnostic in &self.diagnostics {
            require_text(&diagnostic.code, "document_evidence.diagnostic.code")?;
            require_text(
                &diagnostic.severity,
                "document_evidence.diagnostic.severity",
            )?;
            if !matches!(diagnostic.severity.as_str(), "info" | "warning" | "error") {
                return Err(ContractError::Invariant(format!(
                    "unsupported extraction diagnostic severity `{}`",
                    diagnostic.severity
                )));
            }
            require_text(&diagnostic.message, "document_evidence.diagnostic.message")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> DocumentEvidence {
        DocumentEvidence {
            schema_version: DOCUMENT_EVIDENCE_SCHEMA_VERSION.to_owned(),
            document_id: "doc-1".to_owned(),
            upstream_schema_version: "upstream.v1".to_owned(),
            references: Vec::new(),
            citation_callouts: Vec::new(),
            diagnostics: Vec::new(),
            provenance: DocumentExtractionProvenance {
                backend: "fixture".to_owned(),
                backend_version: Some("1".to_owned()),
                configuration: "deterministic".to_owned(),
                input_sha256: Some("a".repeat(64)),
                endpoint_class: None,
                route_trace_digest: None,
            },
            canonical_write_permitted: false,
            retained_full_text: false,
        }
    }

    #[test]
    fn rejects_unbounded_diagnostic_severity() {
        let mut value = evidence();
        value.diagnostics.push(ExtractionDiagnostic {
            code: "fixture".to_owned(),
            severity: "fatal".to_owned(),
            message: "fixture diagnostic".to_owned(),
            subject_id: None,
        });

        assert!(value.validate().is_err());
    }

    #[test]
    fn rejects_noncanonical_input_digest() {
        let mut value = evidence();
        value.provenance.input_sha256 = Some("A".repeat(64));

        assert!(value.validate().is_err());
    }
}
