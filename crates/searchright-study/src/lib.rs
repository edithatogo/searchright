//! Record-report-study graph operations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use searchright_contracts::{
    EvidenceLink, EvidenceRelationship, Report, RetrievalStatus, Study, StudyGraph, Validate,
};

/// Validate graph contracts and relationship referential integrity.
pub fn validate_graph(graph: &StudyGraph) -> Result<(), StudyGraphError> {
    graph.validate()?;
    let known: BTreeSet<&str> = graph
        .reports
        .iter()
        .map(|report| report.report_id.as_str())
        .chain(graph.studies.iter().map(|study| study.study_id.as_str()))
        .collect();
    for link in &graph.links {
        if !known.contains(link.from_id.as_str()) && !is_record_identifier(&link.relationship) {
            return Err(StudyGraphError::UnknownObject(link.from_id.clone()));
        }
        if !known.contains(link.to_id.as_str()) {
            return Err(StudyGraphError::UnknownObject(link.to_id.clone()));
        }
    }
    Ok(())
}

/// Return the latest retrieval status for each report.
#[must_use]
pub fn retrieval_statuses(graph: &StudyGraph) -> BTreeMap<String, RetrievalStatus> {
    graph
        .reports
        .iter()
        .map(|report| {
            let status = report
                .retrieval_attempts
                .last()
                .map_or(RetrievalStatus::NotAttempted, |attempt| {
                    attempt.status.clone()
                });
            (report.report_id.clone(), status)
        })
        .collect()
}

/// Report identifiers not assigned to any study.
#[must_use]
pub fn unlinked_reports(graph: &StudyGraph) -> Vec<String> {
    let linked: BTreeSet<&str> = graph
        .studies
        .iter()
        .flat_map(|study| study.report_ids.iter().map(String::as_str))
        .collect();
    graph
        .reports
        .iter()
        .filter(|report| !linked.contains(report.report_id.as_str()))
        .map(|report| report.report_id.clone())
        .collect()
}

/// Add a report and an evidence-bearing link to an existing study.
pub fn attach_report(
    graph: &mut StudyGraph,
    report: Report,
    study_id: &str,
    link: EvidenceLink,
) -> Result<(), StudyGraphError> {
    if graph
        .reports
        .iter()
        .any(|item| item.report_id == report.report_id)
    {
        return Err(StudyGraphError::DuplicateObject(report.report_id));
    }
    let study = graph
        .studies
        .iter_mut()
        .find(|item| item.study_id == study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(study_id.to_owned()))?;
    if link.relationship != EvidenceRelationship::ReportOfStudy
        || link.from_id != report.report_id
        || link.to_id != study_id
    {
        return Err(StudyGraphError::InvalidAttachmentLink);
    }
    study.report_ids.push(report.report_id.clone());
    graph.reports.push(report);
    graph.links.push(link);
    validate_graph(graph)
}

/// Count reports per study.
#[must_use]
pub fn reports_per_study(graph: &StudyGraph) -> BTreeMap<String, usize> {
    graph
        .studies
        .iter()
        .map(|study: &Study| (study.study_id.clone(), study.report_ids.len()))
        .collect()
}

const fn is_record_identifier(relationship: &EvidenceRelationship) -> bool {
    matches!(relationship, EvidenceRelationship::RecordDescribesReport)
}

/// Study-graph operation error.
#[derive(Debug, thiserror::Error)]
pub enum StudyGraphError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// A graph object identifier was unknown.
    #[error("study graph references unknown object `{0}`")]
    UnknownObject(String),
    /// A graph object identifier was duplicated.
    #[error("study graph already contains object `{0}`")]
    DuplicateObject(String),
    /// The attachment edge did not describe the requested report-to-study link.
    #[error("attachment link must connect the new report to the selected study")]
    InvalidAttachmentLink,
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{
        EvidenceLink, EvidenceRelationship, Report, STUDY_GRAPH_SCHEMA_VERSION, Study, StudyGraph,
    };

    use super::*;

    fn graph() -> StudyGraph {
        StudyGraph {
            schema_version: STUDY_GRAPH_SCHEMA_VERSION.to_owned(),
            review_id: "review-1".to_owned(),
            reports: vec![Report {
                report_id: "report-1".to_owned(),
                record_ids: vec!["record-1".to_owned()],
                title: "Primary report".to_owned(),
                publication_year: Some(2024),
                doi: None,
                pmid: None,
                registry_ids: Vec::new(),
                retrieval_attempts: Vec::new(),
            }],
            studies: vec![Study {
                study_id: "study-1".to_owned(),
                report_ids: vec!["report-1".to_owned()],
                label: "Study one".to_owned(),
                study_design: Some("randomised trial".to_owned()),
                registration_ids: Vec::new(),
                notes: Vec::new(),
            }],
            links: vec![EvidenceLink {
                link_id: "link-1".to_owned(),
                from_id: "report-1".to_owned(),
                to_id: "study-1".to_owned(),
                relationship: EvidenceRelationship::ReportOfStudy,
                confidence: 1.0,
                evidence: vec!["shared registration".to_owned()],
                asserted_by: "human-1".to_owned(),
                asserted_at: "2026-08-06T00:00:00Z".to_owned(),
            }],
        }
    }

    #[test]
    fn valid_graph_has_no_unlinked_reports() {
        let graph = graph();
        assert!(validate_graph(&graph).is_ok());
        assert!(unlinked_reports(&graph).is_empty());
    }
}
