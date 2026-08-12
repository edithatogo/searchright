//! Default-deny access decisions for authenticated remote Searchright services.
//!
//! Replay checks are available through [`authorise_with_replay`] and
//! [`ReplayLedger`]. The ledger records request identifiers, not bearer-token
//! identifiers: `OAuth` access tokens are legitimately reused until expiry while
//! an individual consequential request must not be replayed. The ledger is
//! deliberately small and in-memory, so it only protects one process.
//! Distributed deployments must use a shared store with equivalent uniqueness
//! and expiry semantics.

#![forbid(unsafe_code)]

use std::collections::{HashSet, VecDeque};

use searchright_contracts::{
    ACCESS_DECISION_SCHEMA_VERSION, AccessDecision, AccessRequest, AccessScope, PrincipalKind,
    TenantPolicy, Validate,
};

const DEFAULT_REPLAY_LEDGER_ENTRIES: usize = 4096;

/// Bounded record of recently accepted request identifiers, used to reject replays.
#[derive(Debug)]
pub struct ReplayLedger {
    maximum_entries: usize,
    entries: HashSet<String>,
    order: VecDeque<String>,
}

impl ReplayLedger {
    /// Create a ledger retaining up to `maximum_entries` accepted request identifiers.
    #[must_use]
    pub fn with_capacity(maximum_entries: usize) -> Self {
        Self {
            maximum_entries: maximum_entries.max(1),
            entries: HashSet::new(),
            order: VecDeque::new(),
        }
    }

    /// Return whether the request identifier is currently recorded.
    #[must_use]
    pub fn has_seen(&self, request_id: &str) -> bool {
        self.entries.contains(request_id)
    }

    /// Record an accepted request identifier, evicting the oldest entry when full.
    pub fn record_accepted(&mut self, request_id: &str) {
        if self.entries.contains(request_id) {
            return;
        }
        while self.order.len() >= self.maximum_entries {
            if let Some(expired) = self.order.pop_front() {
                self.entries.remove(&expired);
            }
        }
        let request_id = request_id.to_owned();
        self.entries.insert(request_id.clone());
        self.order.push_back(request_id);
    }
}

impl Default for ReplayLedger {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_REPLAY_LEDGER_ENTRIES)
    }
}

/// Evaluate one request under a tenant policy.
pub fn authorise(
    policy: &TenantPolicy,
    request: &AccessRequest,
) -> Result<AccessDecision, AccessError> {
    let blockers = evaluate(policy, request)?;
    Ok(decision(policy, request, blockers))
}

/// Evaluate one request and reject request identifiers already accepted by the ledger.
pub fn authorise_with_replay(
    policy: &TenantPolicy,
    request: &AccessRequest,
    ledger: &mut ReplayLedger,
) -> Result<AccessDecision, AccessError> {
    let mut blockers = evaluate(policy, request)?;
    if ledger.has_seen(&request.request_id) {
        blockers.push("access.replay.request_reused".to_owned());
    }
    let decision = decision(policy, request, blockers);
    if decision.permitted {
        ledger.record_accepted(&request.request_id);
    }
    Ok(decision)
}

fn evaluate(policy: &TenantPolicy, request: &AccessRequest) -> Result<Vec<String>, AccessError> {
    policy.validate()?;
    request.validate()?;
    let mut blockers = Vec::new();
    if !request.authenticated {
        blockers.push("access.authentication.required".to_owned());
    }
    if request.tenant_id != policy.tenant_id {
        blockers.push("access.tenant.mismatch".to_owned());
    }
    if !policy
        .allowed_regions
        .iter()
        .any(|region| region == &request.region)
    {
        blockers.push("access.region.denied".to_owned());
    }
    for scope in &request.scopes {
        if !policy.allowed_scopes.contains(scope) {
            blockers.push(format!("access.scope.denied.{}", scope_code(*scope)));
        }
    }
    if request.external_write
        && (!request.scopes.contains(&AccessScope::ExternalWrite) || !request.human_approval)
    {
        blockers.push("access.external_write.requires_scope_and_human_approval".to_owned());
    }
    if request.final_eligibility_decision
        && (request.principal_kind != PrincipalKind::Human
            || !request.scopes.contains(&AccessScope::ScreeningDecide)
            || !request.human_approval)
    {
        blockers.push("access.final_decision.human_only".to_owned());
    }
    Ok(blockers)
}

const fn scope_code(scope: AccessScope) -> &'static str {
    match scope {
        AccessScope::ReviewRead => "review_read",
        AccessScope::ReviewWrite => "review_write",
        AccessScope::SearchExecute => "search_execute",
        AccessScope::ScreeningRecommend => "screening_recommend",
        AccessScope::ScreeningDecide => "screening_decide",
        AccessScope::TenantAdmin => "tenant_admin",
        AccessScope::ExternalWrite => "external_write",
    }
}

fn decision(
    policy: &TenantPolicy,
    request: &AccessRequest,
    blockers: Vec<String>,
) -> AccessDecision {
    let human_approval_required = blockers
        .iter()
        .any(|code| code.contains("external_write") || code.contains("final_decision"));
    AccessDecision {
        schema_version: ACCESS_DECISION_SCHEMA_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        tenant_id: policy.tenant_id.clone(),
        permitted: blockers.is_empty(),
        blockers,
        human_approval_required,
    }
}

/// Access-policy error.
#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use searchright_contracts::{ACCESS_REQUEST_SCHEMA_VERSION, TENANT_POLICY_SCHEMA_VERSION};

    fn policy() -> TenantPolicy {
        TenantPolicy {
            schema_version: TENANT_POLICY_SCHEMA_VERSION.to_owned(),
            tenant_id: "tenant-demo".to_owned(),
            allowed_regions: vec!["AU".to_owned()],
            allowed_scopes: vec![
                AccessScope::ReviewRead,
                AccessScope::ReviewWrite,
                AccessScope::SearchExecute,
                AccessScope::ScreeningRecommend,
                AccessScope::ScreeningDecide,
                AccessScope::ExternalWrite,
            ],
            maximum_concurrent_tasks: 4,
            external_model_processing_allowed: false,
            restricted_full_text_persistence_allowed: false,
            cross_tenant_aggregation_allowed: false,
            approved_by: "Governance officer".to_owned(),
            policy_version: "1".to_owned(),
        }
    }

    fn request() -> AccessRequest {
        AccessRequest {
            schema_version: ACCESS_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: "access-demo-1".to_owned(),
            principal_id: "reviewer-1".to_owned(),
            principal_kind: PrincipalKind::Human,
            tenant_id: "tenant-demo".to_owned(),
            scopes: vec![AccessScope::ReviewRead, AccessScope::ScreeningDecide],
            region: "AU".to_owned(),
            authenticated: true,
            external_write: false,
            final_eligibility_decision: true,
            human_approval: true,
        }
    }

    fn decision_for(request: &AccessRequest) -> AccessDecision {
        let Ok(decision) = authorise(&policy(), request) else {
            panic!("access decision should be produced");
        };
        decision
    }

    fn assert_blocked(decision: &AccessDecision, code: &str) {
        assert!(!decision.permitted);
        assert!(decision.blockers.iter().any(|blocker| blocker == code));
    }

    #[test]
    fn replayed_request_id_is_denied_on_second_use() {
        let policy = policy();
        let request = request();
        let mut ledger = ReplayLedger::default();
        let Ok(first) = authorise_with_replay(&policy, &request, &mut ledger) else {
            panic!("first replay-aware decision should be produced");
        };
        assert!(first.permitted);
        let Ok(second) = authorise_with_replay(&policy, &request, &mut ledger) else {
            panic!("second replay-aware decision should be produced");
        };
        assert_blocked(&second, "access.replay.request_reused");
    }

    #[test]
    fn cross_tenant_request_is_denied() {
        let mut request = request();
        request.tenant_id = "tenant-other".to_owned();
        assert_blocked(&decision_for(&request), "access.tenant.mismatch");
    }

    #[test]
    fn disallowed_region_is_denied() {
        let mut request = request();
        request.region = "EU".to_owned();
        assert_blocked(&decision_for(&request), "access.region.denied");
    }

    #[test]
    fn unrequested_scope_is_denied() {
        let mut request = request();
        request.scopes.push(AccessScope::TenantAdmin);
        assert_blocked(&decision_for(&request), "access.scope.denied.tenant_admin");
    }

    #[test]
    fn external_write_without_human_approval_is_denied() {
        let mut request = request();
        request.external_write = true;
        request.scopes.push(AccessScope::ExternalWrite);
        request.human_approval = false;
        let decision = decision_for(&request);
        assert_blocked(
            &decision,
            "access.external_write.requires_scope_and_human_approval",
        );
        assert!(decision.human_approval_required);
    }

    #[test]
    fn final_eligibility_decision_by_non_human_is_denied() {
        let mut request = request();
        request.principal_kind = PrincipalKind::Service;
        let decision = decision_for(&request);
        assert_blocked(&decision, "access.final_decision.human_only");
        assert!(decision.human_approval_required);
    }

    #[test]
    fn multiple_control_violations_accumulate_blockers() {
        let mut request = request();
        request.authenticated = false;
        request.tenant_id = "tenant-other".to_owned();
        request.region = "EU".to_owned();
        request.scopes.push(AccessScope::TenantAdmin);
        request.external_write = true;
        request.human_approval = false;
        request.principal_kind = PrincipalKind::Service;
        let decision = decision_for(&request);
        for code in [
            "access.authentication.required",
            "access.tenant.mismatch",
            "access.region.denied",
            "access.scope.denied.tenant_admin",
            "access.external_write.requires_scope_and_human_approval",
            "access.final_decision.human_only",
        ] {
            assert!(decision.blockers.iter().any(|blocker| blocker == code));
        }
    }

    #[test]
    fn fully_compliant_request_is_permitted() {
        let decision = decision_for(&request());
        assert!(decision.permitted);
        assert!(decision.blockers.is_empty());
        assert!(!decision.human_approval_required);
    }
}
