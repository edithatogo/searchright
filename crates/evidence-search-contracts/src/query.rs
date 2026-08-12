use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    COMPILED_STRATEGY_SCHEMA_VERSION, ContractError, NAMED_FILTER_PACK_SCHEMA_VERSION,
    NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION, SEARCH_STRATEGY_SCHEMA_VERSION, Validate,
    require_schema_version, require_text,
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

/// Source citation for one named filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterSourceCitation {
    /// Human-readable source title.
    pub title: String,
    /// Complete citation text suitable for reporting.
    pub citation: String,
    /// Source-defined version or edition of the filter.
    pub source_version: String,
    /// Optional source URI; access and redistribution rights are not implied.
    pub source_uri: Option<String>,
}

impl Validate for FilterSourceCitation {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.title, "named_filter.source.title")?;
        require_text(&self.citation, "named_filter.source.citation")?;
        require_text(&self.source_version, "named_filter.source.source_version")?;
        if let Some(source_uri) = &self.source_uri {
            require_text(source_uri, "named_filter.source.source_uri")?;
        }
        Ok(())
    }
}

/// SHA-256 checksum over the exact UTF-8 bytes of a native filter expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterChecksum {
    /// Digest algorithm. Version 1 admits only `sha256`.
    pub algorithm: String,
    /// Lowercase hexadecimal digest.
    pub digest: String,
}

impl Validate for FilterChecksum {
    fn validate(&self) -> Result<(), ContractError> {
        if self.algorithm != "sha256" {
            return Err(ContractError::Invariant(
                "named-filter checksums must use sha256".to_owned(),
            ));
        }
        if self.digest.len() != 64
            || !self
                .digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::Invariant(
                "named-filter checksum digest must be 64 lowercase hexadecimal characters"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

/// Explicit applicability boundary for one source-native named filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterApplicability {
    /// Information-source identifiers for which the source defines the filter.
    pub source_ids: Vec<String>,
    /// Provider/platform versions against which the filter was assessed.
    pub platform_versions: Vec<String>,
    /// Intended methodological use of the filter.
    pub intended_use: String,
    /// Known exclusions, limitations or non-applicable contexts.
    pub limitations: Vec<String>,
}

impl Validate for FilterApplicability {
    fn validate(&self) -> Result<(), ContractError> {
        require_unique_nonempty_text(&self.source_ids, "named_filter.applicability.source_ids")?;
        require_unique_nonempty_text(
            &self.platform_versions,
            "named_filter.applicability.platform_versions",
        )?;
        require_text(
            &self.intended_use,
            "named_filter.applicability.intended_use",
        )?;
        require_unique_nonempty_text(&self.limitations, "named_filter.applicability.limitations")
    }
}

/// Highest evidence level established for a named-filter record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterValidationState {
    /// Schema and local semantic invariants only; no methodological claim.
    StructuralOnly,
    /// An accountable reviewer assessed methodological suitability.
    MethodologicallyReviewed,
    /// Methodological review and provider-version currency were both evidenced.
    MethodologicallyReviewedAndProviderCurrent,
}

/// Accountable validation evidence for one named filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterValidation {
    /// Highest validation state supported by the evidence.
    pub state: FilterValidationState,
    /// Stable reviewer or validator identity.
    pub reviewer_id: String,
    /// Accountable role of the reviewer or validator.
    pub reviewer_role: String,
    /// Validation method actually performed.
    pub method: String,
    /// Durable reference to the validation evidence.
    pub evidence_reference: String,
    /// SHA-256 checksum of the referenced evidence bytes.
    pub evidence_sha256: String,
}

impl Validate for FilterValidation {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.reviewer_id, "named_filter.validation.reviewer_id")?;
        require_text(&self.reviewer_role, "named_filter.validation.reviewer_role")?;
        require_text(&self.method, "named_filter.validation.method")?;
        require_text(
            &self.evidence_reference,
            "named_filter.validation.evidence_reference",
        )?;
        validate_sha256(
            &self.evidence_sha256,
            "named_filter.validation.evidence_sha256",
        )
    }
}

/// Redistribution decision for exact filter text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedistributionDecision {
    /// Exact filter text may be redistributed under the recorded basis.
    Permitted,
    /// Exact filter text must not be redistributed.
    Prohibited,
    /// Accountable rights review is still required.
    ReviewRequired,
}

/// Rights basis and redistribution decision for one named filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterRights {
    /// Copyright, licence or authorship basis for holding the exact expression.
    pub basis: String,
    /// Explicit redistribution decision; silence never implies permission.
    pub redistribution: RedistributionDecision,
    /// Accountable decision-maker or policy identifier.
    pub decided_by: String,
    /// Durable reference supporting the decision.
    pub evidence_reference: String,
}

impl Validate for FilterRights {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.basis, "named_filter.rights.basis")?;
        require_text(&self.decided_by, "named_filter.rights.decided_by")?;
        require_text(
            &self.evidence_reference,
            "named_filter.rights.evidence_reference",
        )
    }
}

/// One versioned, source-native named filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NamedFilterRecord {
    /// Stable filter identifier across versions.
    pub filter_id: String,
    /// Immutable record version.
    pub version: String,
    /// Human-readable filter name.
    pub name: String,
    /// Native dialect of `expression`.
    pub dialect: SearchDialect,
    /// Exact source-native filter expression.
    pub expression: String,
    /// Checksum over the exact UTF-8 bytes of `expression`.
    pub checksum: FilterChecksum,
    /// Source citation and source-defined version.
    pub source: FilterSourceCitation,
    /// Explicit provider, version and use constraints.
    pub applicability: FilterApplicability,
    /// Evidence-scaled validation status and accountable evidence.
    pub validation: FilterValidation,
    /// Rights basis and explicit redistribution decision.
    pub rights: FilterRights,
    /// First ISO 8601 calendar date on which this record is applicable.
    pub effective_from: String,
    /// Last ISO 8601 calendar date on which this record is applicable.
    pub expires_on: String,
}

impl Validate for NamedFilterRecord {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.filter_id, "named_filter.filter_id")?;
        require_text(&self.version, "named_filter.version")?;
        require_text(&self.name, "named_filter.name")?;
        require_text(&self.expression, "named_filter.expression")?;
        if self.expression.chars().any(char::is_control) {
            return Err(ContractError::Invariant(
                "named-filter expressions must not contain control characters".to_owned(),
            ));
        }
        if let SearchDialect::Custom(name) = &self.dialect {
            require_text(name, "named_filter.dialect.custom")?;
        }
        self.checksum.validate()?;
        let computed = Sha256::digest(self.expression.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if computed != self.checksum.digest {
            return Err(ContractError::Invariant(
                "named-filter checksum must match the exact UTF-8 expression bytes".to_owned(),
            ));
        }
        self.source.validate()?;
        self.applicability.validate()?;
        self.validation.validate()?;
        self.rights.validate()?;
        require_date(&self.effective_from, "named_filter.effective_from")?;
        require_date(&self.expires_on, "named_filter.expires_on")?;
        if self.effective_from > self.expires_on {
            return Err(ContractError::Invariant(
                "named-filter effective date must not follow its expiry date".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Versioned collection of named filters validated as of a declared date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NamedFilterPack {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable pack identifier across versions.
    pub pack_id: String,
    /// Immutable pack version.
    pub version: String,
    /// Human-readable pack title.
    pub title: String,
    /// Date on which every record was checked for structural validity and currency.
    pub validated_on: String,
    /// Last date on which this pack may be treated as current without revalidation.
    pub expires_on: String,
    /// Versioned named-filter records.
    pub filters: Vec<NamedFilterRecord>,
}

impl Validate for NamedFilterPack {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            NAMED_FILTER_PACK_SCHEMA_VERSION,
            "named_filter_pack.schema_version",
        )?;
        require_text(&self.pack_id, "named_filter_pack.pack_id")?;
        require_text(&self.version, "named_filter_pack.version")?;
        require_text(&self.title, "named_filter_pack.title")?;
        require_date(&self.validated_on, "named_filter_pack.validated_on")?;
        require_date(&self.expires_on, "named_filter_pack.expires_on")?;
        if self.validated_on > self.expires_on {
            return Err(ContractError::Invariant(
                "named-filter pack validation date must not follow its expiry date".to_owned(),
            ));
        }
        if self.filters.is_empty() {
            return Err(ContractError::EmptyCollection("named_filter_pack.filters"));
        }
        let mut identities = BTreeSet::new();
        for filter in &self.filters {
            filter.validate()?;
            if !identities.insert((&filter.filter_id, &filter.version)) {
                return Err(ContractError::Invariant(
                    "named-filter pack record identities must be unique".to_owned(),
                ));
            }
            if filter.effective_from > self.validated_on || filter.expires_on < self.validated_on {
                return Err(ContractError::Invariant(
                    "every named filter must be current on the pack validation date".to_owned(),
                ));
            }
            if filter.expires_on < self.expires_on {
                return Err(ContractError::Invariant(
                    "named-filter pack expiry must not follow a record expiry".to_owned(),
                ));
            }
            if filter.rights.redistribution != RedistributionDecision::Permitted {
                return Err(ContractError::Invariant(
                    "validated named-filter packs may contain exact text only when redistribution is explicitly permitted"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }
}

fn require_unique_nonempty_text(
    values: &[String],
    field: &'static str,
) -> Result<(), ContractError> {
    if values.is_empty() {
        return Err(ContractError::EmptyCollection(field));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !seen.insert(value) {
            return Err(ContractError::Invariant(format!(
                "`{field}` must not contain duplicate values"
            )));
        }
    }
    Ok(())
}

fn require_date(value: &str, field: &'static str) -> Result<(), ContractError> {
    require_text(value, field)?;
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return Err(ContractError::Invariant(format!(
            "`{field}` must be an ISO 8601 calendar date in YYYY-MM-DD form"
        )));
    }
    let year = value.get(0..4).and_then(|part| part.parse::<u16>().ok());
    let month = value.get(5..7).and_then(|part| part.parse::<u8>().ok());
    let day = value.get(8..10).and_then(|part| part.parse::<u8>().ok());
    let Some((year, month, day)) = year.zip(month).zip(day).map(|((y, m), d)| (y, m, d)) else {
        return Err(ContractError::Invariant(format!(
            "`{field}` must be a valid ISO 8601 calendar date"
        )));
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if day == 0 || day > maximum_day {
        return Err(ContractError::Invariant(format!(
            "`{field}` must be a valid ISO 8601 calendar date"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ContractError::Invariant(format!(
            "`{field}` must be 64 lowercase hexadecimal characters"
        )));
    }
    Ok(())
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
