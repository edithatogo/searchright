//! Accessible deterministic diagnostics for human and machine consumers.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use searchright_contracts::{Diagnostic, DiagnosticLocale, DiagnosticSeverity, Validate};

/// Stable output representation for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticOutput {
    /// Plain text with no ANSI colour or terminal-dependent control sequences.
    PlainText,
    /// Pretty JSON.
    Json,
    /// One compact JSON document per line.
    JsonLines,
}

/// Localised human-readable fields for one stable diagnostic code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessage {
    /// Human-readable message in the catalogue locale.
    pub message: String,
    /// Optional corrective action in the catalogue locale.
    pub remediation: Option<String>,
}

/// An explicit, deterministic message catalogue for one locale.
///
/// Codes remain stable machine identifiers. A missing entry never silently
/// changes the source diagnostic: callers must choose whether fallback is
/// permitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessageCatalog {
    /// Locale supplied by every entry in this catalogue.
    pub locale: DiagnosticLocale,
    /// Messages keyed by stable diagnostic code.
    pub messages: BTreeMap<String, DiagnosticMessage>,
}

/// Policy applied when a requested catalogue lacks a diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingMessagePolicy {
    /// Preserve the source message and locale.
    PreserveSource,
    /// Reject rendering so incomplete translation coverage is visible.
    Reject,
}

/// Apply an explicit message catalogue without changing diagnostic semantics.
pub fn localize(
    diagnostics: &[Diagnostic],
    catalog: &DiagnosticMessageCatalog,
    missing: MissingMessagePolicy,
) -> Result<Vec<Diagnostic>, DiagnosticError> {
    let mut localized = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        diagnostic.validate()?;
        let Some(message) = catalog.messages.get(&diagnostic.code) else {
            match missing {
                MissingMessagePolicy::PreserveSource => {
                    localized.push(diagnostic.clone());
                    continue;
                }
                MissingMessagePolicy::Reject => {
                    return Err(DiagnosticError::MissingMessage(diagnostic.code.clone()));
                }
            }
        };
        let mut translated = diagnostic.clone();
        translated.message.clone_from(&message.message);
        translated.remediation.clone_from(&message.remediation);
        translated.locale = catalog.locale.clone();
        translated.validate()?;
        localized.push(translated);
    }
    Ok(localized)
}

/// Validate, sort and render diagnostics in a stable representation.
pub fn render(
    diagnostics: &[Diagnostic],
    output: DiagnosticOutput,
) -> Result<String, DiagnosticError> {
    let mut ordered = diagnostics.to_vec();
    for diagnostic in &ordered {
        diagnostic.validate()?;
    }
    ordered.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
    });
    match output {
        DiagnosticOutput::PlainText => Ok(render_plain(&ordered)),
        DiagnosticOutput::Json => Ok(serde_json::to_string_pretty(&ordered)?),
        DiagnosticOutput::JsonLines => {
            let mut document = String::new();
            for diagnostic in &ordered {
                document.push_str(&serde_json::to_string(diagnostic)?);
                document.push('\n');
            }
            Ok(document)
        }
    }
}

fn render_plain(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        output.push_str(severity_label(diagnostic.severity));
        output.push(' ');
        output.push_str(&diagnostic.code);
        output.push_str(": ");
        output.push_str(&diagnostic.message);
        if let Some(path) = diagnostic.path.as_deref() {
            output.push_str(" [");
            output.push_str(path);
            if let Some(line) = diagnostic.line {
                output.push(':');
                output.push_str(&line.to_string());
                if let Some(column) = diagnostic.column {
                    output.push(':');
                    output.push_str(&column.to_string());
                }
            }
            output.push(']');
        }
        output.push('\n');
        if let Some(remediation) = diagnostic.remediation.as_deref() {
            output.push_str("  remediation: ");
            output.push_str(remediation);
            output.push('\n');
        }
        if !diagnostic.evidence_ids.is_empty() {
            output.push_str("  evidence: ");
            output.push_str(&diagnostic.evidence_ids.join(", "));
            output.push('\n');
        }
        output.push_str("  locale: ");
        output.push_str(locale_label(&diagnostic.locale));
        output.push('\n');
    }
    output
}

const fn severity_label(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Information => "INFO",
        DiagnosticSeverity::Warning => "WARNING",
        DiagnosticSeverity::Error => "ERROR",
        DiagnosticSeverity::Blocking => "BLOCKING",
    }
}

fn locale_label(locale: &DiagnosticLocale) -> &str {
    match locale {
        DiagnosticLocale::EnAu => "en-AU",
        DiagnosticLocale::EnNz => "en-NZ",
        DiagnosticLocale::EnUs => "en-US",
        DiagnosticLocale::MiNz => "mi-NZ",
        DiagnosticLocale::Custom(value) => value,
    }
}

/// Diagnostic rendering failure.
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// JSON rendering failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A strict message catalogue did not contain the stable diagnostic code.
    #[error("message catalogue has no entry for diagnostic code `{0}`")]
    MissingMessage(String),
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{
        DIAGNOSTIC_SCHEMA_VERSION, Diagnostic, DiagnosticLocale, DiagnosticSeverity,
    };

    use super::*;

    #[test]
    fn plain_output_is_stable_and_contains_no_ansi_sequences() {
        let diagnostic = Diagnostic {
            schema_version: DIAGNOSTIC_SCHEMA_VERSION.to_owned(),
            code: "strategy.translation.review_required".to_owned(),
            severity: DiagnosticSeverity::Blocking,
            message: "Human translation review is required.".to_owned(),
            remediation: Some("Review each material dialect warning.".to_owned()),
            evidence_ids: vec!["strategy-1".to_owned()],
            path: Some("strategies/medline.json".to_owned()),
            line: Some(4),
            column: Some(2),
            locale: DiagnosticLocale::EnAu,
            blocking: true,
        };
        let rendered = render(&[diagnostic], DiagnosticOutput::PlainText);
        assert!(rendered.is_ok());
        if let Ok(rendered) = rendered {
            assert!(rendered.contains("BLOCKING strategy.translation.review_required"));
            assert!(!rendered.contains('\u{1b}'));
        }
    }

    fn sample(code: &str, severity: DiagnosticSeverity) -> Diagnostic {
        Diagnostic {
            schema_version: DIAGNOSTIC_SCHEMA_VERSION.to_owned(),
            code: code.to_owned(),
            severity,
            message: "Human translation review is required.".to_owned(),
            remediation: Some("Review each material dialect warning.".to_owned()),
            evidence_ids: vec!["strategy-1".to_owned()],
            path: Some("strategies/medline.json".to_owned()),
            line: Some(4),
            column: Some(2),
            locale: DiagnosticLocale::EnAu,
            blocking: severity == DiagnosticSeverity::Blocking,
        }
    }

    #[test]
    fn all_formats_are_deterministic_and_machine_formats_round_trip() {
        let diagnostics = vec![
            sample("strategy.warning", DiagnosticSeverity::Warning),
            sample("strategy.blocking", DiagnosticSeverity::Blocking),
        ];
        for output in [
            DiagnosticOutput::PlainText,
            DiagnosticOutput::Json,
            DiagnosticOutput::JsonLines,
        ] {
            let first = render(&diagnostics, output);
            let second = render(&diagnostics, output);
            assert_eq!(first.ok(), second.ok());
        }

        let json = render(&diagnostics, DiagnosticOutput::Json).unwrap_or_default();
        let decoded: Result<Vec<Diagnostic>, _> = serde_json::from_str(&json);
        assert!(matches!(decoded, Ok(items) if items.len() == 2));

        let jsonl = render(&diagnostics, DiagnosticOutput::JsonLines).unwrap_or_default();
        assert_eq!(jsonl.lines().count(), 2);
        assert!(
            jsonl
                .lines()
                .all(|line| serde_json::from_str::<Diagnostic>(line).is_ok())
        );
    }

    #[test]
    fn plain_output_uses_words_and_does_not_depend_on_terminal_width() {
        let rendered = render(
            &[sample("strategy.warning", DiagnosticSeverity::Warning)],
            DiagnosticOutput::PlainText,
        )
        .unwrap_or_default();
        assert!(rendered.starts_with("WARNING strategy.warning:"));
        assert!(rendered.contains("  remediation:"));
        assert!(!rendered.contains('\r'));
        assert!(
            !rendered
                .chars()
                .any(|character| matches!(character, '⚠' | '✗' | '✓'))
        );
    }

    #[test]
    fn catalogue_changes_only_human_fields_and_locale() {
        let source = sample("strategy.warning", DiagnosticSeverity::Warning);
        let catalog = DiagnosticMessageCatalog {
            locale: DiagnosticLocale::MiNz,
            messages: BTreeMap::from([(
                source.code.clone(),
                DiagnosticMessage {
                    message: "Me arotake tēnei whakamāoritanga.".to_owned(),
                    remediation: Some("Arotakengia ngā whakatūpato.".to_owned()),
                },
            )]),
        };
        let localized =
            localize(&[source.clone()], &catalog, MissingMessagePolicy::Reject).unwrap_or_default();
        assert_eq!(localized.len(), 1);
        let translated = &localized[0];
        assert_eq!(translated.code, source.code);
        assert_eq!(translated.severity, source.severity);
        assert_eq!(translated.blocking, source.blocking);
        assert_eq!(translated.evidence_ids, source.evidence_ids);
        assert_eq!(translated.locale, DiagnosticLocale::MiNz);
        assert_ne!(translated.message, source.message);
    }

    #[test]
    fn missing_catalogue_entries_are_explicit_or_preserve_source() {
        let source = sample("strategy.warning", DiagnosticSeverity::Warning);
        let catalog = DiagnosticMessageCatalog {
            locale: DiagnosticLocale::EnNz,
            messages: BTreeMap::new(),
        };
        assert!(matches!(
            localize(&[source.clone()], &catalog, MissingMessagePolicy::Reject),
            Err(DiagnosticError::MissingMessage(code)) if code == source.code
        ));
        assert!(matches!(
            localize(
                &[source.clone()],
                &catalog,
                MissingMessagePolicy::PreserveSource
            ),
            Ok(items) if items == vec![source]
        ));
    }
}
