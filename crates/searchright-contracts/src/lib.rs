//! Canonical Rust types for Searchright's public and persisted contracts.

#![forbid(unsafe_code)]

mod access;
mod amendment;
mod assurance;
mod benchmark;
mod delivery;
mod diagnostic;
mod discovery;
mod document;
mod governance;
mod integration;
mod interchange;
mod licensed;
mod living;
mod migration;
mod ops;
mod plan;
mod plugin;
mod policy;
mod prisma;
mod ranking;
mod screening;
mod standards;
mod study;
mod validation;

pub use evidence_search_contracts::{
    AUDIT_EVENT_SCHEMA_VERSION, Actor, AuditEvent, AuditEventDraft,
    BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, BibliographicRecord, COMPILED_STRATEGY_SCHEMA_VERSION,
    CONTRACT_FAMILY, CompiledStrategy, ContractError, DateLimit, ExecutionPolicy,
    NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION, NativeNormalisationState, NativeParseDiagnostic,
    NativeParseSeverity, NativeQueryLine, NativeQueryLineKind, NativeSearchStrategy,
    NativeSourceSpan, PROVIDER_PAGE_SCHEMA_VERSION, ProviderCapability, ProviderManifest,
    ProviderPage, ProviderSupportLevel, QueryExpr, RecordIdentifiers, RecordKind,
    SEARCH_RUN_SCHEMA_VERSION, SEARCH_STRATEGY_SCHEMA_VERSION, SOURCE_RECEIPT_SCHEMA_VERSION,
    SearchDialect, SearchField, SearchLimit, SearchRequest, SearchRun, SearchStrategy, SearchTerm,
    SourceReceipt, StrategyWarning, TranslationFidelity, Validate,
};
pub(crate) use evidence_search_contracts::{require_schema_version, require_text};

pub use access::{AccessDecision, AccessRequest, AccessScope, PrincipalKind, TenantPolicy};
pub use amendment::{AmendmentChange, AmendmentDecision, AmendmentKind, ProtocolAmendment};
pub use assurance::{LifecycleStage, LifecycleTransition, TransitionActorKind, WorkflowTrace};
pub use benchmark::{BenchmarkMetric, BenchmarkReport};
pub use delivery::{
    GitHubRepositorySettings, IntegrationReleaseTrain, ReleaseRehearsal, ReleaseRehearsalStatus,
    ReleaseTrainComponent, ReleaseTrainStage, RepositoryFeatures, RepositoryMergePolicy,
    RepositoryRuleset, RepositorySecurityControls, RepositoryVisibility, RulesetEnforcement,
};
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
    GitHubProjectField, GitHubProjectFieldDataType, GitHubProjectFieldValue, GitHubProjectManifest,
    GitHubProjectOwnerType, GitHubProjectSyncPolicy, GitHubProjectView, GitHubProjectViewLayout,
    IntegrationContractReference, IntegrationMode, IntegrationPassport,
    IntegrationVerificationGate,
};
pub use interchange::{InterchangeFormat, InterchangeReceipt};
pub use licensed::LicensedAdapterProfile;
pub use living::{LivingUpdateRun, RecordChange, RecordChangeKind, UpdateCursor, UpdateRunStatus};
pub use migration::{ParityDimensionResult, SourcerightParityReport};
pub use ops::{
    BackupKind, BackupManifest, ComponentHealth, HealthState, IncidentRecord, IncidentSeverity,
    TelemetryPolicy,
};
pub use plan::{
    EligibilityCriterion, EligibilitySet, FrameworkKind, InformationSource, InformationSourceKind,
    ProtocolRegistration, QuestionFramework, ResearchQuestion, ReviewGovernance, ReviewKind,
    ReviewPlan, ScreeningStage,
};
pub use plugin::{ComponentCapability, ProviderComponentManifest};
pub use policy::{
    ContentSafetyFinding, ExecutionEnvelope, FullTextHandling, NetworkCapability, SecretHandling,
    UntrustedContentPolicy,
};
pub use prisma::{ExclusionCount, PrismaFlow, PrismaSItem, PrismaSItemStatus, PrismaSLedger};
pub use ranking::{CalibrationCounts, RankingCalibration, RankingFeature, RankingScore};
pub use screening::{
    AgentAuthority, ConflictResolution, DecisionValue, ExclusionReason, ReviewerKind,
    ScreeningDecision, ScreeningPolicy, ScreeningRound,
};
pub use standards::{
    StandardAssessment, StandardFamily, StandardItem, StandardItemAssessment, StandardItemState,
    StandardPack,
};
pub use study::{
    EvidenceLink, EvidenceRelationship, Report, RetrievalAttempt, RetrievalStatus, Study,
    StudyGraph,
};
pub use validation::{
    FindingSeverity, PressElement, PressFinding, PressReview, SearchValidationReport, SeedRecord,
    TranslationLossAssessment,
};

/// Canonical screening-policy contract version.
pub const SCREENING_POLICY_SCHEMA_VERSION: &str = "org.searchright.screening-policy.v1";
/// Canonical deterministic agent-workflow contract version.
pub const AGENT_WORKFLOW_SCHEMA_VERSION: &str = "org.searchright.agent-workflow.v1";
/// Canonical review-plan contract version.
pub const REVIEW_PLAN_SCHEMA_VERSION: &str = "org.searchright.review-plan.v1";
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
pub const GITHUB_ISSUE_HIERARCHY_SCHEMA_VERSION: &str = "org.searchright.github-issue-hierarchy.v2";
/// Canonical GitHub Project v2 projection manifest contract version.
pub const GITHUB_PROJECT_SCHEMA_VERSION: &str = "org.searchright.github-project.v1";
/// Canonical consumer-driven integration contract-suite version.
pub const CONSUMER_CONTRACT_SUITE_SCHEMA_VERSION: &str =
    "org.searchright.consumer-contract-suite.v1";

/// Canonical accessible diagnostic contract version.
pub const DIAGNOSTIC_SCHEMA_VERSION: &str = "org.searchright.diagnostic.v1";
/// Canonical institutional data-governance policy contract version.
pub const INSTITUTIONAL_POLICY_SCHEMA_VERSION: &str = "org.searchright.institutional-policy.v1";
/// Canonical institutional data-handling request contract version.
pub const DATA_HANDLING_REQUEST_SCHEMA_VERSION: &str = "org.searchright.data-handling-request.v1";
/// Canonical institutional data-handling decision contract version.
pub const DATA_HANDLING_DECISION_SCHEMA_VERSION: &str = "org.searchright.data-handling-decision.v1";

/// Canonical component-health contract version.
pub const COMPONENT_HEALTH_SCHEMA_VERSION: &str = "org.searchright.component-health.v1";
/// Canonical telemetry-policy contract version.
pub const TELEMETRY_POLICY_SCHEMA_VERSION: &str = "org.searchright.telemetry-policy.v1";
/// Canonical backup-manifest contract version.
pub const BACKUP_MANIFEST_SCHEMA_VERSION: &str = "org.searchright.backup-manifest.v1";
/// Canonical incident-record contract version.
pub const INCIDENT_RECORD_SCHEMA_VERSION: &str = "org.searchright.incident-record.v1";
/// Canonical tenant-policy contract version.
pub const TENANT_POLICY_SCHEMA_VERSION: &str = "org.searchright.tenant-policy.v1";
/// Canonical access-request contract version.
pub const ACCESS_REQUEST_SCHEMA_VERSION: &str = "org.searchright.access-request.v1";
/// Canonical access-decision contract version.
pub const ACCESS_DECISION_SCHEMA_VERSION: &str = "org.searchright.access-decision.v1";
/// Canonical GitHub repository-settings manifest version.
pub const GITHUB_REPOSITORY_SETTINGS_SCHEMA_VERSION: &str =
    "org.searchright.github-repository-settings.v1";
/// Canonical cross-repository release-train version.
pub const INTEGRATION_RELEASE_TRAIN_SCHEMA_VERSION: &str =
    "org.searchright.integration-release-train.v1";
/// Canonical release-rehearsal version.
pub const RELEASE_REHEARSAL_SCHEMA_VERSION: &str = "org.searchright.release-rehearsal.v1";
