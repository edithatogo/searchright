//! Shared evidence-search kernel used by Searchright and intended for Sourceright.

#![forbid(unsafe_code)]

mod audit;
mod compiler;
mod native;
mod provider;

pub use audit::{AuditError, AuditLedger, AuditVerification};
pub use compiler::{COMPILER_VERSION, CompileError, QueryCompiler};
pub use native::{NATIVE_PARSER_VERSION, parse_native_strategy};
pub use provider::{
    CachedProviderPage, ExecutionResult, MemoryPageCache, PageCache, ProviderError, ProviderMode,
    ProviderRegistry, SearchProvider,
};
