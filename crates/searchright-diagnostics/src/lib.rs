//! Accessible deterministic diagnostics for human and machine consumers.

#![forbid(unsafe_code)]

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
}
