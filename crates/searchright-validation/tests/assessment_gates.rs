//! Gate tests for search-validation assessment (Track 16 coverage sprint).
//!
//! Synthetic reports exercise each `ready` conjunction term independently so
//! a regression in any gate fails closed rather than silently approving.

use searchright_contracts::{
    FindingSeverity, PressElement, PressFinding, PressReview, PressReviewDecision,
    SEARCH_VALIDATION_SCHEMA_VERSION, SearchValidationReport, SeedRecord,
    TranslationLossAssessment,
};
use searchright_validation::{assess, press_domains_covered, seed_recall, translation_acceptable};

static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn finding(element: PressElement, severity: FindingSeverity, resolved: bool) -> PressFinding {
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    PressFinding {
        finding_id: format!("finding-{element:?}-{n}"),
        element,
        severity,
        message: "synthetic finding".to_string(),
        recommendation: "synthetic correction".to_string(),
        resolved,
    }
}

fn press_review() -> PressReview {
    PressReview {
        press_review_id: "press-1".to_string(),
        strategy_id: "strategy-1".to_string(),
        strategy_version: "1".to_string(),
        reviewer_id: "press-reviewer".to_string(),
        reviewed_at: "2026-08-06T00:00:00Z".to_string(),
        findings: vec![
            finding(
                PressElement::TranslationOfQuestion,
                FindingSeverity::Note,
                true,
            ),
            finding(
                PressElement::BooleanAndProximity,
                FindingSeverity::Note,
                true,
            ),
            finding(PressElement::SubjectHeadings, FindingSeverity::Note, true),
            finding(PressElement::TextWords, FindingSeverity::Note, true),
            finding(
                PressElement::SpellingSyntaxAndLines,
                FindingSeverity::Note,
                true,
            ),
            finding(PressElement::LimitsAndFilters, FindingSeverity::Note, true),
        ],
        decision: PressReviewDecision::Approved,
    }
}

fn seed(id: &str, retrieved: bool) -> SeedRecord {
    SeedRecord {
        seed_id: id.to_string(),
        identifier: format!("doi:10.0000/{id}"),
        relevance_basis: "synthetic seed".to_string(),
        retrieved,
        source_id: "src-1".to_string(),
    }
}

fn assessment(observed: u32, maximum: u32, approved: bool) -> TranslationLossAssessment {
    TranslationLossAssessment {
        strategy_id: "strategy-1".to_string(),
        target_dialect: "pubmed".to_string(),
        maximum_material_warnings: maximum,
        observed_material_warnings: observed,
        human_approved: approved,
        notes: vec![],
    }
}

fn ready_report() -> SearchValidationReport {
    SearchValidationReport {
        schema_version: SEARCH_VALIDATION_SCHEMA_VERSION.to_string(),
        review_id: "review-1".to_string(),
        strategy_id: "strategy-1".to_string(),
        strategy_version: "1".to_string(),
        press_reviews: vec![press_review()],
        seed_records: vec![seed("s1", true), seed("s2", true)],
        translation_assessments: vec![assessment(0, 2, true)],
        minimum_seed_recall: Some(1.0),
        approved_for_execution: true,
        approved_by: Some("execution-approver".to_string()),
    }
}

fn first_review(report: &mut SearchValidationReport) -> &mut PressReview {
    report
        .press_reviews
        .first_mut()
        .unwrap_or_else(|| panic!("expected PRESS review"))
}

#[test]
fn complete_report_with_all_gates_passing_is_ready() {
    let summary = assess(&ready_report()).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(summary.ready);
    assert_eq!(summary.press_domains_covered, 6);
    assert!(summary.press_review_complete);
    assert!(summary.press_decisions_approved);
    assert_eq!(summary.unresolved_major_findings, 0);
    assert_eq!(summary.unacceptable_translations, 0);
    assert_eq!(summary.seed_recall.recall, Some(1.0));
}

#[test]
fn empty_report_fails_closed() {
    let mut report = ready_report();
    report.press_reviews.clear();
    report.approved_for_execution = false;
    report.approved_by = None;
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!summary.ready);
    assert_eq!(summary.press_domains_covered, 0);
}

#[test]
fn seed_recall_counts_retrieved() {
    let mut report = ready_report();
    report.seed_records = vec![seed("s1", true), seed("s2", false), seed("s3", true)];
    let recall = seed_recall(&report);
    assert_eq!(recall.retrieved, 2);
    assert_eq!(recall.total, 3);
    assert!((recall.recall.unwrap_or(0.0) - 2.0 / 3.0).abs() < 1e-12);
}

#[test]
fn seed_recall_empty_is_none() {
    let mut report = ready_report();
    report.seed_records.clear();
    let recall = seed_recall(&report);
    assert_eq!(recall.recall, None);
    assert_eq!(recall.total, 0);
}

#[test]
fn seed_recall_threshold_blocks_execution() {
    let mut report = ready_report();
    report.seed_records = vec![seed("s1", true), seed("s2", false)];
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!summary.ready);
}

#[test]
fn accepted_minor_findings_do_not_block_but_major_does() {
    let mut advisory = ready_report();
    first_review(&mut advisory).findings.push(finding(
        PressElement::TextWords,
        FindingSeverity::Advisory,
        false,
    ));
    let advisory_summary = assess(&advisory).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(advisory_summary.ready);
    assert_eq!(advisory_summary.unresolved_major_findings, 0);

    let mut major = ready_report();
    first_review(&mut major).findings.push(finding(
        PressElement::BooleanAndProximity,
        FindingSeverity::Major,
        false,
    ));
    let major_summary = assess(&major).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!major_summary.ready);
    assert_eq!(major_summary.unresolved_major_findings, 1);

    let mut resolved = major;
    for f in &mut first_review(&mut resolved).findings {
        f.resolved = true;
    }
    let resolved_summary = assess(&resolved).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(resolved_summary.ready);
}

#[test]
fn duplicate_press_elements_do_not_inflate_coverage() {
    let mut report = ready_report();
    first_review(&mut report).findings = vec![
        finding(PressElement::TextWords, FindingSeverity::Note, true),
        PressFinding {
            finding_id: "finding-dup".to_string(),
            element: PressElement::TextWords,
            severity: FindingSeverity::Note,
            message: "synthetic finding".to_string(),
            recommendation: "synthetic correction".to_string(),
            resolved: true,
        },
    ];
    assert_eq!(press_domains_covered(&report), 1);
}

#[test]
fn translation_loss_budget_is_enforced() {
    assert!(translation_acceptable(&assessment(2, 2, true)));
    assert!(!translation_acceptable(&assessment(3, 2, true)));
    assert!(!translation_acceptable(&assessment(0, 2, false)));
}

#[test]
fn unacceptable_translation_blocks_ready() {
    let mut report = ready_report();
    report.translation_assessments = vec![assessment(5, 2, true)];
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!summary.ready);
    assert_eq!(summary.unacceptable_translations, 1);
}

#[test]
fn missing_human_approval_blocks_ready() {
    let mut report = ready_report();
    report.approved_for_execution = false;
    report.approved_by = None;
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!summary.ready);
}

#[test]
fn rejected_press_decision_blocks_ready() {
    let mut report = ready_report();
    first_review(&mut report).decision = PressReviewDecision::Rejected;
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(!summary.ready);
    assert!(!summary.press_decisions_approved);
}

#[test]
fn self_approval_error_propagates_as_contract_error() {
    let mut report = ready_report();
    report.approved_by = Some("press-reviewer".to_string());
    assert!(assess(&report).is_err());
}

#[test]
fn strategy_mismatch_propagates_as_contract_error() {
    let mut report = ready_report();
    first_review(&mut report).strategy_id = "other-strategy".to_string();
    assert!(assess(&report).is_err());
}

#[test]
fn approved_with_advisory_still_permits_execution() {
    let mut report = ready_report();
    first_review(&mut report).decision = PressReviewDecision::ApprovedWithAdvisory;
    let summary = assess(&report).unwrap_or_else(|_| panic!("expected assessment"));
    assert!(summary.ready);
}
