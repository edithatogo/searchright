use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, DIAGNOSTIC_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Severity assigned to a stable Searchright diagnostic.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Informational context that does not require action.
    Information,
    /// A potential issue that should be reviewed.
    Warning,
    /// An operation failed but the wider review may continue.
    Error,
    /// A governance or methodological condition prevents progression.
    Blocking,
}

/// Locale used for a human-readable diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLocale {
    /// Australian English.
    EnAu,
    /// New Zealand English.
    EnNz,
    /// United States English.
    EnUs,
    /// Māori. Translation coverage must be declared by the caller.
    MiNz,
    /// BCP 47 language tag not represented by a built-in variant.
    Custom(String),
}

/// Stable, accessible diagnostic shared by CLI, MCP and library consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable namespaced diagnostic code.
    pub code: String,
    /// Severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable message.
    pub message: String,
    /// Actionable remediation.
    pub remediation: Option<String>,
    /// Evidence identifiers supporting the diagnostic.
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    /// Repository or review-artifact path, when applicable.
    pub path: Option<String>,
    /// One-based line number, when applicable.
    pub line: Option<u64>,
    /// One-based column number, when applicable.
    pub column: Option<u64>,
    /// Locale of the human-readable fields.
    pub locale: DiagnosticLocale,
    /// Whether the condition prevents the requested transition or operation.
    pub blocking: bool,
}

impl Validate for Diagnostic {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            DIAGNOSTIC_SCHEMA_VERSION,
            "diagnostic.schema_version",
        )?;
        require_text(&self.code, "diagnostic.code")?;
        require_text(&self.message, "diagnostic.message")?;
        if !self.code.contains('.')
            || self.code.chars().any(|character| {
                !(character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, '.' | '_' | '-'))
            })
        {
            return Err(ContractError::Invariant(
                "diagnostic code must be a lower-case namespaced identifier".to_owned(),
            ));
        }
        if let Some(remediation) = self.remediation.as_deref() {
            require_text(remediation, "diagnostic.remediation")?;
        }
        if self
            .evidence_ids
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(ContractError::Invariant(
                "diagnostic evidence identifiers must not be empty".to_owned(),
            ));
        }
        if let Some(path) = self.path.as_deref() {
            require_text(path, "diagnostic.path")?;
        }
        if self.line.is_none() && self.column.is_some() {
            return Err(ContractError::Invariant(
                "diagnostic column requires a line number".to_owned(),
            ));
        }
        if self.line == Some(0) || self.column == Some(0) {
            return Err(ContractError::Invariant(
                "diagnostic line and column numbers are one-based".to_owned(),
            ));
        }
        if matches!(self.severity, DiagnosticSeverity::Blocking) != self.blocking {
            return Err(ContractError::Invariant(
                "blocking diagnostics must use blocking severity, and vice versa".to_owned(),
            ));
        }
        if let DiagnosticLocale::Custom(tag) = &self.locale {
            require_text(tag, "diagnostic.locale.custom")?;
            if !tag.contains('-') {
                return Err(ContractError::Invariant(
                    "custom diagnostic locales must use a BCP 47-style tag".to_owned(),
                ));
            }
        }
        Ok(())
    }
}
