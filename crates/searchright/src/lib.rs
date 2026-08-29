//! Public Searchright facade.
//!
//! Consumers may depend on the smaller component crates when they need a narrow
//! surface. This facade re-exports the stable product-level API.

#![forbid(unsafe_code)]

mod authority;
mod engine;

pub use authority::{
    EffectAuthorityAttestation, EffectAuthorityError, EffectAuthorityRequest,
    EffectAuthorityVerifier, VerifiedEffectAuthority, verify_effect_authority,
};

pub use engine::{
    EngineError, HumanConfirmation, InterchangeExport, LocalPersistenceOutcome,
    LocalReviewOperation, PLAN_REVIEW_RESULT_SCHEMA_VERSION, PRESS_REVIEW_RESULT_SCHEMA_VERSION,
    PlanAssessment, PlanReviewOutcome, PressReviewOutcome, PrismaArtifact, PrismaOutput,
    SearchExecutionOperation, SearchrightEngine, StudyGraphAssessment,
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
pub use searchright_licensed as licensed;
pub use searchright_living as living;
pub use searchright_plugin_sdk as plugin_sdk;
pub use searchright_policy as policy;
pub use searchright_prisma as prisma;
pub use searchright_provenance as provenance;
pub use searchright_ranking as ranking;
pub use searchright_screening as screening;
pub use searchright_store as store;
pub use searchright_study as study;
pub use searchright_validation as validation;

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{InterchangeExport, PlanAssessment, PrismaArtifact, StudyGraphAssessment};

    fn assert_schema_has_payload_shape<T: schemars::JsonSchema>() {
        let value = match serde_json::to_value(schemars::schema_for!(T)) {
            Ok(value) => value,
            Err(error) => panic!("schema serialisation failed: {error}"),
        };
        let Some(object) = value.as_object() else {
            panic!("schema root must be a JSON object");
        };
        assert!(!object.is_empty(), "schema root must not be empty");
        assert!(
            has_payload_shape(&value),
            "schema must describe an object, union or array payload: {value}"
        );
    }

    fn has_payload_shape(value: &Value) -> bool {
        match value {
            Value::Object(object) => {
                object.contains_key("properties")
                    || object.contains_key("oneOf")
                    || object.contains_key("anyOf")
                    || object.contains_key("items")
                    || object.values().any(has_payload_shape)
            }
            Value::Array(values) => values.iter().any(has_payload_shape),
            _ => false,
        }
    }

    #[test]
    fn facade_success_payloads_have_json_schemas() {
        assert_schema_has_payload_shape::<PlanAssessment>();
        assert_schema_has_payload_shape::<searchright_contracts::CompiledStrategy>();
        assert_schema_has_payload_shape::<searchright_dedup::DedupResult>();
        assert_schema_has_payload_shape::<PrismaArtifact>();
        assert_schema_has_payload_shape::<evidence_search_core::AuditVerification>();
        assert_schema_has_payload_shape::<StudyGraphAssessment>();
        assert_schema_has_payload_shape::<searchright_validation::SearchValidationSummary>();
        assert_schema_has_payload_shape::<searchright_interchange::ImportResult>();
        assert_schema_has_payload_shape::<InterchangeExport>();
        assert_schema_has_payload_shape::<Vec<searchright_contracts::RecordChange>>();
        assert_schema_has_payload_shape::<searchright_provenance::ProvenanceBundle>();
        assert_schema_has_payload_shape::<Vec<searchright_contracts::RankingScore>>();
        assert_schema_has_payload_shape::<Vec<searchright_contracts::ContentSafetyFinding>>();
        assert_schema_has_payload_shape::<searchright_contracts::DataHandlingDecision>();
        assert_schema_has_payload_shape::<searchright_assurance::AssuranceReport>();
        assert_schema_has_payload_shape::<Vec<searchright_discovery::DiscoveredCandidate>>();
        assert_schema_has_payload_shape::<searchright_licensed::LicensedRequestPlan>();
        assert_schema_has_payload_shape::<Vec<searchright_contracts::ProviderManifest>>();
        assert_schema_has_payload_shape::<searchright_agent::AgentWorkflow>();
    }
}
