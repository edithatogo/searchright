//! Comprehensive integration scenario tests for Track 27: Accessibility, internationalisation and usability.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use searchright_contracts::{
    DIAGNOSTIC_SCHEMA_VERSION, Diagnostic, DiagnosticLocale, DiagnosticSeverity,
};
use searchright_diagnostics::{
    DiagnosticError, DiagnosticMessage, DiagnosticMessageCatalog, DiagnosticOutput,
    MissingMessagePolicy, localize, render,
};

fn make_diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    message: &str,
    remediation: Option<&str>,
    path: Option<&str>,
    line: Option<u64>,
    column: Option<u64>,
    evidence_ids: Vec<&str>,
    locale: DiagnosticLocale,
) -> Diagnostic {
    let blocking = severity == DiagnosticSeverity::Blocking;
    Diagnostic {
        schema_version: DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        code: code.to_string(),
        severity,
        message: message.to_string(),
        remediation: remediation.map(ToString::to_string),
        path: path.map(ToString::to_string),
        line,
        column,
        evidence_ids: evidence_ids.into_iter().map(ToString::to_string).collect(),
        locale,
        blocking,
    }
}

#[test]
fn test_multiformat_accessible_rendering_plain_json_jsonl() {
    let d1 = make_diagnostic(
        "strategy.syntax.parenthesis_mismatch",
        DiagnosticSeverity::Blocking,
        "Unbalanced parenthesis in search strategy block.",
        Some("Add matching closing parenthesis."),
        Some("strategies/pubmed.medline"),
        Some(14),
        Some(22),
        vec!["syntax-tree-01", "parser-log-42"],
        DiagnosticLocale::EnAu,
    );
    let d2 = make_diagnostic(
        "dedup.field.doi_missing",
        DiagnosticSeverity::Warning,
        "Record lacks digital object identifier (DOI).",
        Some("Verify record metadata via Crossref or Europe PMC."),
        Some("records/batch-003.ris"),
        Some(108),
        None,
        vec!["dedup-manifest-03"],
        DiagnosticLocale::EnAu,
    );
    let d3 = make_diagnostic(
        "screening.prioritisation.active_learning_hint",
        DiagnosticSeverity::Information,
        "High relevance candidate surfaced by active learning model.",
        None,
        None,
        None,
        None,
        vec!["al-model-v1"],
        DiagnosticLocale::EnAu,
    );

    let diagnostics = vec![d2, d1, d3];

    // Format 1: PlainText
    let plain = match render(&diagnostics, DiagnosticOutput::PlainText) {
        Ok(p) => p,
        Err(e) => panic!("plain rendering must succeed: {e:?}"),
    };

    // Verify ANSI color-free invariant
    assert!(
        !plain.contains('\x1b'),
        "plain text must contain zero ANSI escape codes"
    );

    // Verify explicit text-based severity headers (sorted: BLOCKING first, then WARNING, then INFO)
    let lines: Vec<&str> = plain.lines().collect();
    assert!(
        lines.first().is_some_and(|l| l.starts_with("BLOCKING")),
        "first rendered diagnostic must be BLOCKING"
    );
    assert!(
        plain.contains("BLOCKING strategy.syntax.parenthesis_mismatch: Unbalanced parenthesis")
    );
    assert!(plain.contains("WARNING dedup.field.doi_missing: Record lacks digital object"));
    assert!(plain.contains("INFO screening.prioritisation.active_learning_hint: High relevance"));
    assert!(plain.contains("  remediation: Add matching closing parenthesis."));
    assert!(plain.contains("  evidence: syntax-tree-01, parser-log-42"));
    assert!(plain.contains("  locale: en-AU"));

    // Verify screen-reader & terminal-width safety: no unlabelled symbols
    for glyph in ['⚠', '✗', '✓', '⛔'] {
        assert!(
            !plain.contains(glyph),
            "plain text must not contain unlabelled glyph {glyph}"
        );
    }

    // Format 2: Pretty JSON
    let rendered_json_doc = match render(&diagnostics, DiagnosticOutput::Json) {
        Ok(j) => j,
        Err(e) => panic!("json rendering must succeed: {e:?}"),
    };
    let parsed: Vec<Diagnostic> = match serde_json::from_str(&rendered_json_doc) {
        Ok(p) => p,
        Err(e) => panic!("rendered json must parse back into Diagnostics: {e:?}"),
    };
    assert_eq!(parsed.len(), 3);
    assert_eq!(
        parsed.first().map(|d| d.severity),
        Some(DiagnosticSeverity::Blocking)
    );

    // Format 3: JSONLines
    let rendered_json_lines = match render(&diagnostics, DiagnosticOutput::JsonLines) {
        Ok(jl) => jl,
        Err(e) => panic!("jsonl rendering must succeed: {e:?}"),
    };
    let jsonl_lines: Vec<&str> = rendered_json_lines.lines().collect();
    assert_eq!(jsonl_lines.len(), 3);
    for line in jsonl_lines {
        let single: Diagnostic = match serde_json::from_str(line) {
            Ok(s) => s,
            Err(e) => panic!("each JSONL line must parse individually: {e:?}"),
        };
        assert!(!single.code.is_empty());
    }
}

#[test]
fn test_deterministic_sorting_across_collection_permutations() {
    let d1 = make_diagnostic(
        "a.code.blocking",
        DiagnosticSeverity::Blocking,
        "Critical issue",
        Some("Fix immediately"),
        None,
        None,
        None,
        vec!["ev-1"],
        DiagnosticLocale::EnAu,
    );
    let d2 = make_diagnostic(
        "b.code.error",
        DiagnosticSeverity::Error,
        "Error issue",
        None,
        None,
        None,
        None,
        vec!["ev-2"],
        DiagnosticLocale::EnAu,
    );
    let d3 = make_diagnostic(
        "c.code.warning",
        DiagnosticSeverity::Warning,
        "Warning issue",
        None,
        None,
        None,
        None,
        vec!["ev-3"],
        DiagnosticLocale::EnAu,
    );

    let list_alpha = vec![d1.clone(), d2.clone(), d3.clone()];
    let list_beta = vec![d3, d1, d2];

    let rendered_alpha = match render(&list_alpha, DiagnosticOutput::PlainText) {
        Ok(r) => r,
        Err(e) => panic!("render alpha failed: {e:?}"),
    };
    let rendered_beta = match render(&list_beta, DiagnosticOutput::PlainText) {
        Ok(r) => r,
        Err(e) => panic!("render beta failed: {e:?}"),
    };

    assert_eq!(
        rendered_alpha, rendered_beta,
        "rendering must be bitwise deterministic across permutations"
    );
}

#[test]
fn test_message_catalogue_localization_and_fallback_policies() {
    let d_original = make_diagnostic(
        "strategy.translation.review_required",
        DiagnosticSeverity::Blocking,
        "Human translation review is required.",
        Some("Review each material dialect warning."),
        Some("strategies/embase.ris"),
        Some(4),
        Some(1),
        vec!["ev-trans-01"],
        DiagnosticLocale::EnAu,
    );
    let d_untranslated = make_diagnostic(
        "ops.audit.retention_flag",
        DiagnosticSeverity::Warning,
        "Log retention window is approaching limit.",
        Some("Archive historical logs."),
        None,
        None,
        None,
        vec!["ev-audit-01"],
        DiagnosticLocale::EnAu,
    );

    let mut catalog_messages = BTreeMap::new();
    catalog_messages.insert(
        "strategy.translation.review_required".to_string(),
        DiagnosticMessage {
            message: "Me arotake tēnei whakamāoritanga e tētahi tangata.".to_string(),
            remediation: Some("Arotakengia ngā whakatūpato katoa mō ngā reo mita.".to_string()),
        },
    );

    let catalog_mi = DiagnosticMessageCatalog {
        locale: DiagnosticLocale::MiNz,
        messages: catalog_messages,
    };

    let set = vec![d_original, d_untranslated];

    // Case 1: MissingMessagePolicy::PreserveSource
    let preserved = match localize(&set, &catalog_mi, MissingMessagePolicy::PreserveSource) {
        Ok(p) => p,
        Err(e) => panic!("localize with preserve must succeed: {e:?}"),
    };
    assert_eq!(preserved.len(), 2);

    let Some(translated_step) = preserved.first() else {
        panic!("missing translated diagnostic");
    };
    assert_eq!(translated_step.code, "strategy.translation.review_required");
    assert_eq!(translated_step.locale, DiagnosticLocale::MiNz);
    assert_eq!(
        translated_step.message,
        "Me arotake tēnei whakamāoritanga e tētahi tangata."
    );
    assert_eq!(
        translated_step.remediation.as_deref(),
        Some("Arotakengia ngā whakatūpato katoa mō ngā reo mita.")
    );
    // Preserves stable machine fields
    assert_eq!(translated_step.severity, DiagnosticSeverity::Blocking);
    assert_eq!(
        translated_step.path.as_deref(),
        Some("strategies/embase.ris")
    );
    assert_eq!(translated_step.evidence_ids, vec!["ev-trans-01"]);

    let Some(untranslated_step) = preserved.get(1) else {
        panic!("missing untranslated diagnostic");
    };
    assert_eq!(untranslated_step.code, "ops.audit.retention_flag");
    assert_eq!(untranslated_step.locale, DiagnosticLocale::EnAu);
    assert_eq!(
        untranslated_step.message,
        "Log retention window is approaching limit."
    );

    // Case 2: MissingMessagePolicy::Reject
    match localize(&set, &catalog_mi, MissingMessagePolicy::Reject) {
        Err(DiagnosticError::MissingMessage(code)) => {
            assert_eq!(code, "ops.audit.retention_flag");
        }
        other => panic!("expected MissingMessage error, got: {other:?}"),
    }
}

#[test]
fn test_contract_validation_fails_closed_on_invalid_data() {
    // 1. Invalid uppercase code
    let mut invalid = make_diagnostic(
        "INVALID.CODE.UPPERCASE",
        DiagnosticSeverity::Warning,
        "Test message",
        None,
        None,
        None,
        None,
        vec!["ev-1"],
        DiagnosticLocale::EnAu,
    );
    assert!(render(&[invalid.clone()], DiagnosticOutput::PlainText).is_err());

    // 2. Blocking severity requires blocking = true
    invalid.code = "valid.code.namespaced".to_string();
    invalid.severity = DiagnosticSeverity::Blocking;
    invalid.blocking = false; // mismatch!
    assert!(render(&[invalid.clone()], DiagnosticOutput::PlainText).is_err());

    // 3. Column without line fails validation
    invalid.severity = DiagnosticSeverity::Warning;
    invalid.blocking = false;
    invalid.line = None;
    invalid.column = Some(10);
    assert!(render(&[invalid.clone()], DiagnosticOutput::PlainText).is_err());

    // 4. Zero line number fails validation (one-based)
    invalid.line = Some(0);
    invalid.column = None;
    assert!(render(&[invalid.clone()], DiagnosticOutput::PlainText).is_err());

    // 5. Custom locale without BCP 47 hyphen fails validation
    invalid.line = Some(1);
    invalid.locale = DiagnosticLocale::Custom("invalidlocale".to_string());
    assert!(render(&[invalid], DiagnosticOutput::PlainText).is_err());
}
