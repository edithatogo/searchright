//! Product-level application service shared by the CLI, MCP server and Rust consumers.

use std::collections::BTreeMap;

use evidence_search_core::{AuditLedger, AuditVerification, QueryCompiler};
use schemars::JsonSchema;
use searchright_agent::{AgentWorkflow, ReadinessFinding, assess_plan_readiness};
use searchright_contracts::{
    AuditEvent, BenchmarkReport, BibliographicRecord, ContentSafetyFinding, DataHandlingDecision,
    DataHandlingRequest, Diagnostic, DiscoveryRun, DocumentEvidence, ExecutionEnvelope,
    InstitutionalPolicy, InterchangeFormat, LicensedAdapterProfile, LivingUpdateRun, PrismaFlow,
    PrismaSLedger, ProtocolAmendment, ProviderComponentManifest, ProviderManifest,
    RankingCalibration, RankingScore, RecordChange, RetrievalStatus, ReviewPlan, SearchDialect,
    SearchStrategy, SearchValidationReport, SourceReceipt, StandardAssessment, StandardPack,
    StudyGraph, UntrustedContentPolicy, Validate, WorkflowTrace,
};
use searchright_dedup::{DedupConfig, DedupResult, Deduplicator};
use searchright_interchange::ImportResult;
use searchright_provenance::ProvenanceBundle;
use searchright_validation::SearchValidationSummary;
use serde::{Deserialize, Serialize};

/// Stateless product facade. All durable or network effects remain explicit in component APIs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchrightEngine;

/// Result of validating and methodologically assessing a review plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlanAssessment {
    /// Stable review identifier.
    pub review_id: String,
    /// Deterministic readiness findings.
    pub findings: Vec<ReadinessFinding>,
    /// Whether no blocking finding remains.
    pub ready_for_strategy_design: bool,
}

/// Serialisable PRISMA rendering selected by a caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "artifact_type", content = "artifact", rename_all = "snake_case")]
pub enum PrismaArtifact {
    /// Validated canonical flow.
    Flow(PrismaFlow),
    /// Mermaid source for a flow diagram.
    Mermaid(String),
    /// PRISMA-S reporting ledgers.
    PrismaSLedger(Vec<PrismaSLedger>),
}

/// Supported PRISMA rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrismaOutput {
    /// Return the validated flow contract.
    Json,
    /// Return Mermaid source.
    Mermaid,
    /// Return the PRISMA-S ledger.
    PrismaSLedger,
}

/// Compact study-graph assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StudyGraphAssessment {
    /// Number of reports.
    pub reports: usize,
    /// Number of underlying studies.
    pub studies: usize,
    /// Reports that have not been associated with an underlying study.
    pub unlinked_report_ids: Vec<String>,
    /// Current full-text retrieval status by report.
    pub retrieval_statuses: BTreeMap<String, RetrievalStatus>,
    /// Number of reports for each study.
    pub reports_per_study: BTreeMap<String, usize>,
}

/// Complete record-interchange operation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InterchangeExport {
    /// Serialised output.
    pub document: String,
    /// Deterministic conversion receipt.
    pub receipt: searchright_contracts::InterchangeReceipt,
}

impl SearchrightEngine {
    /// Validate a review plan and calculate conservative readiness findings.
    pub fn validate_plan(plan: &ReviewPlan) -> Result<PlanAssessment, EngineError> {
        plan.validate()?;
        let findings = assess_plan_readiness(plan);
        let ready_for_strategy_design = !findings.iter().any(|finding| finding.blocking);
        Ok(PlanAssessment {
            review_id: plan.review_id.clone(),
            findings,
            ready_for_strategy_design,
        })
    }

    /// Validate a source-specific strategy without compiling or executing it.
    pub fn validate_strategy(strategy: &SearchStrategy) -> Result<(), EngineError> {
        Ok(strategy.validate()?)
    }

    /// Validate neutral, non-canonical document extraction evidence.
    pub fn validate_document_evidence(evidence: &DocumentEvidence) -> Result<(), EngineError> {
        Ok(evidence.validate()?)
    }

    /// Compile a portable strategy into one source-specific dialect.
    pub fn compile_strategy(
        strategy: &SearchStrategy,
        dialect: SearchDialect,
    ) -> Result<searchright_contracts::CompiledStrategy, EngineError> {
        Ok(QueryCompiler::compile(strategy, dialect)?)
    }

    /// Generate reviewable duplicate clusters without deleting source records.
    pub fn deduplicate(
        records: &[BibliographicRecord],
        config: DedupConfig,
    ) -> Result<DedupResult, EngineError> {
        Ok(Deduplicator::new(config)?.cluster(records)?)
    }

    /// Validate and render one PRISMA artefact.
    pub fn prisma(flow: &PrismaFlow, output: PrismaOutput) -> Result<PrismaArtifact, EngineError> {
        searchright_prisma::validate_flow(flow)?;
        match output {
            PrismaOutput::Json => Ok(PrismaArtifact::Flow(flow.clone())),
            PrismaOutput::Mermaid => Ok(PrismaArtifact::Mermaid(
                searchright_prisma::render_mermaid(flow)?,
            )),
            PrismaOutput::PrismaSLedger => Ok(PrismaArtifact::PrismaSLedger(
                searchright_prisma::build_prisma_s_ledger(flow)?,
            )),
        }
    }

    /// Verify an append-only audit chain.
    pub fn verify_audit(events: Vec<AuditEvent>) -> Result<AuditVerification, EngineError> {
        Ok(AuditLedger::from_events(events).verify()?)
    }

    /// Validate the record-report-study graph and derive review summaries.
    pub fn assess_study_graph(graph: &StudyGraph) -> Result<StudyGraphAssessment, EngineError> {
        searchright_study::validate_graph(graph)?;
        Ok(StudyGraphAssessment {
            reports: graph.reports.len(),
            studies: graph.studies.len(),
            unlinked_report_ids: searchright_study::unlinked_reports(graph),
            retrieval_statuses: searchright_study::retrieval_statuses(graph),
            reports_per_study: searchright_study::reports_per_study(graph),
        })
    }

    /// Evaluate PRESS, seed-set recall and translation-loss gates.
    pub fn assess_search_validation(
        report: &SearchValidationReport,
    ) -> Result<SearchValidationSummary, EngineError> {
        Ok(searchright_validation::assess(report)?)
    }

    /// Import records from a supported bibliographic interchange format.
    pub fn import_records(
        input: &str,
        format: InterchangeFormat,
        source_receipt_id: &str,
    ) -> Result<ImportResult, EngineError> {
        Ok(searchright_interchange::import_records(
            input,
            format,
            source_receipt_id,
        )?)
    }

    /// Export records and emit a deterministic conversion receipt.
    pub fn export_records(
        review_id: &str,
        records: &[BibliographicRecord],
        input_format: InterchangeFormat,
        output_format: InterchangeFormat,
    ) -> Result<InterchangeExport, EngineError> {
        let input = serde_json::to_vec(records)?;
        let document = searchright_interchange::export_records(records, output_format.clone())?;
        let receipt = searchright_interchange::conversion_receipt(
            review_id,
            input_format,
            output_format,
            &input,
            document.as_bytes(),
            records.len(),
            records.len(),
            Vec::new(),
            true,
        )?;
        Ok(InterchangeExport { document, receipt })
    }

    /// Compare parent and current result sets for a living review update.
    pub fn diff_living_records(
        previous: &[BibliographicRecord],
        current: &[BibliographicRecord],
    ) -> Result<Vec<RecordChange>, EngineError> {
        Ok(searchright_living::diff_records(previous, current)?)
    }

    /// Validate immutable living-review lineage.
    pub fn validate_living_lineage(runs: &[LivingUpdateRun]) -> Result<(), EngineError> {
        Ok(searchright_living::validate_lineage(runs)?)
    }

    /// Build RO-Crate 1.3 and W3C PROV-compatible exports.
    pub fn provenance(
        plan: &ReviewPlan,
        receipts: &[SourceReceipt],
        events: &[AuditEvent],
    ) -> Result<ProvenanceBundle, EngineError> {
        Ok(searchright_provenance::build_bundle(
            plan, receipts, events,
        )?)
    }

    /// Rank records transparently for prioritisation only.
    pub fn rank_records(
        records: &[BibliographicRecord],
        query_terms: &[String],
    ) -> Result<Vec<RankingScore>, EngineError> {
        Ok(searchright_ranking::LexicalRanker::default().score(records, query_terms)?)
    }

    /// Validate a ranking calibration report and its no-auto-exclusion contract.
    pub fn validate_ranking_calibration(
        calibration: &RankingCalibration,
    ) -> Result<(), EngineError> {
        Ok(searchright_ranking::validate_calibration(calibration)?)
    }

    /// Inspect untrusted provider/full-text content without executing it as instructions.
    #[must_use]
    pub fn inspect_content(
        subject_id: &str,
        text: &str,
        policy: UntrustedContentPolicy,
    ) -> Vec<ContentSafetyFinding> {
        searchright_policy::inspect_untrusted_text(subject_id, text, policy)
    }

    /// Render validated diagnostics in an accessible, stable representation.
    pub fn render_diagnostics(
        diagnostics: &[Diagnostic],
        output: searchright_diagnostics::DiagnosticOutput,
    ) -> Result<String, EngineError> {
        Ok(searchright_diagnostics::render(diagnostics, output)?)
    }

    /// Evaluate a data-handling request against an institutional governance policy.
    pub fn evaluate_governance(
        policy: &InstitutionalPolicy,
        request: &DataHandlingRequest,
    ) -> Result<DataHandlingDecision, EngineError> {
        Ok(searchright_governance::evaluate(policy, request)?)
    }

    /// Authorise an endpoint against a validated execution envelope.
    pub fn authorise_endpoint(
        envelope: &ExecutionEnvelope,
        endpoint: &str,
    ) -> Result<(), EngineError> {
        let endpoint = url::Url::parse(endpoint)?;
        Ok(searchright_policy::authorise_endpoint(envelope, &endpoint)?)
    }

    /// Validate one protocol amendment.
    pub fn validate_amendment(amendment: &ProtocolAmendment) -> Result<(), EngineError> {
        Ok(amendment.validate()?)
    }

    /// Validate a standards-pack definition.
    pub fn validate_standard_pack(pack: &StandardPack) -> Result<(), EngineError> {
        Ok(pack.validate()?)
    }

    /// Validate a standards assessment.
    pub fn validate_standard_assessment(
        assessment: &StandardAssessment,
    ) -> Result<(), EngineError> {
        Ok(assessment.validate()?)
    }

    /// Validate one supplementary-discovery run.
    pub fn validate_discovery_run(run: &DiscoveryRun) -> Result<(), EngineError> {
        Ok(run.validate()?)
    }

    /// Verify a complete lifecycle trace against the finite assurance model.
    pub fn verify_workflow_trace(
        trace: &WorkflowTrace,
    ) -> Result<searchright_assurance::AssuranceReport, EngineError> {
        Ok(searchright_assurance::verify_trace(trace)?)
    }

    /// Resolve bounded supplementary-discovery candidates for human release.
    pub fn discovery_candidates(
        run: &DiscoveryRun,
    ) -> Result<Vec<searchright_discovery::DiscoveredCandidate>, EngineError> {
        Ok(searchright_discovery::bounded_candidates(run)?)
    }

    /// Verify a provider-component manifest and exact WASI component bytes.
    pub fn verify_provider_component(
        manifest: &ProviderComponentManifest,
        bytes: &[u8],
    ) -> Result<(), EngineError> {
        Ok(searchright_plugin_sdk::verify_component(manifest, bytes)?)
    }

    /// Build a redacted bring-your-own-access request plan without exposing credentials.
    pub fn plan_licensed_request(
        profile: &LicensedAdapterProfile,
        strategy: &searchright_contracts::CompiledStrategy,
        endpoint: &str,
    ) -> Result<searchright_licensed::LicensedRequestPlan, EngineError> {
        Ok(searchright_licensed::plan_request(
            profile, strategy, endpoint,
        )?)
    }

    /// Validate a benchmark report and its claim boundary.
    pub fn validate_benchmark_report(report: &BenchmarkReport) -> Result<(), EngineError> {
        Ok(report.validate()?)
    }

    /// List the deterministic, no-network provider fixtures available by default.
    pub fn default_provider_manifests() -> Result<Vec<ProviderManifest>, EngineError> {
        let mut registry = evidence_search_core::ProviderRegistry::new();
        searchright_connectors::register_mvp_fixtures(&mut registry)
            .map_err(|error| EngineError::Provider(error.to_string()))?;
        Ok(registry.manifests())
    }

    /// Return the default conservative agent workflow.
    #[must_use]
    pub fn workflow() -> AgentWorkflow {
        AgentWorkflow::systematic_search()
    }
}

/// Product-level operation error.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Canonical contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Query compilation failed.
    #[error(transparent)]
    Compile(#[from] evidence_search_core::CompileError),
    /// Audit verification failed.
    #[error(transparent)]
    Audit(#[from] evidence_search_core::AuditError),
    /// Deduplication failed.
    #[error(transparent)]
    Dedup(#[from] searchright_dedup::DedupError),
    /// PRISMA generation failed.
    #[error(transparent)]
    Prisma(#[from] searchright_prisma::PrismaError),
    /// Study-graph processing failed.
    #[error(transparent)]
    Study(#[from] searchright_study::StudyGraphError),
    /// Search validation failed.
    #[error(transparent)]
    Validation(#[from] searchright_validation::ValidationError),
    /// Interchange processing failed.
    #[error(transparent)]
    Interchange(#[from] searchright_interchange::InterchangeError),
    /// Living-review processing failed.
    #[error(transparent)]
    Living(#[from] searchright_living::LivingError),
    /// Provenance export failed.
    #[error(transparent)]
    Provenance(#[from] searchright_provenance::ProvenanceError),
    /// Ranking failed.
    #[error(transparent)]
    Ranking(#[from] searchright_ranking::RankingError),
    /// Execution policy denied an operation.
    #[error(transparent)]
    Policy(#[from] searchright_policy::PolicyError),
    /// Workflow assurance failed.
    #[error(transparent)]
    Assurance(#[from] searchright_assurance::AssuranceError),
    /// Supplementary discovery failed.
    #[error(transparent)]
    Discovery(#[from] searchright_discovery::DiscoveryError),
    /// Diagnostic rendering failed.
    #[error(transparent)]
    Diagnostic(#[from] searchright_diagnostics::DiagnosticError),
    /// Institutional governance evaluation failed.
    #[error(transparent)]
    Governance(#[from] searchright_governance::GovernanceError),
    /// Provider-component verification failed.
    #[error(transparent)]
    Plugin(#[from] searchright_plugin_sdk::PluginError),
    /// Licensed-adapter planning failed.
    #[error(transparent)]
    Licensed(#[from] searchright_licensed::LicensedError),
    /// Provider registry initialisation failed.
    #[error("provider registry failed: {0}")]
    Provider(String),
    /// Endpoint URL was malformed.
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// JSON serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use searchright_contracts::{
        EligibilitySet, FrameworkKind, ProtocolRegistration, QuestionFramework, ResearchQuestion,
        ReviewGovernance, ReviewKind,
    };

    use super::*;

    #[test]
    fn invalid_plan_is_rejected_at_the_shared_facade() {
        let plan = ReviewPlan {
            schema_version: "org.searchright.review-plan.v1".to_owned(),
            review_id: String::new(),
            title: "Example".to_owned(),
            review_kind: ReviewKind::Systematic,
            question: ResearchQuestion {
                text: "Question".to_owned(),
                framework: QuestionFramework {
                    kind: FrameworkKind::Pico,
                    elements: BTreeMap::new(),
                },
                notes: Vec::new(),
            },
            objectives: Vec::new(),
            eligibility: EligibilitySet {
                include: Vec::new(),
                exclude: Vec::new(),
                version: "1".to_owned(),
            },
            information_sources: Vec::new(),
            strategy_ids: Vec::new(),
            protocol: ProtocolRegistration {
                registry: None,
                identifier: None,
                version: "1".to_owned(),
                amendments: Vec::new(),
            },
            governance: ReviewGovernance {
                title_abstract_reviewers: 2,
                full_text_reviewers: 2,
                press_review_required: true,
                protocol_amendment_roles: vec!["principal investigator".to_owned()],
                conflict_resolution: "human adjudication".to_owned(),
            },
        };
        assert!(matches!(
            SearchrightEngine::validate_plan(&plan),
            Err(EngineError::Contract(_))
        ));
    }
}
