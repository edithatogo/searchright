//! Render ordering and localisation edge cases.

use std::collections::BTreeMap;

use searchright_contracts::{
    DIAGNOSTIC_SCHEMA_VERSION, Diagnostic, DiagnosticLocale, DiagnosticSeverity,
};
use searchright_diagnostics::{
    DiagnosticError, DiagnosticMessage, DiagnosticMessageCatalog, DiagnosticOutput,
    MissingMessagePolicy, localize, render,
};

fn diag(code: &str, severity: DiagnosticSeverity, message: &str) -> Diagnostic {
    let blocking = severity == DiagnosticSeverity::Blocking;
    Diagnostic {
        schema_version: DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        code: code.to_string(),
        severity,
        message: message.to_string(),
        remediation: None,
        evidence_ids: vec![],
        path: None,
        line: None,
        column: None,
        locale: DiagnosticLocale::EnAu,
        blocking,
    }
}

fn catalog(locale: DiagnosticLocale, entries: Vec<(&str, &str)>) -> DiagnosticMessageCatalog {
    DiagnosticMessageCatalog {
        locale,
        messages: entries
            .into_iter()
            .map(|(code, message)| {
                (
                    code.to_string(),
                    DiagnosticMessage {
                        message: message.to_string(),
                        remediation: None,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn render_sorts_by_severity_then_code() {
    let diagnostics = vec![
        diag("aaa.info", DiagnosticSeverity::Information, "info"),
        diag("zzz.error", DiagnosticSeverity::Error, "error"),
        diag("aaa.error", DiagnosticSeverity::Error, "error a"),
    ];
    let rendered = render(&diagnostics, DiagnosticOutput::PlainText)
        .unwrap_or_else(|_| panic!("expected render"));
    let lines: Vec<&str> = rendered.lines().collect();
    // Errors first, tie-break by code ascending: aaa.error, zzz.error, then info.
    assert!(
        lines
            .first()
            .unwrap_or_else(|| panic!("expected line"))
            .contains("aaa.error")
    );
    assert!(
        lines
            .get(2)
            .unwrap_or_else(|| panic!("expected line"))
            .contains("zzz.error")
    );
    assert!(
        lines
            .get(4)
            .unwrap_or_else(|| panic!("expected line"))
            .contains("aaa.info")
    );
}

#[test]
fn render_json_is_pretty_and_roundtrips_code_order() {
    let diagnostics = vec![diag("aaa.warn", DiagnosticSeverity::Warning, "warn")];
    let rendered =
        render(&diagnostics, DiagnosticOutput::Json).unwrap_or_else(|_| panic!("expected render"));
    assert!(rendered.contains('\n'));
    assert!(rendered.contains("aaa.warn"));
}

#[test]
fn render_json_lines_ends_each_document_with_newline() {
    let diagnostics = vec![
        diag("aaa.info", DiagnosticSeverity::Information, "one"),
        diag("bbb.info", DiagnosticSeverity::Information, "two"),
    ];
    let rendered = render(&diagnostics, DiagnosticOutput::JsonLines)
        .unwrap_or_else(|_| panic!("expected render"));
    assert!(rendered.ends_with('\n'));
    assert_eq!(rendered.lines().count(), 2);
}

#[test]
fn render_rejects_invalid_diagnostic_contract() {
    let mut bad = diag("aaa.info", DiagnosticSeverity::Information, "one");
    bad.schema_version = "org.searchright.diagnostic.v99".to_string();
    assert!(matches!(
        render(&[bad], DiagnosticOutput::PlainText),
        Err(DiagnosticError::Contract(_))
    ));
}

#[test]
fn render_rejects_uppercase_code() {
    let bad = diag("AAA.INFO", DiagnosticSeverity::Information, "one");
    assert!(render(&[bad], DiagnosticOutput::PlainText).is_err());
}

#[test]
fn render_plain_includes_path_line_column_evidence_and_locales() {
    let mut d = diag("aaa.warn", DiagnosticSeverity::Warning, "warn");
    d.path = Some("src/lib.rs".to_string());
    d.line = Some(10);
    d.column = Some(3);
    d.evidence_ids = vec!["ev-1".to_string(), "ev-2".to_string()];
    d.locale = DiagnosticLocale::MiNz;
    let rendered =
        render(&[d], DiagnosticOutput::PlainText).unwrap_or_else(|_| panic!("expected render"));
    assert!(rendered.contains("[src/lib.rs:10:3]"));
    assert!(rendered.contains("evidence: ev-1, ev-2"));
    assert!(rendered.contains("locale: mi-NZ"));
    assert!(rendered.contains("WARNING"));
}

#[test]
fn render_plain_omits_column_when_only_line_present() {
    let mut d = diag("aaa.warn", DiagnosticSeverity::Warning, "warn");
    d.path = Some("src/lib.rs".to_string());
    d.line = Some(7);
    let rendered =
        render(&[d], DiagnosticOutput::PlainText).unwrap_or_else(|_| panic!("expected render"));
    assert!(rendered.contains("[src/lib.rs:7]"));
}

#[test]
fn localize_translates_and_switches_locale() {
    let diagnostics = vec![diag("aaa.warn", DiagnosticSeverity::Warning, "warn")];
    let cat = catalog(
        DiagnosticLocale::EnNz,
        vec![("aaa.warn", "translated warning")],
    );
    let localized = localize(&diagnostics, &cat, MissingMessagePolicy::Reject)
        .unwrap_or_else(|_| panic!("expected localize"));
    let first = localized
        .first()
        .unwrap_or_else(|| panic!("expected diagnostic"));
    assert_eq!(first.message, "translated warning");
    assert_eq!(first.locale, DiagnosticLocale::EnNz);
}

#[test]
fn localize_preserve_source_keeps_missing_codes() {
    let diagnostics = vec![diag("aaa.warn", DiagnosticSeverity::Warning, "warn")];
    let cat = catalog(DiagnosticLocale::EnNz, vec![]);
    let localized = localize(&diagnostics, &cat, MissingMessagePolicy::PreserveSource)
        .unwrap_or_else(|_| panic!("expected localize"));
    let first = localized
        .first()
        .unwrap_or_else(|| panic!("expected diagnostic"));
    assert_eq!(first.message, "warn");
    assert_eq!(first.locale, DiagnosticLocale::EnAu);
}

#[test]
fn localize_reject_policy_errors_on_missing_code() {
    let diagnostics = vec![diag("aaa.warn", DiagnosticSeverity::Warning, "warn")];
    let cat = catalog(DiagnosticLocale::EnNz, vec![]);
    assert!(matches!(
        localize(&diagnostics, &cat, MissingMessagePolicy::Reject),
        Err(DiagnosticError::MissingMessage(code)) if code == "aaa.warn"
    ));
}

#[test]
fn localize_rejects_invalid_source_diagnostic() {
    let mut bad = diag("aaa.warn", DiagnosticSeverity::Warning, "warn");
    bad.code = "AAA.WARN".to_string();
    let cat = catalog(DiagnosticLocale::EnNz, vec![("AAA.WARN", "translated")]);
    assert!(matches!(
        localize(&[bad], &cat, MissingMessagePolicy::PreserveSource),
        Err(DiagnosticError::Contract(_))
    ));
}

#[test]
fn empty_diagnostics_render_empty_documents() {
    assert_eq!(
        render(&[], DiagnosticOutput::PlainText).unwrap_or_else(|_| panic!("expected render")),
        String::new()
    );
    assert_eq!(
        render(&[], DiagnosticOutput::JsonLines).unwrap_or_else(|_| panic!("expected render")),
        String::new()
    );
    let json = render(&[], DiagnosticOutput::Json).unwrap_or_else(|_| panic!("expected render"));
    assert_eq!(json, "[]");
}

#[test]
fn custom_locale_label_renders_tag() {
    let mut d = diag("aaa.warn", DiagnosticSeverity::Warning, "warn");
    d.locale = DiagnosticLocale::Custom("fr-CA".to_string());
    let rendered =
        render(&[d], DiagnosticOutput::PlainText).unwrap_or_else(|_| panic!("expected render"));
    assert!(rendered.contains("locale: fr-CA"));
}
