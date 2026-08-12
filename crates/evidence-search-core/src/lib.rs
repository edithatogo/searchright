//! Shared evidence-search kernel used by Searchright and intended for Sourceright.

#![forbid(unsafe_code)]

mod audit;
mod compiler;
mod native;
mod provider;

pub use audit::{AuditError, AuditLedger, AuditVerification, verify_event_integrity};
pub use compiler::{COMPILER_VERSION, CompileError, QueryCompiler};
pub use native::{NATIVE_PARSER_VERSION, parse_native_strategy};
pub use provider::{
    CachedProviderPage, ExecutionResult, MemoryPageCache, PageCache, ProviderError, ProviderMode,
    ProviderRegistry, SearchProvider, canonical_record_digest,
    validate_resolved_endpoint_addresses,
};
