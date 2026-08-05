//! Canonical Rust types for Searchright's public and persisted contracts.

#![forbid(unsafe_code)]

mod audit;
mod plan;
mod prisma;
mod provider;
mod query;
mod record;
mod screening;

pub use audit::{Actor, AuditEvent, AuditEventDraft};
pub use plan::{
    EligibilityCriterion, EligibilitySet, FrameworkKind, InformationSource, InformationSourceKind,
    ProtocolRegistration, QuestionFramework, ResearchQuestion, ReviewGovernance, ReviewKind,
    ReviewPlan, ScreeningStage,
};
pub use prisma::{ExclusionCount, PrismaFlow, PrismaSItem, PrismaSItemStatus, PrismaSLedger};
pub use provider::{
    ExecutionPolicy, ProviderCapability, ProviderManifest, ProviderPage, ProviderSupportLevel,
    SearchRequest, SearchRun, SourceReceipt,
};
pub use query::{
    CompiledStrategy, DateLimit, QueryExpr, SearchDialect, SearchField, SearchLimit, SearchStrategy,
    SearchTerm, StrategyWarning,
};
pub use record::{BibliographicRecord, RecordIdentifiers, RecordKind};
pub use screening::{
    AgentAuthority, ConflictResolution, DecisionValue, ExclusionReason, ReviewerKind,
    ScreeningDecision, ScreeningPolicy, ScreeningRound,
};

/// Current repository-wide contract family identifier.
pub const CONTRACT_FAMILY: &str = "org.searchright";

/// Canonical review-plan contract version.
pub const REVIEW_PLAN_SCHEMA_VERSION: &str = "org.searchright.review-plan.v1";
/// Canonical search-strategy contract version.
pub const SEARCH_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.search-strategy.v1";
/// Canonical audit-event contract version.
pub const AUDIT_EVENT_SCHEMA_VERSION: &str = "org.searchright.audit-event.v1";
/// Canonical PRISMA-flow contract version.
pub const PRISMA_FLOW_SCHEMA_VERSION: &str = "org.searchright.prisma-flow.v1";

/// Error returned by contract validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// A required field was empty.
    #[error("required field `{0}` is empty")]
    EmptyField(&'static str),
    /// A collection that must contain at least one value was empty.
    #[error("collection `{0}` must not be empty")]
    EmptyCollection(&'static str),
    /// A value violated a cross-field invariant.
    #[error("contract invariant failed: {0}")]
    Invariant(String),
}

/// Contract types implement lightweight semantic validation in addition to JSON Schema.
pub trait Validate {
    /// Validate semantic and cross-field invariants.
    fn validate(&self) -> Result<(), ContractError>;
}

pub(crate) fn require_text(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        Err(ContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

pub(crate) fn require_schema_version(
    value: &str,
    expected: &'static str,
    field: &'static str,
) -> Result<(), ContractError> {
    require_text(value, field)?;
    if value == expected {
        Ok(())
    } else {
        Err(ContractError::Invariant(format!(
            "`{field}` must be `{expected}`, found `{value}`"
        )))
    }
}
