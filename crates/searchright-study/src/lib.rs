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
    validate_graph(graph)?;
    if graph
        .reports
        .iter()
        .any(|item| item.report_id == report.report_id)
    {
        return Err(StudyGraphError::DuplicateObject(report.report_id));
    }
    let study_index = graph
        .studies
        .iter()
        .position(|item| item.study_id == study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(study_id.to_owned()))?;
    if link.relationship != EvidenceRelationship::ReportOfStudy
        || link.from_id != report.report_id
        || link.to_id != study_id
    {
        return Err(StudyGraphError::InvalidAttachmentLink);
    }
    let mut candidate = graph.clone();
    candidate
        .studies
        .get_mut(study_index)
        .ok_or_else(|| StudyGraphError::UnknownObject(study_id.to_owned()))?
        .report_ids
        .push(report.report_id.clone());
    candidate.reports.push(report);
    candidate.links.push(link);
    validate_graph(&candidate)?;
    *graph = candidate;
    Ok(())
}

/// Link an existing report to an existing study using explicit evidence.
pub fn link_report_to_study(
    graph: &mut StudyGraph,
    report_id: &str,
    study_id: &str,
    link: EvidenceLink,
) -> Result<(), StudyGraphError> {
    validate_graph(graph)?;
    if !graph
        .reports
        .iter()
        .any(|report| report.report_id == report_id)
        || !graph.studies.iter().any(|study| study.study_id == study_id)
    {
        let unknown = if graph
            .reports
            .iter()
            .any(|report| report.report_id == report_id)
        {
            study_id
        } else {
            report_id
        };
        return Err(StudyGraphError::UnknownObject(unknown.to_owned()));
    }
    if link.relationship != EvidenceRelationship::ReportOfStudy
        || link.from_id != report_id
        || link.to_id != study_id
    {
        return Err(StudyGraphError::InvalidAttachmentLink);
    }

    let mut candidate = graph.clone();
    let study = candidate
        .studies
        .iter_mut()
        .find(|study| study.study_id == study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(study_id.to_owned()))?;
    if study
        .report_ids
        .iter()
        .any(|existing| existing == report_id)
    {
        return Err(StudyGraphError::DuplicateObject(format!(
            "{report_id} in {study_id}"
        )));
    }
    study.report_ids.push(report_id.to_owned());
    candidate.links.push(link);
    validate_graph(&candidate)?;
    *graph = candidate;
    Ok(())
}

/// Merge one or more source studies into a target study transactionally.
pub fn merge_studies(
    graph: &mut StudyGraph,
    target_study_id: &str,
    source_study_ids: &[&str],
    evidence: EvidenceLink,
) -> Result<(), StudyGraphError> {
    validate_graph(graph)?;
    if source_study_ids.is_empty() {
        return Ok(());
    }
    if source_study_ids.contains(&target_study_id) {
        return Err(StudyGraphError::InvalidOperation(
            "cannot merge a study into itself".to_owned(),
        ));
    }
    if !graph
        .studies
        .iter()
        .any(|study| study.study_id == target_study_id)
    {
        return Err(StudyGraphError::UnknownObject(target_study_id.to_owned()));
    }
    for &src in source_study_ids {
        if !graph.studies.iter().any(|study| study.study_id == src) {
            return Err(StudyGraphError::UnknownObject(src.to_owned()));
        }
    }

    let mut candidate = graph.clone();
    let mut collected_report_ids = Vec::new();

    for &src in source_study_ids {
        if let Some(pos) = candidate
            .studies
            .iter()
            .position(|study| study.study_id == src)
        {
            let removed = candidate.studies.remove(pos);
            collected_report_ids.extend(removed.report_ids);
        }
    }

    let target = candidate
        .studies
        .iter_mut()
        .find(|study| study.study_id == target_study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(target_study_id.to_owned()))?;

    for rid in collected_report_ids {
        if !target.report_ids.contains(&rid) {
            target.report_ids.push(rid);
        }
    }

    let source_set: BTreeSet<&str> = source_study_ids.iter().copied().collect();
    for link in &mut candidate.links {
        if source_set.contains(link.to_id.as_str()) {
            target_study_id.clone_into(&mut link.to_id);
        }
    }

    candidate.links.push(evidence);
    validate_graph(&candidate)?;
    *graph = candidate;
    Ok(())
}

/// Split reports from an existing study into a new study transactionally.
pub fn split_study(
    graph: &mut StudyGraph,
    original_study_id: &str,
    mut new_study: Study,
    report_ids_to_move: &[&str],
    evidence: EvidenceLink,
) -> Result<(), StudyGraphError> {
    validate_graph(graph)?;
    if report_ids_to_move.is_empty() {
        return Err(StudyGraphError::InvalidOperation(
            "split requires at least one report to move".to_owned(),
        ));
    }
    if graph
        .studies
        .iter()
        .any(|study| study.study_id == new_study.study_id)
    {
        return Err(StudyGraphError::DuplicateObject(new_study.study_id));
    }
    let orig = graph
        .studies
        .iter()
        .find(|study| study.study_id == original_study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(original_study_id.to_owned()))?;

    for &rid in report_ids_to_move {
        if !orig.report_ids.iter().any(|id| id == rid) {
            return Err(StudyGraphError::UnknownObject(format!(
                "report {rid} not in study {original_study_id}"
            )));
        }
    }

    let mut candidate = graph.clone();
    let move_set: BTreeSet<&str> = report_ids_to_move.iter().copied().collect();

    let orig_mut = candidate
        .studies
        .iter_mut()
        .find(|study| study.study_id == original_study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(original_study_id.to_owned()))?;
    orig_mut
        .report_ids
        .retain(|id| !move_set.contains(id.as_str()));

    new_study.report_ids = report_ids_to_move.iter().map(|s| (*s).to_owned()).collect();
    let new_study_id = new_study.study_id.clone();
    candidate.studies.push(new_study);

    for link in &mut candidate.links {
        if move_set.contains(link.from_id.as_str()) && link.to_id == original_study_id {
            link.to_id.clone_from(&new_study_id);
        }
    }

    candidate.links.push(evidence);
    validate_graph(&candidate)?;
    *graph = candidate;
    Ok(())
}

/// Detach a report from a study transactionally.
pub fn detach_report(
    graph: &mut StudyGraph,
    report_id: &str,
    study_id: &str,
    evidence: EvidenceLink,
) -> Result<(), StudyGraphError> {
    validate_graph(graph)?;
    let mut candidate = graph.clone();
    let study = candidate
        .studies
        .iter_mut()
        .find(|s| s.study_id == study_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(study_id.to_owned()))?;

    let pos = study
        .report_ids
        .iter()
        .position(|id| id == report_id)
        .ok_or_else(|| StudyGraphError::UnknownObject(format!("{report_id} in {study_id}")))?;
    study.report_ids.remove(pos);

    candidate.links.push(evidence);
    validate_graph(&candidate)?;
    *graph = candidate;
    Ok(())
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
    /// Invalid graph modification operation.
    #[error("invalid study graph operation: {0}")]
    InvalidOperation(String),
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions"
)]
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

    #[test]
    fn failed_attachment_does_not_mutate_graph() {
        let mut graph = graph();
        let original = graph.clone();
        let report = Report {
            report_id: "report-2".to_owned(),
            record_ids: vec!["record-2".to_owned()],
            title: "Secondary report".to_owned(),
            publication_year: None,
            doi: None,
            pmid: None,
            registry_ids: Vec::new(),
            retrieval_attempts: Vec::new(),
        };
        let invalid_link = EvidenceLink {
            link_id: "link-1".to_owned(),
            from_id: "report-2".to_owned(),
            to_id: "study-1".to_owned(),
            relationship: EvidenceRelationship::ReportOfStudy,
            confidence: 1.0,
            evidence: vec!["test evidence".to_owned()],
            asserted_by: "human-1".to_owned(),
            asserted_at: "2026-08-12T00:00:00Z".to_owned(),
        };

        assert!(matches!(
            attach_report(&mut graph, report, "study-1", invalid_link),
            Err(StudyGraphError::Contract(_))
        ));
        assert_eq!(graph, original);
    }

    #[test]
    fn merge_and_split_studies_operations() {
        let mut graph = graph();
        let report2 = Report {
            report_id: "report-2".to_owned(),
            record_ids: vec!["record-2".to_owned()],
            title: "Secondary report".to_owned(),
            publication_year: Some(2025),
            doi: None,
            pmid: None,
            registry_ids: Vec::new(),
            retrieval_attempts: Vec::new(),
        };
        let link2 = EvidenceLink {
            link_id: "link-2".to_owned(),
            from_id: "report-2".to_owned(),
            to_id: "study-2".to_owned(),
            relationship: EvidenceRelationship::ReportOfStudy,
            confidence: 1.0,
            evidence: vec!["trial id".to_owned()],
            asserted_by: "human-1".to_owned(),
            asserted_at: "2026-08-12T00:00:00Z".to_owned(),
        };
        graph.reports.push(report2);
        graph.studies.push(Study {
            study_id: "study-2".to_owned(),
            report_ids: vec!["report-2".to_owned()],
            label: "Study two".to_owned(),
            study_design: None,
            registration_ids: Vec::new(),
            notes: Vec::new(),
        });
        graph.links.push(link2);

        // Merge study-2 into study-1
        let merge_evidence = EvidenceLink {
            link_id: "merge-1".to_owned(),
            from_id: "report-2".to_owned(),
            to_id: "study-1".to_owned(),
            relationship: EvidenceRelationship::ReportOfStudy,
            confidence: 0.95,
            evidence: vec!["human merged duplicate trial cohorts".to_owned()],
            asserted_by: "reviewer-1".to_owned(),
            asserted_at: "2026-08-15T00:00:00Z".to_owned(),
        };
        let merge_res = merge_studies(&mut graph, "study-1", &["study-2"], merge_evidence);
        assert!(merge_res.is_ok());
        assert_eq!(graph.studies.len(), 1);
        assert_eq!(graph.studies[0].report_ids.len(), 2);

        // Split report-2 into new study-3
        let new_study = Study {
            study_id: "study-3".to_owned(),
            report_ids: Vec::new(),
            label: "Study three".to_owned(),
            study_design: None,
            registration_ids: Vec::new(),
            notes: Vec::new(),
        };
        let split_evidence = EvidenceLink {
            link_id: "split-1".to_owned(),
            from_id: "report-2".to_owned(),
            to_id: "study-3".to_owned(),
            relationship: EvidenceRelationship::ReportOfStudy,
            confidence: 0.95,
            evidence: vec!["human split follow-up trial into distinct arm".to_owned()],
            asserted_by: "reviewer-1".to_owned(),
            asserted_at: "2026-08-16T00:00:00Z".to_owned(),
        };
        let split_res = split_study(
            &mut graph,
            "study-1",
            new_study,
            &["report-2"],
            split_evidence,
        );
        assert!(split_res.is_ok());
        assert_eq!(graph.studies.len(), 2);
        assert_eq!(reports_per_study(&graph).get("study-1"), Some(&1));
        assert_eq!(reports_per_study(&graph).get("study-3"), Some(&1));
    }
}
