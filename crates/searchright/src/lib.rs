//! Public Searchright facade.
//!
//! Consumers may depend on the smaller component crates when they need a narrow
//! surface. This facade re-exports the stable product-level API.

#![forbid(unsafe_code)]

mod engine;

pub use engine::{
    EngineError, InterchangeExport, PlanAssessment, PrismaArtifact, PrismaOutput, SearchrightEngine,
    StudyGraphAssessment,
};

pub use evidence_search_core as core;
pub use searchright_agent as agent;
pub use searchright_assurance as assurance;
pub use searchright_bench as bench;
pub use searchright_connectors as connectors;
pub use searchright_contracts as contracts;
pub use searchright_dedup as dedup;
pub use searchright_diagnostics as diagnostics;
pub use searchright_discovery as discovery;
pub use searchright_governance as governance;
pub use searchright_interchange as interchange;
pub use searchright_living as living;
pub use searchright_licensed as licensed;
pub use searchright_policy as policy;
pub use searchright_plugin_sdk as plugin_sdk;
pub use searchright_prisma as prisma;
pub use searchright_provenance as provenance;
pub use searchright_ranking as ranking;
pub use searchright_screening as screening;
pub use searchright_store as store;
pub use searchright_study as study;
pub use searchright_validation as validation;
