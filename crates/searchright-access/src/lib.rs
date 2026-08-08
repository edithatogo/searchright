//! Default-deny access decisions for authenticated remote Searchright services.

#![forbid(unsafe_code)]

use searchright_contracts::{
    ACCESS_DECISION_SCHEMA_VERSION, AccessDecision, AccessRequest, AccessScope, PrincipalKind,
    TenantPolicy, Validate,
};

/// Evaluate one request under a tenant policy.
pub fn authorise(policy: &TenantPolicy, request: &AccessRequest) -> Result<AccessDecision, AccessError> {
    policy.validate()?;
    request.validate()?;
    let mut blockers = Vec::new();
    if !request.authenticated {
        blockers.push("access.authentication.required".to_owned());
    }
    if request.tenant_id != policy.tenant_id {
        blockers.push("access.tenant.mismatch".to_owned());
    }
    if !policy.allowed_regions.iter().any(|region| region == &request.region) {
        blockers.push("access.region.denied".to_owned());
    }
    for scope in &request.scopes {
        if !policy.allowed_scopes.contains(scope) {
            blockers.push(format!("access.scope.denied.{scope:?}").to_ascii_lowercase());
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
    let human_approval_required = blockers.iter().any(|code| {
        code.contains("external_write") || code.contains("final_decision")
    });
    Ok(AccessDecision {
        schema_version: ACCESS_DECISION_SCHEMA_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        tenant_id: policy.tenant_id.clone(),
        permitted: blockers.is_empty(),
        blockers,
        human_approval_required,
    })
}

/// Access-policy error.
#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
}
