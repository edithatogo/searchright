//! Authentication, authorisation, tenancy and data-residency contracts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ACCESS_DECISION_SCHEMA_VERSION, ACCESS_REQUEST_SCHEMA_VERSION, ContractError,
    TENANT_POLICY_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Authenticated principal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    /// Human reviewer or administrator.
    Human,
    /// Service identity.
    Service,
    /// Bounded agent identity acting for a human or service.
    Agent,
}

/// Stable capability scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessScope {
    /// Read review contracts and metadata.
    ReviewRead,
    /// Write local review artefacts.
    ReviewWrite,
    /// Execute bounded provider searches.
    SearchExecute,
    /// Record screening recommendations.
    ScreeningRecommend,
    /// Record human screening decisions.
    ScreeningDecide,
    /// Manage tenant policy.
    TenantAdmin,
    /// Perform an external write after preview and approval.
    ExternalWrite,
}

/// Tenant isolation and residency policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TenantPolicy {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable tenant identifier.
    pub tenant_id: String,
    /// Permitted deployment/data regions.
    pub allowed_regions: Vec<String>,
    /// Permitted scopes.
    pub allowed_scopes: Vec<AccessScope>,
    /// Maximum concurrent long-running tasks.
    pub maximum_concurrent_tasks: u32,
    /// Whether external model processing is permitted.
    pub external_model_processing_allowed: bool,
    /// Whether restricted full text may be persisted.
    pub restricted_full_text_persistence_allowed: bool,
    /// Whether cross-tenant aggregation is permitted.
    pub cross_tenant_aggregation_allowed: bool,
    /// Human/institutional approver.
    pub approved_by: String,
    /// Policy version.
    pub policy_version: String,
}

/// One authenticated authorisation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccessRequest {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable request identifier.
    pub request_id: String,
    /// Principal identifier.
    pub principal_id: String,
    /// Principal kind.
    pub principal_kind: PrincipalKind,
    /// Tenant boundary.
    pub tenant_id: String,
    /// Requested scopes.
    pub scopes: Vec<AccessScope>,
    /// Requested region.
    pub region: String,
    /// Whether the identity was authenticated by the configured verifier.
    pub authenticated: bool,
    /// Whether an operation would write to an external system.
    pub external_write: bool,
    /// Whether the operation would make a final eligibility decision.
    pub final_eligibility_decision: bool,
    /// Whether a human explicitly approved this request.
    pub human_approval: bool,
}

/// Authorisation decision with stable blockers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccessDecision {
    /// Contract identifier.
    pub schema_version: String,
    /// Request identifier.
    pub request_id: String,
    /// Policy tenant identifier.
    pub tenant_id: String,
    /// Whether the request is permitted.
    pub permitted: bool,
    /// Stable denial or review codes.
    pub blockers: Vec<String>,
    /// Whether additional human approval is required.
    pub human_approval_required: bool,
}

impl Validate for TenantPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(&self.schema_version, TENANT_POLICY_SCHEMA_VERSION, "tenant_policy.schema_version")?;
        require_text(&self.tenant_id, "tenant_policy.tenant_id")?;
        require_text(&self.approved_by, "tenant_policy.approved_by")?;
        require_text(&self.policy_version, "tenant_policy.policy_version")?;
        if self.allowed_regions.is_empty() || self.allowed_scopes.is_empty() || self.maximum_concurrent_tasks == 0 {
            return Err(ContractError::Invariant("tenant policy requires regions, scopes and a positive task bound".to_owned()));
        }
        if self.cross_tenant_aggregation_allowed {
            return Err(ContractError::Invariant("cross-tenant aggregation is not permitted by the v1 policy".to_owned()));
        }
        Ok(())
    }
}

impl Validate for AccessRequest {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(&self.schema_version, ACCESS_REQUEST_SCHEMA_VERSION, "access_request.schema_version")?;
        require_text(&self.request_id, "access_request.request_id")?;
        require_text(&self.principal_id, "access_request.principal_id")?;
        require_text(&self.tenant_id, "access_request.tenant_id")?;
        require_text(&self.region, "access_request.region")?;
        if self.scopes.is_empty() {
            return Err(ContractError::EmptyCollection("access_request.scopes"));
        }
        Ok(())
    }
}

impl Validate for AccessDecision {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(&self.schema_version, ACCESS_DECISION_SCHEMA_VERSION, "access_decision.schema_version")?;
        require_text(&self.request_id, "access_decision.request_id")?;
        require_text(&self.tenant_id, "access_decision.tenant_id")?;
        if self.permitted && (!self.blockers.is_empty() || self.human_approval_required) {
            return Err(ContractError::Invariant("permitted access decisions cannot retain blockers or pending approval".to_owned()));
        }
        if !self.permitted && self.blockers.is_empty() {
            return Err(ContractError::Invariant("denied access decisions require a blocker".to_owned()));
        }
        Ok(())
    }
}
