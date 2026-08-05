use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, SEARCH_STRATEGY_SCHEMA_VERSION, Validate, require_schema_version, require_text,
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
    /// Optional controlled-vocabulary system such as MeSH or Emtree.
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
                "controlled-vocabulary terms may target only the subject-heading field"
                    .to_owned(),
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
                    character.is_ascii_alphanumeric()
                        || matches!(character, '_' | '-' | '.' | '/')
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
    Term { term: SearchTerm },
    /// All children must match.
    And { children: Vec<QueryExpr> },
    /// At least one child must match.
    Or { children: Vec<QueryExpr> },
    /// Include one expression while excluding another.
    Not {
        include: Box<QueryExpr>,
        exclude: Box<QueryExpr>,
    },
    /// Terms must occur within a distance.
    Proximity {
        left: Box<QueryExpr>,
        right: Box<QueryExpr>,
        distance: u16,
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
    /// CINAHL via EBSCOhost.
    CinahlEbsco,
    /// PsycINFO via Ovid.
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

/// Rendered provider query plus translation evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledStrategy {
    /// Source strategy identifier.
    pub strategy_id: String,
    /// Target dialect.
    pub dialect: SearchDialect,
    /// Rendered query.
    pub query: String,
    /// Translation warnings.
    #[serde(default)]
    pub warnings: Vec<StrategyWarning>,
    /// Deterministic hash of canonical input and compiler version.
    pub compilation_hash: String,
    /// Compiler contract version.
    pub compiler_version: String,
}
