use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, STUDY_GRAPH_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Relationship between two evidence objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelationship {
    /// A bibliographic record describes a report.
    RecordDescribesReport,
    /// A report concerns a study.
    ReportOfStudy,
    /// A report is a protocol for a study.
    ProtocolForStudy,
    /// A report is a secondary analysis of a study.
    SecondaryAnalysisOfStudy,
    /// A report updates or corrects another report.
    UpdatesReport,
    /// Two objects are known duplicates.
    DuplicateOf,
    /// A declared custom relationship.
    Custom(String),
}

/// Full-text retrieval state for one report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStatus {
    /// Retrieval has not yet been attempted.
    NotAttempted,
    /// A usable full text was retrieved.
    Retrieved,
    /// A retrieval attempt failed.
    NotRetrieved,
    /// Access is restricted by licence or authentication.
    Restricted,
    /// The report appears unavailable.
    Unavailable,
    /// The team is awaiting a response from an author or institution.
    AwaitingContact,
    /// Another explicitly described state.
    Other(String),
}

/// Auditable attempt to retrieve a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalAttempt {
    /// Stable attempt identifier.
    pub attempt_id: String,
    /// Report identifier.
    pub report_id: String,
    /// RFC 3339 timestamp.
    pub attempted_at: String,
    /// Retrieval route such as publisher, repository, interlibrary loan or author contact.
    pub method: String,
    /// Result of the attempt.
    pub status: RetrievalStatus,
    /// Evidence-bearing note or failure reason.
    pub note: String,
    /// Rights or access basis. This must not contain credentials.
    pub rights_basis: Option<String>,
    /// Optional content checksum for locally held, rights-compliant material.
    pub content_digest: Option<String>,
}

/// One publication, preprint, registry entry, abstract or other report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    /// Stable report identifier.
    pub report_id: String,
    /// Source bibliographic record identifiers.
    pub record_ids: Vec<String>,
    /// Report title.
    pub title: String,
    /// Publication year when known.
    pub publication_year: Option<i32>,
    /// DOI when known.
    pub doi: Option<String>,
    /// PMID when known.
    pub pmid: Option<String>,
    /// Registry identifiers.
    #[serde(default)]
    pub registry_ids: Vec<String>,
    /// Retrieval attempts in chronological order.
    #[serde(default)]
    pub retrieval_attempts: Vec<RetrievalAttempt>,
}

/// One underlying study, which may have multiple reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Study {
    /// Stable study identifier.
    pub study_id: String,
    /// Linked report identifiers.
    pub report_ids: Vec<String>,
    /// Human-readable label.
    pub label: String,
    /// Declared study design when known.
    pub study_design: Option<String>,
    /// Trial or study registration identifiers.
    #[serde(default)]
    pub registration_ids: Vec<String>,
    /// Notes supporting linkage or uncertainty.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// Evidence-bearing graph edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceLink {
    /// Stable link identifier.
    pub link_id: String,
    /// Source object identifier.
    pub from_id: String,
    /// Destination object identifier.
    pub to_id: String,
    /// Relationship type.
    pub relationship: EvidenceRelationship,
    /// Confidence from zero to one.
    pub confidence: f64,
    /// Evidence used to assert the relationship.
    pub evidence: Vec<String>,
    /// Human or tool that asserted the relationship.
    pub asserted_by: String,
    /// RFC 3339 assertion timestamp.
    pub asserted_at: String,
}

/// Record-report-study graph for one review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StudyGraph {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Reports in the graph.
    #[serde(default)]
    pub reports: Vec<Report>,
    /// Studies in the graph.
    #[serde(default)]
    pub studies: Vec<Study>,
    /// Typed links among records, reports and studies.
    #[serde(default)]
    pub links: Vec<EvidenceLink>,
}

impl Validate for RetrievalAttempt {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.attempt_id, "retrieval.attempt_id")?;
        require_text(&self.report_id, "retrieval.report_id")?;
        require_text(&self.attempted_at, "retrieval.attempted_at")?;
        require_text(&self.method, "retrieval.method")?;
        require_text(&self.note, "retrieval.note")?;
        if let Some(rights_basis) = &self.rights_basis {
            require_text(rights_basis, "retrieval.rights_basis")?;
        }
        if let Some(content_digest) = &self.content_digest {
            require_text(content_digest, "retrieval.content_digest")?;
        }
        Ok(())
    }
}

impl Validate for Report {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.report_id, "report.report_id")?;
        require_text(&self.title, "report.title")?;
        if self.record_ids.is_empty() {
            return Err(ContractError::EmptyCollection("report.record_ids"));
        }
        for attempt in &self.retrieval_attempts {
            attempt.validate()?;
            if attempt.report_id != self.report_id {
                return Err(ContractError::Invariant(format!(
                    "retrieval attempt `{}` points to report `{}` instead of `{}`",
                    attempt.attempt_id, attempt.report_id, self.report_id
                )));
            }
        }
        Ok(())
    }
}

impl Validate for Study {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.study_id, "study.study_id")?;
        require_text(&self.label, "study.label")?;
        if self.report_ids.is_empty() {
            return Err(ContractError::EmptyCollection("study.report_ids"));
        }
        Ok(())
    }
}

impl Validate for EvidenceLink {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.link_id, "study_graph.links.link_id")?;
        require_text(&self.from_id, "study_graph.links.from_id")?;
        require_text(&self.to_id, "study_graph.links.to_id")?;
        require_text(&self.asserted_by, "study_graph.links.asserted_by")?;
        require_text(&self.asserted_at, "study_graph.links.asserted_at")?;
        if self.from_id == self.to_id {
            return Err(ContractError::Invariant(
                "study-graph links must not be self-referential".to_owned(),
            ));
        }
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ContractError::Invariant(
                "study-graph link confidence must be between zero and one".to_owned(),
            ));
        }
        if self.evidence.is_empty() {
            return Err(ContractError::EmptyCollection("study_graph.links.evidence"));
        }
        Ok(())
    }
}

impl Validate for StudyGraph {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            STUDY_GRAPH_SCHEMA_VERSION,
            "study_graph.schema_version",
        )?;
        require_text(&self.review_id, "study_graph.review_id")?;

        let mut object_ids = BTreeSet::new();
        let mut report_ids = BTreeSet::new();
        let mut link_ids = BTreeSet::new();
        for report in &self.reports {
            report.validate()?;
            if !object_ids.insert(report.report_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "study-graph object identifier `{}` is duplicated",
                    report.report_id
                )));
            }
            report_ids.insert(report.report_id.as_str());
        }
        for study in &self.studies {
            study.validate()?;
            if !object_ids.insert(study.study_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "study-graph object identifier `{}` is duplicated",
                    study.study_id
                )));
            }
            let mut study_report_ids = BTreeSet::new();
            for report_id in &study.report_ids {
                if !report_ids.contains(report_id.as_str()) {
                    return Err(ContractError::Invariant(format!(
                        "study `{}` references unknown report `{report_id}`",
                        study.study_id
                    )));
                }
                if !study_report_ids.insert(report_id.as_str()) {
                    return Err(ContractError::Invariant(format!(
                        "study `{}` contains duplicate report `{report_id}`",
                        study.study_id
                    )));
                }
            }
        }
        for link in &self.links {
            link.validate()?;
            if !link_ids.insert(link.link_id.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "study-graph link identifier `{}` is duplicated",
                    link.link_id
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(id: &str) -> Report {
        Report {
            report_id: id.to_owned(),
            record_ids: vec![format!("record-{id}")],
            title: format!("Report {id}"),
            publication_year: None,
            doi: None,
            pmid: None,
            registry_ids: Vec::new(),
            retrieval_attempts: Vec::new(),
        }
    }

    fn study(id: &str, report_ids: &[&str]) -> Study {
        Study {
            study_id: id.to_owned(),
            report_ids: report_ids.iter().map(|value| (*value).to_owned()).collect(),
            label: format!("Study {id}"),
            study_design: None,
            registration_ids: Vec::new(),
            notes: Vec::new(),
        }
    }

    fn graph(studies: Vec<Study>) -> StudyGraph {
        StudyGraph {
            schema_version: STUDY_GRAPH_SCHEMA_VERSION.to_owned(),
            review_id: "review-1".to_owned(),
            reports: vec![report("report-1")],
            studies,
            links: Vec::new(),
        }
    }

    #[test]
    fn rejects_duplicate_report_assignment_within_a_study() {
        let graph = graph(vec![study("study-1", &["report-1", "report-1"])]);
        assert!(
            matches!(graph.validate(), Err(ContractError::Invariant(message)) if message.contains("duplicate report"))
        );
    }

    #[test]
    fn permits_one_report_to_describe_multiple_studies() {
        let graph = graph(vec![
            study("study-1", &["report-1"]),
            study("study-2", &["report-1"]),
        ]);
        assert!(graph.validate().is_ok());
    }
}
