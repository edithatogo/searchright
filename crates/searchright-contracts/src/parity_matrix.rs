//! Declared execution evidence, distinct from the frozen v1 advisory summary.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Expected-cell catalogue contract identifier.
pub const PARITY_CATALOGUE_SCHEMA_V2: &str = "org.searchright.sourceright-parity-catalogue.v2";
/// Observed matrix contract identifier.
pub const PARITY_MATRIX_SCHEMA_V2: &str = "org.searchright.sourceright-parity-matrix.v2";
/// Exact canonical JSON comparison; no semantic coercion or normalisation.
pub const PARITY_COMPARATOR_V2: &str = "canonical-json-blake3.v1";

/// Whether supplied observations are synthetic examples or declared executions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParityEvidenceKind {
    /// Test/example data, never an execution receipt.
    Synthetic,
    /// Caller-declared execution, not authenticated by matrix validation.
    DeclaredExecution,
}

/// Direction of a comparison, also bound into each observation digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParityRunSide {
    /// Old implementation.
    Legacy,
    /// Proposed shared implementation.
    Shared,
}

/// Exact identity of one required comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParityCellKey {
    /// Registry provider identifier.
    pub provider_id: String,
    /// SHA-256 of fixture bytes, declared by the caller.
    pub fixture_sha256: String,
    /// Existing migration scenario identifier.
    pub case_id: String,
    /// Dimension assigned to this scenario.
    pub dimension: String,
}

/// Revision and execution configuration identity, not proof those bytes ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParityRunBinding {
    /// Explicit side prevents output transplantation even for identical pins.
    pub side: ParityRunSide,
    /// Stable repository identity, never fetched by validation.
    pub repository: String,
    /// Full lower-case forty-digit Git commit identifier.
    pub revision: String,
    /// SHA-256 of the execution configuration.
    pub config_sha256: String,
    /// SHA-256 of the harness artifact.
    pub harness_sha256: String,
}

/// Caller-declared fixture origin and rights reference; not legal clearance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixtureProvenance {
    /// Local provenance record reference; not dereferenced.
    pub source_id: String,
    /// Declared rights basis or review reference.
    pub rights_basis: String,
}

/// Required comparison selected independently of the execution report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectedParityCell {
    /// Exact provider/fixture/case/dimension tuple.
    pub key: ParityCellKey,
    /// Required fixture provenance declaration.
    pub provenance: FixtureProvenance,
    /// Supported versioned comparator identifier.
    pub comparator_id: String,
}

/// Independently selected expected scope, pinned externally by the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParityExpectedCatalogueV2 {
    /// Must equal `PARITY_CATALOGUE_SCHEMA_V2`.
    pub schema_version: String,
    /// Synthetic and declared-execution scopes cannot be interchanged.
    pub evidence_kind: ParityEvidenceKind,
    /// Required legacy execution identity.
    pub legacy: ParityRunBinding,
    /// Required shared execution identity.
    pub shared: ParityRunBinding,
    /// Explicit required cells; no inferred Cartesian product.
    pub cells: Vec<ExpectedParityCell>,
}

/// Provider outcomes must remain distinguishable from failed or absent execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParityOutcome {
    /// Provider execution produced a successful result.
    Success,
    /// Provider execution produced an error, potentially the expected test result.
    ProviderError,
    /// Harness failed to produce a provider observation.
    HarnessFailure,
    /// Provider execution did not run.
    Skipped,
}

/// An observation plus a context-bound digest; artifacts remain caller supplied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParityExecutionObservation {
    /// Execution classification, included in the digest.
    pub status: ParityOutcome,
    /// Already parsed, minimised JSON; arbitrary raw responses are inappropriate.
    pub value: Value,
    /// Stable execution reference, never executed or fetched by validation.
    pub execution_id: String,
    /// Declared SHA-256 of supporting evidence bytes.
    pub evidence_sha256: String,
    /// Context-bound canonical BLAKE3 digest recomputed by the validator.
    pub digest: String,
}

/// Observations for one exact expected comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservedParityCell {
    /// Repeated expected tuple/provenance/comparator, matched exactly.
    pub expected: ExpectedParityCell,
    /// Legacy-side outcome.
    pub legacy: ParityExecutionObservation,
    /// Shared-side outcome.
    pub shared: ParityExecutionObservation,
    /// Unauthenticated advisory references; cannot waive a difference.
    pub decision_references: Vec<String>,
}

/// Caller-declared execution matrix; contains no readiness or cutover flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParityExecutionMatrixV2 {
    /// Must equal `PARITY_MATRIX_SCHEMA_V2`.
    pub schema_version: String,
    /// Nature of the supplied evidence.
    pub evidence_kind: ParityEvidenceKind,
    /// Actual declared legacy identity, matched to the trusted catalogue.
    pub legacy: ParityRunBinding,
    /// Actual declared shared identity, matched to the trusted catalogue.
    pub shared: ParityRunBinding,
    /// Exactly one result per externally required cell.
    pub cells: Vec<ObservedParityCell>,
}
