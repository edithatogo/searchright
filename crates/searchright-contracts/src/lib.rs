//! Canonical Rust types for Searchright's public and persisted contracts.

#![forbid(unsafe_code)]

mod amendment;
mod audit;
mod assurance;
mod benchmark;
mod diagnostic;
mod discovery;
mod document;
mod governance;
mod integration;
mod interchange;
mod living;
mod licensed;
mod migration;
mod plan;
mod plugin;
mod policy;
mod prisma;
mod provider;
mod query;
mod ranking;
mod record;
mod screening;
mod standards;
mod study;
mod validation;

pub use amendment::{AmendmentChange, AmendmentDecision, AmendmentKind, ProtocolAmendment};
pub use audit::{Actor, AuditEvent, AuditEventDraft};
pub use assurance::{LifecycleStage, LifecycleTransition, TransitionActorKind, WorkflowTrace};
pub use benchmark::{BenchmarkMetric, BenchmarkReport};
pub use diagnostic::{Diagnostic, DiagnosticLocale, DiagnosticSeverity};
pub use discovery::{DiscoveryEdge, DiscoveryMethod, DiscoveryRun};
pub use document::{
    CitationCalloutEvidence, DocumentEvidence, DocumentExtractionProvenance, DocumentSpan,
    ExtractedFieldEvidence, ExtractedReferenceEvidence, ExtractionDiagnostic,
};
pub use governance::{
    DataClassification, DataHandlingDecision, DataHandlingRequest, DataOperationKind,
    DeploymentMode, InstitutionalPolicy,
};
pub use integration::{
    ConsumerContractInteraction, ConsumerContractStatus, ConsumerContractSuite,
    DependencyDirection, GitHubIssueHierarchy, GitHubIssueKind, GitHubIssueNode,
    IntegrationContractReference, IntegrationMode, IntegrationPassport,
    IntegrationVerificationGate,
};
pub use interchange::{InterchangeFormat, InterchangeReceipt};
pub use living::{LivingUpdateRun, RecordChange, RecordChangeKind, UpdateCursor, UpdateRunStatus};
pub use licensed::LicensedAdapterProfile;
pub use migration::{ParityDimensionResult, SourcerightParityReport};
pub use plugin::{ComponentCapability, ProviderComponentManifest};
pub use plan::{
    EligibilityCriterion, EligibilitySet, FrameworkKind, InformationSource, InformationSourceKind,
    ProtocolRegistration, QuestionFramework, ResearchQuestion, ReviewGovernance, ReviewKind,
    ReviewPlan, ScreeningStage,
};
pub use policy::{
    ContentSafetyFinding, ExecutionEnvelope, FullTextHandling, NetworkCapability, SecretHandling,
    UntrustedContentPolicy,
};
pub use prisma::{ExclusionCount, PrismaFlow, PrismaSItem, PrismaSItemStatus, PrismaSLedger};
pub use provider::{
    ExecutionPolicy, ProviderCapability, ProviderManifest, ProviderPage, ProviderSupportLevel,
    SearchRequest, SearchRun, SourceReceipt,
};
pub use query::{
    CompiledStrategy, DateLimit, QueryExpr, SearchDialect, SearchField, SearchLimit, SearchStrategy,
    SearchTerm, StrategyWarning, TranslationFidelity,
};
pub use ranking::{CalibrationCounts, RankingCalibration, RankingFeature, RankingScore};
pub use record::{BibliographicRecord, RecordIdentifiers, RecordKind};
pub use screening::{
    AgentAuthority, ConflictResolution, DecisionValue, ExclusionReason, ReviewerKind,
    ScreeningDecision, ScreeningPolicy, ScreeningRound,
};
pub use standards::{
    StandardAssessment, StandardFamily, StandardItem, StandardItemAssessment, StandardItemState,
    StandardPack,
};
pub use study::{
    EvidenceLink, EvidenceRelationship, Report, RetrievalAttempt, RetrievalStatus, Study, StudyGraph,
};
pub use validation::{
    FindingSeverity, PressElement, PressFinding, PressReview, SearchValidationReport, SeedRecord,
    TranslationLossAssessment,
};

/// Current repository-wide contract family identifier.
pub const CONTRACT_FAMILY: &str = "org.searchright";

/// Canonical provider-normalised bibliographic-record contract version.
pub const BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION: &str =
    "org.searchright.bibliographic-record.v1";
/// Canonical compiled-strategy contract version.
pub const COMPILED_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.compiled-strategy.v1";
/// Canonical provider-page contract version.
pub const PROVIDER_PAGE_SCHEMA_VERSION: &str = "org.searchright.provider-page.v1";
/// Canonical source-receipt contract version.
pub const SOURCE_RECEIPT_SCHEMA_VERSION: &str = "org.searchright.source-receipt.v1";
/// Canonical search-run contract version.
pub const SEARCH_RUN_SCHEMA_VERSION: &str = "org.searchright.search-run.v1";
/// Canonical screening-policy contract version.
pub const SCREENING_POLICY_SCHEMA_VERSION: &str = "org.searchright.screening-policy.v1";
/// Canonical deterministic agent-workflow contract version.
pub const AGENT_WORKFLOW_SCHEMA_VERSION: &str = "org.searchright.agent-workflow.v1";
/// Canonical review-plan contract version.
pub const REVIEW_PLAN_SCHEMA_VERSION: &str = "org.searchright.review-plan.v1";
/// Canonical search-strategy contract version.
pub const SEARCH_STRATEGY_SCHEMA_VERSION: &str = "org.searchright.search-strategy.v1";
/// Canonical audit-event contract version.
pub const AUDIT_EVENT_SCHEMA_VERSION: &str = "org.searchright.audit-event.v1";
/// Canonical PRISMA-flow contract version.
pub const PRISMA_FLOW_SCHEMA_VERSION: &str = "org.searchright.prisma-flow.v1";
/// Canonical record-report-study graph contract version.
pub const STUDY_GRAPH_SCHEMA_VERSION: &str = "org.searchright.study-graph.v1";
/// Canonical protocol-amendment contract version.
pub const PROTOCOL_AMENDMENT_SCHEMA_VERSION: &str = "org.searchright.protocol-amendment.v1";
/// Canonical living-update contract version.
pub const LIVING_UPDATE_SCHEMA_VERSION: &str = "org.searchright.living-update.v1";
/// Canonical search-validation contract version.
pub const SEARCH_VALIDATION_SCHEMA_VERSION: &str = "org.searchright.search-validation.v1";
/// Canonical standards-pack contract version.
pub const STANDARD_PACK_SCHEMA_VERSION: &str = "org.searchright.standard-pack.v1";
/// Canonical standards-assessment contract version.
pub const STANDARD_ASSESSMENT_SCHEMA_VERSION: &str = "org.searchright.standard-assessment.v1";
/// Canonical execution-envelope contract version.
pub const EXECUTION_ENVELOPE_SCHEMA_VERSION: &str = "org.searchright.execution-envelope.v1";
/// Canonical ranking-calibration contract version.
pub const RANKING_CALIBRATION_SCHEMA_VERSION: &str = "org.searchright.ranking-calibration.v1";
/// Canonical interchange-receipt contract version.
pub const INTERCHANGE_RECEIPT_SCHEMA_VERSION: &str = "org.searchright.interchange-receipt.v1";
/// Canonical supplementary-discovery contract version.
pub const DISCOVERY_RUN_SCHEMA_VERSION: &str = "org.searchright.discovery-run.v1";
/// Canonical workflow-assurance trace contract version.
pub const WORKFLOW_TRACE_SCHEMA_VERSION: &str = "org.searchright.workflow-trace.v1";
/// Canonical WASI provider-component manifest contract version.
pub const PROVIDER_COMPONENT_SCHEMA_VERSION: &str = "org.searchright.provider-component.v1";
/// Canonical reproducible benchmark report contract version.
pub const BENCHMARK_REPORT_SCHEMA_VERSION: &str = "org.searchright.benchmark-report.v1";
/// Canonical bring-your-own-access adapter profile contract version.
pub const LICENSED_ADAPTER_SCHEMA_VERSION: &str = "org.searchright.licensed-adapter.v1";
/// Canonical Sourceright/shared-core parity report contract version.
pub const SOURCERIGHT_PARITY_REPORT_SCHEMA_VERSION: &str =
    "org.searchright.sourceright-parity-report.v1";

/// Canonical neutral scholarly document-evidence contract version.
pub const DOCUMENT_EVIDENCE_SCHEMA_VERSION: &str = "org.searchright.document-evidence.v1";
/// Canonical cross-repository integration-passport contract version.
pub const INTEGRATION_PASSPORT_SCHEMA_VERSION: &str = "org.searchright.integration-passport.v1";
/// Canonical generated GitHub issue-hierarchy contract version.
pub const GITHUB_ISSUE_HIERARCHY_SCHEMA_VERSION: &str =
    "org.searchright.github-issue-hierarchy.v1";
/// Canonical consumer-driven integration contract-suite version.
pub const CONSUMER_CONTRACT_SUITE_SCHEMA_VERSION: &str =
    "org.searchright.consumer-contract-suite.v1";

/// Canonical accessible diagnostic contract version.
pub const DIAGNOSTIC_SCHEMA_VERSION: &str = "org.searchright.diagnostic.v1";
/// Canonical institutional data-governance policy contract version.
pub const INSTITUTIONAL_POLICY_SCHEMA_VERSION: &str =
    "org.searchright.institutional-policy.v1";
/// Canonical institutional data-handling request contract version.
pub const DATA_HANDLING_REQUEST_SCHEMA_VERSION: &str =
    "org.searchright.data-handling-request.v1";
/// Canonical institutional data-handling decision contract version.
pub const DATA_HANDLING_DECISION_SCHEMA_VERSION: &str =
    "org.searchright.data-handling-decision.v1";

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
