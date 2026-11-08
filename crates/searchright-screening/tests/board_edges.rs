//! Screening board edge cases.

use searchright_contracts::{
    AgentAuthority, DecisionValue, ReviewerKind, SCREENING_POLICY_SCHEMA_VERSION,
    ScreeningDecision, ScreeningPolicy, ScreeningRound,
};
use searchright_screening::{ScreeningBoard, ScreeningError, export_covidence_csv};

fn policy() -> ScreeningPolicy {
    ScreeningPolicy {
        schema_version: SCREENING_POLICY_SCHEMA_VERSION.to_string(),
        title_abstract_reviewers: 2,
        full_text_reviewers: 1,
        agent_authority: AgentAuthority::AdvisoryOnly,
        minimum_agent_sensitivity: None,
        independent_blinding: true,
        adjudication_rule: "adjudicator decides".to_string(),
    }
}

fn decision(id: &str, subject: &str, reviewer: &str, decision: DecisionValue) -> ScreeningDecision {
    ScreeningDecision {
        decision_id: id.to_string(),
        review_id: "review-1".to_string(),
        subject_id: subject.to_string(),
        round: ScreeningRound::TitleAbstract,
        reviewer_id: reviewer.to_string(),
        reviewer_kind: ReviewerKind::Human,
        decision,
        exclusion_reason: None,
        confidence: None,
        decided_at: "2026-08-06T00:00:00Z".to_string(),
        rationale: "synthetic".to_string(),
        eligibility_version: "1.0".to_string(),
        agent_provenance: None,
    }
}

#[test]
fn board_rejects_invalid_policy() {
    let mut bad = policy();
    bad.title_abstract_reviewers = 0;
    assert!(ScreeningBoard::new(bad).is_err());
}

#[test]
fn non_advisory_without_sensitivity_fails_contract() {
    use searchright_contracts::Validate;
    let mut bad = policy();
    bad.agent_authority = AgentAuthority::IncludeOnly;
    assert!(bad.validate().is_err());
}

#[test]
fn submit_same_reviewer_twice_is_duplicate() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert!(matches!(
        board.submit(decision("d2", "s1", "r1", DecisionValue::Unclear)),
        Err(ScreeningError::DuplicateReviewer { .. })
    ));
}

#[test]
fn submit_duplicate_decision_id_is_rejected() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert!(matches!(
        board.submit(decision("d1", "s2", "r2", DecisionValue::Include)),
        Err(ScreeningError::DuplicateDecisionId { .. })
    ));
}

#[test]
fn over_quota_human_decisions_are_rejected() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    board
        .submit(decision("d2", "s1", "r2", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert!(matches!(
        board.submit(decision("d3", "s1", "r3", DecisionValue::Include)),
        Err(ScreeningError::IndependentReviewComplete)
    ));
}

#[test]
fn role_restricted_reviewer_cannot_submit() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board.assign_role("r1", searchright_screening::ReviewerRole::Adjudicator);
    assert!(matches!(
        board.submit(decision("d1", "s1", "r1", DecisionValue::Include)),
        Err(ScreeningError::UnauthorizedRole { .. })
    ));
}

#[test]
fn status_pending_then_consensus_then_conflict() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert!(matches!(
        board.status("s1", ScreeningRound::TitleAbstract),
        searchright_screening::ScreeningStatus::Pending {
            submitted: 1,
            required: 2
        }
    ));
    board
        .submit(decision("d2", "s1", "r2", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert_eq!(
        board.status("s1", ScreeningRound::TitleAbstract),
        searchright_screening::ScreeningStatus::Consensus(DecisionValue::Include)
    );
    board
        .submit(decision("d3", "s2", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    board
        .submit(decision("d4", "s2", "r2", DecisionValue::Unclear))
        .unwrap_or_else(|_| panic!("expected submit"));
    assert_eq!(
        board.status("s2", ScreeningRound::TitleAbstract),
        searchright_screening::ScreeningStatus::Conflict
    );
}

#[test]
fn decisions_for_reviewer_blinds_others_while_pending() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    let views = board.decisions_for_reviewer("r2", Some(ScreeningRound::TitleAbstract));
    let first = views.first().unwrap_or_else(|| panic!("expected view"));
    assert!(first.is_blinded);
    assert!(first.decision.is_none());
    let own = board.decisions_for_reviewer("r1", Some(ScreeningRound::TitleAbstract));
    let first_own = own.first().unwrap_or_else(|| panic!("expected view"));
    assert!(!first_own.is_blinded);
}

#[test]
fn policy_admin_sees_unblinded() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    board.assign_role("admin", searchright_screening::ReviewerRole::PolicyAdmin);
    board
        .submit(decision("d1", "s1", "r1", DecisionValue::Include))
        .unwrap_or_else(|_| panic!("expected submit"));
    let views = board.decisions_for_reviewer("admin", Some(ScreeningRound::TitleAbstract));
    assert!(
        !views
            .first()
            .unwrap_or_else(|| panic!("expected view"))
            .is_blinded
    );
}

#[test]
fn bulk_preview_rejects_empty() {
    let board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    assert!(matches!(
        board.preview_bulk(vec![]),
        Err(ScreeningError::EmptyBulkSubmission)
    ));
}

#[test]
fn bulk_preview_then_apply_happy_path() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    let preview = board
        .preview_bulk(vec![decision("d1", "s1", "r1", DecisionValue::Include)])
        .unwrap_or_else(|_| panic!("expected preview"));
    assert_eq!(preview.accepted(), 1);
    assert_eq!(
        board.status("s1", ScreeningRound::TitleAbstract),
        searchright_screening::ScreeningStatus::Pending {
            submitted: 0,
            required: 2
        }
    );
    board
        .apply_bulk(preview)
        .unwrap_or_else(|_| panic!("expected apply"));
    assert_eq!(
        board.status("s1", ScreeningRound::TitleAbstract),
        searchright_screening::ScreeningStatus::Pending {
            submitted: 1,
            required: 2
        }
    );
}

#[test]
fn export_csv_escapes_commas_and_quotes() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    let mut d = decision("d1", "s1", "r1", DecisionValue::Include);
    d.rationale = "with, comma and \"quote\"".to_string();
    board
        .submit(d)
        .unwrap_or_else(|_| panic!("expected submit"));
    let csv = export_covidence_csv(&board);
    assert!(csv.contains("\"with, comma"));
}

#[test]
fn invalid_decision_timestamp_fails_contract() {
    let mut board = ScreeningBoard::new(policy()).unwrap_or_else(|_| panic!("expected board"));
    let mut d = decision("d1", "s1", "r1", DecisionValue::Include);
    d.decided_at = String::new();
    assert!(board.submit(d).is_err());
}
