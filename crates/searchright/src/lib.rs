//! Public Searchright facade.
//!
//! Consumers may depend on the smaller component crates when they need a narrow
//! surface. This facade re-exports the stable product-level API.

#![forbid(unsafe_code)]

pub use evidence_search_core as core;
pub use searchright_agent as agent;
pub use searchright_connectors as connectors;
pub use searchright_contracts as contracts;
pub use searchright_dedup as dedup;
pub use searchright_prisma as prisma;
pub use searchright_screening as screening;
pub use searchright_store as store;
