//! Graph-operation edge cases for record-report-study linkage.

use searchright_contracts::{
    EvidenceLink, EvidenceRelationship, Report, RetrievalAttempt, RetrievalStatus,
    STUDY_GRAPH_SCHEMA_VERSION, Study, StudyGraph,
};
use searchright_study::{
    StudyGraphError, attach_report, detach_report, link_report_to_study, merge_studies,
    reports_per_study, retrieval_statuses, split_study, unlinked_reports, validate_graph,
};

fn report(id: &str) -> Report {
    Report {
        report_id: id.to_string(),
        record_ids: vec![format!("record-{id}")],
        title: format!("Report {id}"),
        publication_year: Some(2024),
        doi: None,
        pmid: None,
        registry_ids: vec![],
        retrieval_attempts: vec![],
    }
}

fn attempt(report_id: &str, status: RetrievalStatus) -> RetrievalAttempt {
    RetrievalAttempt {
        attempt_id: format!("attempt-{report_id}-{status:?}"),
        report_id: report_id.to_string(),
        attempted_at: "2026-08-06T00:00:00Z".to_string(),
        method: "publisher".to_string(),
        status,
        note: "synthetic".to_string(),
        rights_basis: None,
        content_digest: None,
    }
}

fn link(link_id: &str, from: &str, to: &str) -> EvidenceLink {
    EvidenceLink {
        link_id: link_id.to_string(),
        from_id: from.to_string(),
        to_id: to.to_string(),
        relationship: EvidenceRelationship::ReportOfStudy,
        confidence: 1.0,
        evidence: vec!["synthetic".to_string()],
        asserted_by: "human-1".to_string(),
        asserted_at: "2026-08-06T00:00:00Z".to_string(),
    }
}

fn graph() -> StudyGraph {
    StudyGraph {
        schema_version: STUDY_GRAPH_SCHEMA_VERSION.to_string(),
        review_id: "review-1".to_string(),
        reports: vec![report("report-1")],
        studies: vec![Study {
            study_id: "study-1".to_string(),
            report_ids: vec!["report-1".to_string()],
            label: "Study one".to_string(),
            study_design: None,
            registration_ids: vec![],
            notes: vec![],
        }],
        links: vec![link("link-1", "report-1", "study-1")],
    }
}

fn graph_two() -> StudyGraph {
    let mut g = graph();
    g.reports.push(report("report-2"));
    g.studies.push(Study {
        study_id: "study-2".to_string(),
        report_ids: vec!["report-2".to_string()],
        label: "Study two".to_string(),
        study_design: None,
        registration_ids: vec![],
        notes: vec![],
    });
    g.links.push(link("link-2", "report-2", "study-2"));
    g
}

#[test]
fn retrieval_status_defaults_to_not_attempted() {
    let statuses = retrieval_statuses(&graph());
    assert_eq!(
        statuses.get("report-1"),
        Some(&RetrievalStatus::NotAttempted)
    );
}

#[test]
fn retrieval_status_uses_latest_attempt() {
    let mut g = graph();
    let r = g
        .reports
        .first_mut()
        .unwrap_or_else(|| panic!("expected report"));
    r.retrieval_attempts = vec![
        attempt("report-1", RetrievalStatus::NotRetrieved),
        attempt("report-1", RetrievalStatus::Retrieved),
    ];
    let statuses = retrieval_statuses(&g);
    assert_eq!(statuses.get("report-1"), Some(&RetrievalStatus::Retrieved));
}

#[test]
fn reports_per_study_counts_membership() {
    let counts = reports_per_study(&graph_two());
    assert_eq!(counts.get("study-1"), Some(&1));
    assert_eq!(counts.get("study-2"), Some(&1));
}

#[test]
fn unlinked_reports_lists_detached() {
    let mut g = graph();
    g.reports.push(report("report-orphan"));
    assert_eq!(unlinked_reports(&g), vec!["report-orphan"]);
}

#[test]
fn validate_graph_rejects_unknown_link_endpoint() {
    let mut g = graph();
    g.links.push(link("link-x", "report-1", "ghost-study"));
    assert!(matches!(
        validate_graph(&g),
        Err(StudyGraphError::UnknownObject(id)) if id == "ghost-study"
    ));
}

#[test]
fn validate_graph_rejects_membership_without_edge() {
    let mut g = graph();
    g.links.clear();
    assert!(matches!(
        validate_graph(&g),
        Err(StudyGraphError::InconsistentMembership { .. })
    ));
}

#[test]
fn validate_graph_rejects_edge_without_membership() {
    let mut g = graph();
    g.studies
        .first_mut()
        .unwrap_or_else(|| panic!("expected study"))
        .report_ids
        .clear();
    // Contract validation requires non-empty report_ids, so this surfaces as Contract.
    assert!(validate_graph(&g).is_err());
}

#[test]
fn attach_report_duplicate_is_rejected_without_mutation() {
    let mut g = graph();
    let before = g.clone();
    assert!(matches!(
        attach_report(
            &mut g,
            report("report-1"),
            "study-1",
            link("link-9", "report-1", "study-1")
        ),
        Err(StudyGraphError::DuplicateObject(_))
    ));
    assert_eq!(g, before);
}

#[test]
fn attach_report_unknown_study_is_rejected() {
    let mut g = graph();
    assert!(matches!(
        attach_report(
            &mut g,
            report("report-2"),
            "ghost",
            link("link-9", "report-2", "ghost")
        ),
        Err(StudyGraphError::UnknownObject(_))
    ));
}

#[test]
fn attach_report_wrong_link_shape_is_rejected() {
    let mut g = graph();
    let mut bad = link("link-9", "report-2", "study-1");
    bad.relationship = EvidenceRelationship::UpdatesReport;
    assert!(matches!(
        attach_report(&mut g, report("report-2"), "study-1", bad),
        Err(StudyGraphError::InvalidAttachmentLink)
    ));
}

#[test]
fn attach_report_happy_path_appends_membership() {
    let mut g = graph();
    attach_report(
        &mut g,
        report("report-2"),
        "study-1",
        link("link-9", "report-2", "study-1"),
    )
    .unwrap_or_else(|_| panic!("expected attach"));
    assert_eq!(unlinked_reports(&g).len(), 0);
    assert_eq!(reports_per_study(&g).get("study-1"), Some(&2));
}

#[test]
fn link_existing_report_twice_is_duplicate() {
    let mut g = graph();
    assert!(matches!(
        link_report_to_study(
            &mut g,
            "report-1",
            "study-1",
            link("link-9", "report-1", "study-1")
        ),
        Err(StudyGraphError::DuplicateObject(_))
    ));
}

#[test]
fn link_unknown_report_or_study_is_rejected() {
    let mut g = graph();
    assert!(matches!(
        link_report_to_study(
            &mut g,
            "ghost-report",
            "study-1",
            link("link-9", "ghost-report", "study-1")
        ),
        Err(StudyGraphError::UnknownObject(_))
    ));
    assert!(matches!(
        link_report_to_study(
            &mut g,
            "report-1",
            "ghost-study",
            link("link-9", "report-1", "ghost-study")
        ),
        Err(StudyGraphError::UnknownObject(_))
    ));
}

#[test]
fn merge_study_into_itself_is_rejected() {
    let mut g = graph_two();
    assert!(matches!(
        merge_studies(
            &mut g,
            "study-1",
            &["study-1"],
            link("m", "report-1", "study-1")
        ),
        Err(StudyGraphError::InvalidOperation(_))
    ));
}

#[test]
fn merge_unknown_target_and_source_are_rejected() {
    let mut g = graph_two();
    assert!(matches!(
        merge_studies(
            &mut g,
            "ghost",
            &["study-2"],
            link("m", "report-2", "ghost")
        ),
        Err(StudyGraphError::UnknownObject(_))
    ));
    assert!(matches!(
        merge_studies(
            &mut g,
            "study-1",
            &["ghost"],
            link("m", "report-2", "study-1")
        ),
        Err(StudyGraphError::UnknownObject(_))
    ));
}

#[test]
fn merge_with_wrong_evidence_is_rejected() {
    let mut g = graph_two();
    let mut bad = link("m", "report-1", "study-1");
    bad.relationship = EvidenceRelationship::UpdatesReport;
    assert!(matches!(
        merge_studies(&mut g, "study-1", &["study-2"], bad),
        Err(StudyGraphError::InvalidOperationEvidence(_))
    ));
}

#[test]
fn merge_happy_path_repoints_links() {
    let mut g = graph_two();
    merge_studies(
        &mut g,
        "study-1",
        &["study-2"],
        link("m", "report-2", "study-1"),
    )
    .unwrap_or_else(|_| panic!("expected merge"));
    assert_eq!(reports_per_study(&g).get("study-1"), Some(&2));
    assert!(!reports_per_study(&g).contains_key("study-2"));
}

#[test]
fn merge_empty_sources_is_noop() {
    let mut g = graph_two();
    let before = g.clone();
    merge_studies(&mut g, "study-1", &[], link("m", "report-1", "study-1"))
        .unwrap_or_else(|_| panic!("expected merge noop"));
    assert_eq!(g, before);
}

#[test]
fn split_requires_reports_to_move() {
    let mut g = graph_two();
    let new_study = Study {
        study_id: "study-3".to_string(),
        report_ids: vec![],
        label: "Split".to_string(),
        study_design: None,
        registration_ids: vec![],
        notes: vec![],
    };
    assert!(matches!(
        split_study(
            &mut g,
            "study-1",
            new_study,
            &[],
            link("s", "report-1", "study-3")
        ),
        Err(StudyGraphError::InvalidOperation(_))
    ));
}

#[test]
fn split_duplicate_new_study_is_rejected() {
    let mut g = graph_two();
    let dup = Study {
        study_id: "study-1".to_string(),
        report_ids: vec![],
        label: "Dup".to_string(),
        study_design: None,
        registration_ids: vec![],
        notes: vec![],
    };
    assert!(matches!(
        split_study(
            &mut g,
            "study-1",
            dup,
            &["report-1"],
            link("s", "report-1", "study-1")
        ),
        Err(StudyGraphError::DuplicateObject(_))
    ));
}

#[test]
fn detach_sole_report_is_rejected() {
    let mut g = graph();
    let mut ev = link("d", "report-1", "study-1");
    ev.relationship = EvidenceRelationship::Custom("report_detached_from_study".to_string());
    assert!(matches!(
        detach_report(&mut g, "report-1", "study-1", ev),
        Err(StudyGraphError::CannotDetachSoleReport { .. })
    ));
}

#[test]
fn detach_with_wrong_evidence_is_rejected() {
    let mut g = graph_two();
    assert!(matches!(
        detach_report(
            &mut g,
            "report-1",
            "study-1",
            link("d", "report-1", "study-1")
        ),
        Err(StudyGraphError::InvalidOperationEvidence(_))
    ));
}

#[test]
fn detach_happy_path_from_multi_report_study() {
    let mut g = graph();
    attach_report(
        &mut g,
        report("report-2"),
        "study-1",
        link("link-9", "report-2", "study-1"),
    )
    .unwrap_or_else(|_| panic!("expected attach"));
    let mut ev = link("d", "report-2", "study-1");
    ev.relationship = EvidenceRelationship::Custom("report_detached_from_study".to_string());
    detach_report(&mut g, "report-2", "study-1", ev).unwrap_or_else(|_| panic!("expected detach"));
    assert_eq!(unlinked_reports(&g), vec!["report-2"]);
}
