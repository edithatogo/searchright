//! Multi-cycle living review and protocol amendment scenario integration tests.

use searchright_contracts::{
    AmendmentChange, AmendmentDecision, AmendmentKind, BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION,
    BibliographicRecord, LivingUpdateRun, PROTOCOL_AMENDMENT_SCHEMA_VERSION, ProtocolAmendment,
    RecordChangeKind, RecordIdentifiers, RecordKind, UpdateCursor, UpdateRunStatus, UpdateStrategy,
    Validate,
};
use searchright_living::{diff_record_sets, record_digest, validate_lineage};

fn test_record(id: &str, title: &str, year: Option<i32>) -> BibliographicRecord {
    BibliographicRecord {
        schema_version: BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_string(),
        record_id: id.to_string(),
        source_receipt_id: format!("receipt-{id}"),
        native_id: id.to_string(),
        kind: RecordKind::JournalArticle,
        identifiers: RecordIdentifiers::default(),
        title: title.to_string(),
        abstract_text: Some(format!("Abstract for {title}")),
        authors: vec!["Author, A.".to_string()],
        container_title: Some("Journal of Systematic Reviews".to_string()),
        publication_year: year,
        publication_date: year.map(|y| format!("{y}-01-01")),
        languages: vec!["eng".to_string()],
        subjects: vec!["Evidence Synthesis".to_string()],
        urls: vec![format!("https://example.org/records/{id}")],
        provider_metadata: serde_json::json!({"mesh_terms": ["Meta-Analysis as Topic"]}),
    }
}

#[test]
fn test_multi_cycle_living_review_lineage_and_supersession() {
    // Strategy classification checks
    let date_cursor = UpdateCursor {
        provider_id: "pubmed".to_string(),
        cursor_kind: "date".to_string(),
        value: "2026-01-01".to_string(),
        retrieved_through: None,
    };
    assert_eq!(date_cursor.strategy(), Some(UpdateStrategy::Date));

    let token_cursor = UpdateCursor {
        provider_id: "openalex".to_string(),
        cursor_kind: "cursor".to_string(),
        value: "token-12345".to_string(),
        retrieved_through: None,
    };
    assert_eq!(token_cursor.strategy(), Some(UpdateStrategy::Cursor));

    // Cycle 0: Initial baseline synthesis run
    let baseline_run = LivingUpdateRun {
        schema_version: "org.searchright.living-update.v1".to_string(),
        run_id: "run-cycle-00-baseline".to_string(),
        review_id: "review-sr-2026-living-01".to_string(),
        parent_run_id: None,
        supersedes_run_id: None,
        protocol_version: "1.0.0".to_string(),
        status: UpdateRunStatus::Completed,
        started_at: "2026-01-01T00:00:00Z".to_string(),
        completed_at: Some("2026-01-01T01:00:00Z".to_string()),
        cursors_before: vec![],
        cursors_after: vec![date_cursor.clone()],
        changes: vec![],
        requires_human_release: false,
    };
    if let Err(e) = baseline_run.validate() {
        panic!("baseline run must validate: {e}");
    }

    // Cycle 1: Update run 6 months later
    let cycle_1_run = LivingUpdateRun {
        schema_version: "org.searchright.living-update.v1".to_string(),
        run_id: "run-cycle-01-update".to_string(),
        review_id: "review-sr-2026-living-01".to_string(),
        parent_run_id: Some("run-cycle-00-baseline".to_string()),
        supersedes_run_id: None,
        protocol_version: "1.0.0".to_string(),
        status: UpdateRunStatus::Completed,
        started_at: "2026-07-01T00:00:00Z".to_string(),
        completed_at: Some("2026-07-01T01:00:00Z".to_string()),
        cursors_before: vec![date_cursor],
        cursors_after: vec![UpdateCursor {
            provider_id: "pubmed".to_string(),
            cursor_kind: "date".to_string(),
            value: "2026-07-01".to_string(),
            retrieved_through: None,
        }],
        changes: vec![],
        requires_human_release: false,
    };
    if let Err(e) = cycle_1_run.validate() {
        panic!("cycle 1 run must validate: {e}");
    }

    // Cycle 2: Major synthesis update that supersedes baseline
    let cycle_2_run = LivingUpdateRun {
        schema_version: "org.searchright.living-update.v1".to_string(),
        run_id: "run-cycle-02-supersede".to_string(),
        review_id: "review-sr-2026-living-01".to_string(),
        parent_run_id: Some("run-cycle-01-update".to_string()),
        supersedes_run_id: Some("run-cycle-00-baseline".to_string()),
        protocol_version: "1.1.0".to_string(),
        status: UpdateRunStatus::Completed,
        started_at: "2027-01-01T00:00:00Z".to_string(),
        completed_at: Some("2027-01-01T02:00:00Z".to_string()),
        cursors_before: vec![UpdateCursor {
            provider_id: "pubmed".to_string(),
            cursor_kind: "date".to_string(),
            value: "2026-07-01".to_string(),
            retrieved_through: None,
        }],
        cursors_after: vec![UpdateCursor {
            provider_id: "pubmed".to_string(),
            cursor_kind: "date".to_string(),
            value: "2027-01-01".to_string(),
            retrieved_through: None,
        }],
        changes: vec![],
        requires_human_release: false,
    };
    if let Err(e) = cycle_2_run.validate() {
        panic!("cycle 2 run must validate: {e}");
    }

    // Multi-cycle lineage validation
    let lineage = vec![baseline_run, cycle_1_run, cycle_2_run];
    if let Err(e) = validate_lineage(&lineage) {
        panic!("multi-cycle lineage must be valid and acyclic: {e}");
    }
}

#[test]
fn test_change_attribution_reproduction_and_human_release() {
    let initial_set = vec![
        test_record(
            "rec-001",
            "Effect of intervention A on outcomes",
            Some(2025),
        ),
        test_record("rec-002", "Long-term follow-up of cohort B", Some(2024)),
    ];
    let updated_set = vec![
        // rec-001 unchanged
        test_record(
            "rec-001",
            "Effect of intervention A on outcomes",
            Some(2025),
        ),
        // rec-002 updated title
        test_record(
            "rec-002",
            "Long-term follow-up of cohort B: Revised",
            Some(2024),
        ),
        // rec-003 added
        test_record("rec-003", "Novel randomized trial of drug C", Some(2026)),
    ];

    let changes = match diff_record_sets(&initial_set, &updated_set) {
        Ok(c) => c,
        Err(e) => panic!("diffing record sets should succeed: {e}"),
    };

    assert_eq!(
        changes.len(),
        2,
        "Expected exactly 2 changes (1 update, 1 addition)"
    );

    let rec_002_change = match changes.iter().find(|c| c.record_id == "rec-002") {
        Some(c) => c,
        None => panic!("rec-002 change must be found"),
    };
    assert_eq!(rec_002_change.kind, RecordChangeKind::Updated);
    assert!(rec_002_change.before_digest.is_some());
    assert!(rec_002_change.after_digest.is_some());
    assert_ne!(rec_002_change.before_digest, rec_002_change.after_digest);

    let rec_003_change = match changes.iter().find(|c| c.record_id == "rec-003") {
        Some(c) => c,
        None => panic!("rec-003 change must be found"),
    };
    assert_eq!(rec_003_change.kind, RecordChangeKind::Added);
    assert!(rec_003_change.before_digest.is_none());

    let rec_003_digest = match record_digest(&updated_set[2]) {
        Ok(d) => d,
        Err(e) => panic!("rec-003 digest calculation must succeed: {e}"),
    };
    assert_eq!(
        rec_003_change.after_digest.as_deref(),
        Some(rec_003_digest.as_str())
    );

    // Safeguard verification: When changes exist, requires_human_release must be true
    let run_with_changes_safe = LivingUpdateRun {
        schema_version: "org.searchright.living-update.v1".to_string(),
        run_id: "run-with-changes".to_string(),
        review_id: "rev-1".to_string(),
        parent_run_id: None,
        supersedes_run_id: None,
        protocol_version: "1.0.0".to_string(),
        status: UpdateRunStatus::Completed,
        started_at: "2026-06-01T00:00:00Z".to_string(),
        completed_at: Some("2026-06-01T01:00:00Z".to_string()),
        cursors_before: vec![],
        cursors_after: vec![],
        changes: changes.clone(),
        requires_human_release: true,
    };
    assert!(run_with_changes_safe.validate().is_ok());

    let run_with_changes_unsafe = LivingUpdateRun {
        schema_version: "org.searchright.living-update.v1".to_string(),
        run_id: "run-with-changes-unsafe".to_string(),
        review_id: "rev-1".to_string(),
        parent_run_id: None,
        supersedes_run_id: None,
        protocol_version: "1.0.0".to_string(),
        status: UpdateRunStatus::Completed,
        started_at: "2026-06-01T00:00:00Z".to_string(),
        completed_at: Some("2026-06-01T01:00:00Z".to_string()),
        cursors_before: vec![],
        cursors_after: vec![],
        changes,
        requires_human_release: false, // Forbidden: changes without human release
    };
    assert!(
        run_with_changes_unsafe.validate().is_err(),
        "Living update run with changes must reject automated inclusion without human release"
    );
}

#[test]
fn test_protocol_amendment_lifecycle_and_approval_safeguards() {
    let amendment = ProtocolAmendment {
        schema_version: PROTOCOL_AMENDMENT_SCHEMA_VERSION.to_string(),
        amendment_id: "amend-2026-001".to_string(),
        review_id: "rev-1".to_string(),
        kind: AmendmentKind::Eligibility,
        version_before: "1.0.0".to_string(),
        version_after: "1.1.0".to_string(),
        proposed_by: "edithatogo".to_string(),
        proposed_at: "2026-06-01T10:00:00Z".to_string(),
        decision: AmendmentDecision::Approved,
        decided_by: Some("principal-investigator-01".to_string()),
        decided_at: Some("2026-06-02T14:30:00Z".to_string()),
        changes: vec![AmendmentChange {
            path: "/eligibility/date_range/end".to_string(),
            before: Some("2025-12-31".to_string()),
            after: "2026-12-31".to_string(),
            rationale: "Extend surveillance window for annual living review update".to_string(),
        }],
        retrospective_impact: "Identified 42 new studies published during the extended window; existing 310 screened studies remain valid with no protocol drift.".to_string(),
        requires_reprocessing: true,
    };

    if let Err(e) = amendment.validate() {
        panic!("protocol amendment must validate successfully: {e}");
    }
}
