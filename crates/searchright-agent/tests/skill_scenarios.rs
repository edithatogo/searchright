//! Fixture-driven authority scenarios for the systematic-search skill.

use searchright_agent::{
    ApprovalAuthority, ApprovalCheck, AuthorityDecision, OperationRequest, PrincipalKind,
    ProposedOperation, evaluate_operation,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ScenarioSuite {
    schema_version: String,
    status: String,
    claim_boundary: String,
    cases: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    id: String,
    request: OperationRequest,
    authority_record: Option<AuthorityRecord>,
    expected: AuthorityDecision,
}

#[derive(Debug, Deserialize)]
struct AuthorityRecord {
    receipt_id: String,
    review_id: String,
    operation: ProposedOperation,
    principal: PrincipalKind,
    scope_sha256: String,
    status: String,
    preconsumed: bool,
}

struct FixtureAuthority {
    record: Option<AuthorityRecord>,
}

impl ApprovalAuthority for FixtureAuthority {
    fn verify_and_consume(&mut self, check: ApprovalCheck<'_>) -> bool {
        let Some(record) = self.record.as_mut() else {
            return false;
        };
        let receipt_matches = check.approval_receipt_id == record.receipt_id;
        if record.preconsumed
            || record.status != "active"
            || !receipt_matches
            || check.review_id != record.review_id
            || check.operation != record.operation
            || check.principal != record.principal
            || check.scope_sha256 != record.scope_sha256
        {
            return false;
        }
        record.preconsumed = true;
        true
    }
}

#[test]
fn deterministic_authority_scenarios_match_the_real_evaluator()
-> Result<(), Box<dyn std::error::Error>> {
    let suite: ScenarioSuite = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../skills/systematic-search/evaluations/authority-scenarios.json"
    )))?;
    assert_eq!(
        suite.schema_version,
        "org.searchright.agent-scenario-suite.v1"
    );
    assert_eq!(
        suite.status,
        "deterministic_fixture_ready_external_host_model_evaluation_pending"
    );
    assert!(
        suite
            .claim_boundary
            .contains("do not establish model behaviour")
    );
    assert!(suite.cases.len() >= 8);

    let mut identifiers = std::collections::BTreeSet::new();
    for scenario in suite.cases {
        assert!(identifiers.insert(scenario.id.clone()));
        let mut authority = FixtureAuthority {
            record: scenario.authority_record,
        };
        assert_eq!(
            evaluate_operation(&scenario.request, &mut authority),
            scenario.expected,
            "authority scenario {}",
            scenario.id
        );
    }
    Ok(())
}
