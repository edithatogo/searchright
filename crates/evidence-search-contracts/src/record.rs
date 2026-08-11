use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, ContractError, Validate, require_schema_version,
    require_text,
};

/// Common identifiers used for record and report matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RecordIdentifiers {
    /// Digital Object Identifier.
    pub doi: Option<String>,
    /// PubMed identifier.
    pub pmid: Option<String>,
    /// PubMed Central identifier.
    pub pmcid: Option<String>,
    /// Clinical trial registration identifier.
    pub trial_registration: Option<String>,
    /// OpenAlex work identifier.
    pub openalex: Option<String>,
    /// International Standard Book Number.
    pub isbn: Option<String>,
    /// Additional namespaced identifiers.
    #[serde(default)]
    pub other: BTreeMap<String, String>,
}

/// Kind of scholarly or grey-literature report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    /// Journal article.
    JournalArticle,
    /// Preprint.
    Preprint,
    /// Conference abstract or proceeding.
    Conference,
    /// Trial-registry record.
    TrialRegistry,
    /// Thesis or dissertation.
    Thesis,
    /// Report.
    Report,
    /// Dataset.
    Dataset,
    /// Other source type.
    Other(String),
}

/// Provider-normalised bibliographic record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BibliographicRecord {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable Searchright record identifier.
    pub record_id: String,
    /// Source receipt that introduced the record.
    pub source_receipt_id: String,
    /// Provider-native identifier.
    pub native_id: String,
    /// Report kind.
    pub kind: RecordKind,
    /// Identifiers.
    #[serde(default)]
    pub identifiers: RecordIdentifiers,
    /// Title.
    pub title: String,
    /// Abstract, when available and permitted.
    pub abstract_text: Option<String>,
    /// Author strings in source order.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Container title.
    pub container_title: Option<String>,
    /// Publication year.
    pub publication_year: Option<i32>,
    /// Publication date as source text or ISO date.
    pub publication_date: Option<String>,
    /// Language codes or source labels.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Subject terms.
    #[serde(default)]
    pub subjects: Vec<String>,
    /// Source URLs retained under policy.
    #[serde(default)]
    pub urls: Vec<String>,
    /// Lossless provider payload subset permitted by policy.
    #[serde(default)]
    pub provider_metadata: Value,
}

impl Validate for BibliographicRecord {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION,
            "record.schema_version",
        )?;
        require_text(&self.record_id, "record.record_id")?;
        require_text(&self.source_receipt_id, "record.source_receipt_id")?;
        require_text(&self.native_id, "record.native_id")?;
        require_text(&self.title, "record.title")?;
        if let Some(year) = self.publication_year
            && !(-10_000..=30_000).contains(&year)
        {
            return Err(ContractError::Invariant(
                "publication year is outside the supported archival range".to_owned(),
            ));
        }
        for value in self
            .authors
            .iter()
            .chain(&self.languages)
            .chain(&self.subjects)
            .chain(&self.urls)
        {
            if value.trim().is_empty() {
                return Err(ContractError::Invariant(
                    "bibliographic string collections must not contain empty values".to_owned(),
                ));
            }
        }
        for (namespace, identifier) in &self.identifiers.other {
            if namespace.trim().is_empty() || identifier.trim().is_empty() {
                return Err(ContractError::Invariant(
                    "additional identifier namespaces and values must be non-empty".to_owned(),
                ));
            }
        }
        Ok(())
    }
}
