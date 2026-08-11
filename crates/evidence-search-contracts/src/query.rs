use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    COMPILED_STRATEGY_SCHEMA_VERSION, ContractError, NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION,
    SEARCH_STRATEGY_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Portable search field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchField {
    /// Search across provider defaults.
    All,
    /// Title.
    Title,
    /// Abstract.
    Abstract,
    /// Title or abstract.
    TitleAbstract,
    /// Author.
    Author,
    /// Journal/source title.
    Journal,
    /// Identifier.
    Identifier,
    /// Controlled-vocabulary heading.
    SubjectHeading,
    /// Keyword field.
    Keyword,
    /// Custom provider-specific field retained explicitly.
    Custom(String),
}

/// One text or controlled-vocabulary term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchTerm {
    /// Literal search text.
    pub text: String,
    /// Target fields.
    #[serde(default)]
    pub fields: Vec<SearchField>,
    /// Optional controlled-vocabulary system such as `MeSH` or Emtree.
    pub vocabulary: Option<String>,
    /// Whether narrower headings should be exploded.
    #[serde(default)]
    pub explode: bool,
    /// Whether the term is a phrase.
    #[serde(default)]
    pub phrase: bool,
    /// Whether truncation is requested.
    #[serde(default)]
    pub truncation: bool,
}

impl Validate for SearchTerm {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.text, "query.term.text")?;
        if self.text.chars().any(char::is_control) {
            return Err(ContractError::Invariant(
                "search terms must not contain control characters".to_owned(),
            ));
        }
        if let Some(vocabulary) = &self.vocabulary {
            require_text(vocabulary, "query.term.vocabulary")?;
        }
        if self.explode
            && self.vocabulary.is_none()
            && !self.fields.contains(&SearchField::SubjectHeading)
        {
            return Err(ContractError::Invariant(
                "explode may be requested only for a controlled-vocabulary term".to_owned(),
            ));
        }
        if self.vocabulary.is_some()
            && !self.fields.is_empty()
            && self
                .fields
                .iter()
                .any(|field| !matches!(field, SearchField::SubjectHeading))
        {
            return Err(ContractError::Invariant(
                "controlled-vocabulary terms may target only the subject-heading field".to_owned(),
            ));
        }
        let mut seen = BTreeSet::new();
        for field in &self.fields {
            let key = serde_json::to_string(field).map_err(|error| {
                ContractError::Invariant(format!("could not canonicalise search field: {error}"))
            })?;
            if !seen.insert(key) {
                return Err(ContractError::Invariant(
                    "search term fields must not contain duplicates".to_owned(),
                ));
            }
            if let SearchField::Custom(name) = field {
                require_text(name, "query.term.fields.custom")?;
                if !name.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
                }) {
                    return Err(ContractError::Invariant(
                        "custom field names may contain only ASCII letters, digits, `_`, `-`, `.`, or `/`"
                            .to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Portable Boolean/proximity query tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum QueryExpr {
    /// Leaf term.
    Term {
        /// The portable search term at this leaf.
        term: SearchTerm,
    },
    /// All children must match.
    And {
        /// The expressions that must all match.
        children: Vec<Self>,
    },
    /// At least one child must match.
    Or {
        /// The expressions of which at least one must match.
        children: Vec<Self>,
    },
    /// Include one expression while excluding another.
    Not {
        /// The expression whose matches are retained.
        include: Box<Self>,
        /// The expression whose matches are excluded.
        exclude: Box<Self>,
    },
    /// Terms must occur within a distance.
    Proximity {
        /// The left-hand expression.
        left: Box<Self>,
        /// The right-hand expression.
        right: Box<Self>,
        /// The maximum allowed distance between the expressions.
        distance: u16,
        /// Whether the expressions must occur in the declared order.
        ordered: bool,
    },
}

impl Validate for QueryExpr {
    fn validate(&self) -> Result<(), ContractError> {
        match self {
            Self::Term { term } => term.validate(),
            Self::And { children } | Self::Or { children } => {
                if children.len() < 2 {
                    return Err(ContractError::Invariant(
                        "and/or expressions require at least two children".to_owned(),
                    ));
                }
                for child in children {
                    child.validate()?;
                }
                Ok(())
            }
            Self::Not { include, exclude } => {
                include.validate()?;
                exclude.validate()
            }
            Self::Proximity {
                left,
                right,
                distance,
                ..
            } => {
                if *distance == 0 {
                    return Err(ContractError::Invariant(
                        "proximity distance must be greater than zero".to_owned(),
                    ));
                }
                left.validate()?;
                right.validate()
            }
        }
    }
}

/// Supported search dialect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchDialect {
    /// PubMed web/API syntax.
    PubMed,
    /// Ovid MEDLINE syntax.
    OvidMedline,
    /// Embase.com syntax.
    Embase,
    /// Europe PMC syntax.
    EuropePmc,
    /// CINAHL via `EBSCOhost`.
    CinahlEbsco,
    /// `PsycINFO` via Ovid.
    PsycInfoOvid,
    /// Scopus advanced search.
    Scopus,
    /// Web of Science advanced search.
    WebOfScience,
    /// Crossref works query approximation.
    Crossref,
    /// OpenAlex works search approximation.
    OpenAlex,
    /// ClinicalTrials.gov query.
    ClinicalTrialsGov,
    /// Generic Boolean syntax.
    GenericBoolean,
    /// Provider-defined dialect.
    Custom(String),
}

/// Inclusive publication-date limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DateLimit {
    /// Start year, if any.
    pub from_year: Option<i32>,
    /// End year, if any.
    pub to_year: Option<i32>,
}

impl Validate for DateLimit {
    fn validate(&self) -> Result<(), ContractError> {
        if self.from_year.is_none() && self.to_year.is_none() {
            return Err(ContractError::Invariant(
                "publication-date limits require at least one boundary".to_owned(),
            ));
        }
        if let (Some(from), Some(to)) = (self.from_year, self.to_year)
            && from > to
        {
            return Err(ContractError::Invariant(
                "publication-date lower bound must not exceed the upper bound".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Search restrictions, which must be reported and justified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SearchLimit {
    /// Publication-date restriction.
    pub publication_date: Option<DateLimit>,
    /// Languages retained by the query.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Publication types retained by the query.
    #[serde(default)]
    pub publication_types: Vec<String>,
    /// Named validated filters.
    #[serde(default)]
    pub filters: Vec<String>,
    /// Rationale for every restriction.
    #[serde(default)]
    pub rationale: Vec<String>,
}

impl Validate for SearchLimit {
    fn validate(&self) -> Result<(), ContractError> {
        if let Some(date) = &self.publication_date {
            date.validate()?;
        }
        for value in self
            .languages
            .iter()
            .chain(&self.publication_types)
            .chain(&self.filters)
            .chain(&self.rationale)
        {
            if value.trim().is_empty() {
                return Err(ContractError::Invariant(
                    "search limits, filters and rationales must not contain empty values"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// Canonical source-specific strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchStrategy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable strategy identifier.
    pub strategy_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Information-source identifier.
    pub source_id: String,
    /// Intended dialect.
    pub dialect: SearchDialect,
    /// Portable query expression.
    pub query: QueryExpr,
    /// Limits and filters.
    #[serde(default)]
    pub limits: SearchLimit,
    /// Strategy from which this was translated, if any.
    pub translated_from: Option<String>,
    /// Searcher notes.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Validate for SearchStrategy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            SEARCH_STRATEGY_SCHEMA_VERSION,
            "schema_version",
        )?;
        require_text(&self.strategy_id, "strategy_id")?;
        require_text(&self.review_id, "review_id")?;
        require_text(&self.source_id, "source_id")?;
        if let SearchDialect::Custom(name) = &self.dialect {
            require_text(name, "dialect.custom")?;
        }
        self.query.validate()?;
        self.limits.validate()?;
        if (!self.limits.languages.is_empty()
            || !self.limits.publication_types.is_empty()
            || !self.limits.filters.is_empty()
            || self.limits.publication_date.is_some())
            && self.limits.rationale.is_empty()
        {
            return Err(ContractError::Invariant(
                "search restrictions and filters require an explicit rationale".to_owned(),
            ));
        }
        if let Some(translated_from) = &self.translated_from {
            require_text(translated_from, "translated_from")?;
        }
        Ok(())
    }
}

/// Byte span in an immutable native search strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NativeSourceSpan {
    /// Inclusive byte offset.
    pub start_byte: u64,
    /// Exclusive byte offset.
    pub end_byte: u64,
}

/// Classification of one native strategy line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeQueryLineKind {
    /// Provider-native search expression.
    Expression,
    /// Combination of previously defined line or set identifiers.
    SetCombination,
    /// Source-native limit or filter command.
    Limit,
    /// Human comment retained verbatim.
    Comment,
    /// Blank line retained to preserve exact source text.
    Blank,
    /// Syntax preserved but not yet classified by the parser.
    Unknown,
}

/// One immutable line in a native search strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NativeQueryLine {
    /// Stable line identity within the strategy.
    pub line_id: String,
    /// Optional provider set/line identifier parsed from the native text.
    pub native_set_id: Option<String>,
    /// Exact source text excluding its line ending.
    pub text: String,
    /// Source classification.
    pub kind: NativeQueryLineKind,
    /// Byte span into `NativeSearchStrategy.raw_text`.
    pub span: NativeSourceSpan,
}

/// Severity of a native-query parser diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeParseSeverity {
    /// Informational parser observation.
    Info,
    /// Potentially lossy or ambiguous interpretation.
    Warning,
    /// Parser could not establish a safe interpretation.
    Error,
}

/// Source-grounded parser diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NativeParseDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Severity.
    pub severity: NativeParseSeverity,
    /// Human-readable explanation.
    pub message: String,
    /// Optional source span.
    pub span: Option<NativeSourceSpan>,
    /// Whether execution or translation requires human review.
    pub review_required: bool,
}

/// Relationship between preserved native text and the portable semantic model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeNormalisationState {
    /// Only source-preserving lexical structure is available.
    RawOnly,
    /// Some semantics were parsed, but unsupported constructs remain.
    Partial,
    /// The portable model represents every known native construct without loss.
    Complete,
}

/// Dual-representation search strategy preserving native text and optional semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NativeSearchStrategy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable strategy identifier.
    pub strategy_id: String,
    /// Source dialect.
    pub dialect: SearchDialect,
    /// Exact source text, including original line endings.
    pub raw_text: String,
    /// Source-preserving lexical lines.
    pub lines: Vec<NativeQueryLine>,
    /// Optional portable semantic representation.
    pub semantic_strategy: Option<SearchStrategy>,
    /// Current normalisation state.
    pub normalisation_state: NativeNormalisationState,
    /// Parser diagnostics.
    #[serde(default)]
    pub diagnostics: Vec<NativeParseDiagnostic>,
    /// Parser implementation version.
    pub parser_version: String,
}

impl Validate for NativeSearchStrategy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION,
            "native_search_strategy.schema_version",
        )?;
        require_text(&self.strategy_id, "native_search_strategy.strategy_id")?;
        require_text(&self.raw_text, "native_search_strategy.raw_text")?;
        require_text(
            &self.parser_version,
            "native_search_strategy.parser_version",
        )?;
        if self.lines.is_empty() {
            return Err(ContractError::EmptyCollection(
                "native_search_strategy.lines",
            ));
        }
        let raw_len = u64::try_from(self.raw_text.len()).unwrap_or(u64::MAX);
        let mut identifiers = BTreeSet::new();
        let mut native_set_ids = BTreeSet::new();
        let mut previous_end = 0_u64;
        for line in &self.lines {
            require_text(&line.line_id, "native_search_strategy.lines.line_id")?;
            if !identifiers.insert(line.line_id.as_str()) {
                return Err(ContractError::Invariant(
                    "native search line identifiers must be unique".to_owned(),
                ));
            }
            if line.span.start_byte < previous_end
                || line.span.end_byte < line.span.start_byte
                || line.span.end_byte > raw_len
            {
                return Err(ContractError::Invariant(
                    "native search line spans must be ordered and remain within raw_text"
                        .to_owned(),
                ));
            }
            previous_end = line.span.end_byte;
            let start = usize::try_from(line.span.start_byte).map_err(|_| {
                ContractError::Invariant("native search line span cannot be indexed".to_owned())
            })?;
            let end = usize::try_from(line.span.end_byte).map_err(|_| {
                ContractError::Invariant("native search line span cannot be indexed".to_owned())
            })?;
            if self.raw_text.as_bytes().get(start..end) != Some(line.text.as_bytes()) {
                return Err(ContractError::Invariant(
                    "native search line text must exactly match its raw_text byte span".to_owned(),
                ));
            }
            if let Some(native_set_id) = &line.native_set_id {
                require_text(native_set_id, "native_search_strategy.lines.native_set_id")?;
                if !native_set_ids.insert(native_set_id.as_str()) {
                    return Err(ContractError::Invariant(
                        "native search set identifiers must be unique".to_owned(),
                    ));
                }
            }
        }
        for diagnostic in &self.diagnostics {
            require_text(&diagnostic.code, "native_search_strategy.diagnostics.code")?;
            require_text(
                &diagnostic.message,
                "native_search_strategy.diagnostics.message",
            )?;
            if let Some(span) = diagnostic.span
                && (span.end_byte < span.start_byte || span.end_byte > raw_len)
            {
                return Err(ContractError::Invariant(
                    "native parser diagnostic span is outside raw_text".to_owned(),
                ));
            }
        }
        if let Some(strategy) = &self.semantic_strategy {
            strategy.validate()?;
            if strategy.strategy_id != self.strategy_id {
                return Err(ContractError::Invariant(
                    "native and semantic strategy identifiers must match".to_owned(),
                ));
            }
        }
        if self.normalisation_state == NativeNormalisationState::Complete
            && self.semantic_strategy.is_none()
        {
            return Err(ContractError::Invariant(
                "complete native normalisation requires a semantic strategy".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Warning produced while translating a portable query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StrategyWarning {
    /// Stable warning code.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
    /// Whether human review is required before execution.
    pub review_required: bool,
}

/// Overall semantic fidelity of a source-specific translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TranslationFidelity {
    /// The portable expression was rendered without a known semantic change.
    Exact,
    /// The target uses different syntax but preserves the requested meaning.
    SourceEquivalent,
    /// Some source-specific interpretation or manual completion is required.
    Approximate,
    /// One or more requested semantics were degraded or replaced by a fallback.
    Degraded,
}

/// Rendered provider query plus translation evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledStrategy {
    /// Contract identifier.
    pub schema_version: String,
    /// Source strategy identifier.
    pub strategy_id: String,
    /// Target dialect.
    pub dialect: SearchDialect,
    /// Rendered query.
    pub query: String,
    /// Translation warnings.
    #[serde(default)]
    pub warnings: Vec<StrategyWarning>,
    /// Aggregate translation fidelity.
    pub fidelity: TranslationFidelity,
    /// Whether execution must pause for human translation review.
    pub review_required: bool,
    /// Stable codes for warnings that represent semantic loss or manual work.
    #[serde(default)]
    pub loss_codes: Vec<String>,
    /// Deterministic hash of canonical input and compiler version.
    pub compilation_hash: String,
    /// Compiler contract version.
    pub compiler_version: String,
}

impl Validate for CompiledStrategy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            COMPILED_STRATEGY_SCHEMA_VERSION,
            "compiled_strategy.schema_version",
        )?;
        require_text(&self.strategy_id, "compiled_strategy.strategy_id")?;
        require_text(&self.query, "compiled_strategy.query")?;
        require_text(&self.compilation_hash, "compiled_strategy.compilation_hash")?;
        require_text(&self.compiler_version, "compiled_strategy.compiler_version")?;
        for warning in &self.warnings {
            require_text(&warning.code, "compiled_strategy.warnings.code")?;
            require_text(&warning.message, "compiled_strategy.warnings.message")?;
        }
        if self.review_required && self.warnings.iter().all(|warning| !warning.review_required) {
            return Err(ContractError::Invariant(
                "compiled strategy requires review but has no review-requiring warning".to_owned(),
            ));
        }
        if self.loss_codes.iter().any(|code| code.trim().is_empty()) {
            return Err(ContractError::Invariant(
                "compiled-strategy loss codes must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}
