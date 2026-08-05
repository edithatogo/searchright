//! Shared evidence-search kernel used by Searchright and intended for Sourceright.

#![forbid(unsafe_code)]

mod audit;
mod compiler;
mod provider;

pub use audit::{AuditError, AuditLedger, AuditVerification};
pub use compiler::{COMPILER_VERSION, QueryCompiler};
pub use provider::{
    ExecutionResult, ProviderError, ProviderMode, ProviderRegistry, SearchProvider,
};
