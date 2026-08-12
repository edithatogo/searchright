//! Product-neutral contracts shared by Searchright and Sourceright.
//!
//! The `org.searchright.*` wire identifiers are retained for the 0.1 contract
//! generation. A future major contract generation may move neutral identifiers
//! without silently changing persisted data.

#![forbid(unsafe_code)]

mod audit;
mod provider;
mod query;
mod record;
mod schema;

pub use audit::{Actor, AuditEvent, AuditEventDraft};
pub use provider::{
    ExecutionPolicy, ProviderCapability, ProviderManifest, ProviderPage, ProviderSupportLevel,
    SearchRequest, SearchRun, SourceReceipt,
};
pub use query::{
    CompiledStrategy, DateLimit, FilterApplicability, FilterChecksum, FilterRights,
    FilterSourceCitation, FilterValidation, FilterValidationState, NamedFilterPack,
    NamedFilterRecord, NativeNormalisationState, NativeParseDiagnostic, NativeParseSeverity,
    NativeQueryLine, NativeQueryLineKind, NativeSearchStrategy, NativeSourceSpan, QueryExpr,
    RedistributionDecision, SearchDialect, SearchField, SearchLimit, SearchStrategy, SearchTerm,
    StrategyWarning, TranslationFidelity,
};
pub use record::{BibliographicRecord, RecordIdentifiers, RecordKind};
pub use schema::{
    RustOwnedSchema, RustSchemaParityScope, rust_owned_schemas, rust_schema_parity_scope,
};

/// Compatibility contract family retained for the 0.1 wire generation.
pub const CONTRACT_FAMILY: &str = "org.searchright";
/// Canonical provider-normalised bibliographic-record contract version.
pub const BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION: &str = "org.searchright.bibliographic-record.v1";
/// Canonical compiled-strategy contract version.
pub const COMPILED_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.compiled-strategy.v1";
/// Canonical provider-page contract version.
pub const PROVIDER_PAGE_SCHEMA_VERSION: &str = "org.searchright.provider-page.v1";
/// Canonical source-receipt contract version.
pub const SOURCE_RECEIPT_SCHEMA_VERSION: &str = "org.searchright.source-receipt.v1";
/// Canonical search-run contract version.
pub const SEARCH_RUN_SCHEMA_VERSION: &str = "org.searchright.search-run.v1";
/// Canonical search-strategy contract version.
pub const SEARCH_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.search-strategy.v1";
/// Canonical named-filter-pack contract version.
pub const NAMED_FILTER_PACK_SCHEMA_VERSION: &str = "org.searchright.named-filter-pack.v1";
/// Canonical native-search-strategy contract version.
pub const NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.native-search-strategy.v1";
/// Canonical audit-event contract version.
pub const AUDIT_EVENT_SCHEMA_VERSION: &str = "org.searchright.audit-event.v1";

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

/// Contract types implement semantic validation in addition to JSON Schema.
pub trait Validate {
    /// Validate semantic and cross-field invariants.
    fn validate(&self) -> Result<(), ContractError>;
}

/// Validate required non-empty text.
#[doc(hidden)]
pub fn require_text(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        Err(ContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

/// Validate a wire schema identifier.
#[doc(hidden)]
pub fn require_schema_version(
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
