//! Product-level application service shared by the CLI, MCP server and Rust consumers.

use std::collections::BTreeMap;

use evidence_search_core::{AuditLedger, AuditVerification, ProviderRegistry, QueryCompiler};
use schemars::JsonSchema;
use searchright_agent::{AgentWorkflow, ReadinessFinding, assess_plan_readiness};
use searchright_contracts::{
    Actor, AuditEvent, AuditEventDraft, BenchmarkReport, BibliographicRecord, ContentSafetyFinding,
    DataHandlingDecision, DataHandlingRequest, Diagnostic, DiscoveryRun, DocumentEvidence,
    ExecutionEnvelope, InstitutionalPolicy, InterchangeFormat, LicensedAdapterProfile,
    LivingUpdateRun, NetworkCapability, PressReview, PrismaFlow, PrismaSLedger, ProtocolAmendment,
    ProviderComponentManifest, ProviderManifest, RankingCalibration, RankingScore, RecordChange,
    RetrievalStatus, ReviewPlan, ScreeningDecision, ScreeningPolicy, SearchDialect, SearchRequest,
    SearchRun, SearchStrategy, SearchValidationReport, SourceReceipt, StandardAssessment,
    StandardPack, StudyGraph, UntrustedContentPolicy, Validate, WorkflowTrace,
};
use searchright_dedup::{DedupConfig, DedupResult, Deduplicator};
use searchright_interchange::ImportResult;
use searchright_provenance::ProvenanceBundle;
use searchright_store::{ExecutionCommit, FileReviewStore};
use searchright_validation::SearchValidationSummary;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::VerifiedEffectAuthority;

const MAXIMUM_LOCAL_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const CONFIRMED_LOCAL_ARTIFACT_SCHEMA_VERSION: &str = "org.searchright.confirmed-local-artifact.v1";
/// Version of the plan-review operation result contract.
pub const PLAN_REVIEW_RESULT_SCHEMA_VERSION: &str = "org.searchright.plan-review-result.v1";
/// Version of the PRESS-review operation result contract.
pub const PRESS_REVIEW_RESULT_SCHEMA_VERSION: &str = "org.searchright.press-review-result.v1";

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

/// Explicit human evidence authorising one bounded local artifact write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HumanConfirmation {
    /// Stable identifier that cannot be reused for different confirmed bytes.
    pub confirmation_id: String,
    /// Human reviewer or approver identifier or pseudonym.
    pub confirmed_by: String,
    /// RFC 3339 timestamp at which the confirmation was recorded.
    pub confirmed_at: String,
}

/// Whether a local review operation is a no-write preview or a confirmed apply.
#[derive(Debug, Clone, Copy)]
pub enum LocalReviewOperation<'a> {
    /// Validate and return the proposed artifact without writing local state.
    Preview,
    /// Persist the exact confirmed envelope as one immutable managed object.
    Apply {
        /// Bounded local store selected by the host, never by untrusted document content.
        store: &'a FileReviewStore,
        /// Explicit human evidence bound into the persisted bytes.
        confirmation: &'a HumanConfirmation,
        /// Opaque grant minted only after trusted host verification.
        authority: &'a VerifiedEffectAuthority,
        /// Exact UTF-8 JSON or YAML submitted to the adapter.
        submitted_document: &'a str,
        /// Submitted document encoding (`json`, `yaml`, or `yml`).
        document_format: &'a str,
    },
}

/// Whether fixture search execution is validation-only or a confirmed immutable apply.
#[derive(Debug, Clone, Copy)]
pub enum SearchExecutionOperation<'a> {
    /// Validate the request and return a no-receipt dry-run description.
    Preview,
    /// Execute a checked-in fixture and persist its receipt and records atomically.
    Apply {
        /// Bounded local store selected by the host.
        store: &'a FileReviewStore,
        /// Stable commit identifier used by the immutable store.
        commit_id: &'a str,
        /// Confirmed local principal recorded in the audit event.
        confirmed_by: &'a str,
        /// Opaque grant minted only after trusted host verification.
        authority: &'a VerifiedEffectAuthority,
    },
}

/// Persistence evidence returned by a preview or confirmed local apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocalPersistenceOutcome {
    /// Whether immutable local bytes were applied.
    pub applied: bool,
    /// Managed-object identifier when applied.
    pub object_id: Option<String>,
    /// SHA-256 digest of the exact persisted envelope when applied.
    pub digest: Option<String>,
    /// Human identifier bound into an applied envelope.
    pub confirmed_by: Option<String>,
}

/// Validated plan plus conservative assessment and local persistence evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlanReviewOutcome {
    /// Versioned operation-result contract identifier.
    pub schema_version: String,
    /// Exact validated plan represented by this operation.
    pub plan: ReviewPlan,
    /// Conservative readiness assessment; this is not plan approval.
    pub assessment: PlanAssessment,
    /// Preview or immutable-apply evidence.
    pub persistence: LocalPersistenceOutcome,
}

/// Validated PRESS record plus local persistence evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PressReviewOutcome {
    /// Versioned operation-result contract identifier.
    pub schema_version: String,
    /// Exact validated PRESS record represented by this operation.
    pub review: PressReview,
    /// Preview or immutable-apply evidence.
    pub persistence: LocalPersistenceOutcome,
}

#[derive(Serialize)]
struct ConfirmedLocalArtifact<'a, T> {
    schema_version: &'static str,
    artifact_kind: &'static str,
    confirmation: &'a HumanConfirmation,
    submitted_document_format: &'a str,
    submitted_document_sha256: String,
    submitted_document: &'a str,
    artifact: &'a T,
}

#[derive(Serialize)]
struct ExecutionBinding<'a> {
    schema_version: &'static str,
    provider_id: &'a str,
    source_label: &'a str,
    request: &'a SearchRequest,
    envelope: &'a ExecutionEnvelope,
    confirmed_by: &'a str,
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
    /// Validate a review plan and optionally persist one explicitly confirmed immutable draft.
    ///
    /// Applying a draft records confirmation of the bytes only. It does not grant execution
    /// authority or represent methodological approval.
    pub fn plan_review(
        plan: &ReviewPlan,
        operation: LocalReviewOperation<'_>,
    ) -> Result<PlanReviewOutcome, EngineError> {
        let assessment = Self::validate_plan(plan)?;
        let persistence = persist_confirmed_artifact(
            "review-plan",
            "plan_review",
            &plan.review_id,
            plan,
            operation,
            None,
        )?;
        Ok(PlanReviewOutcome {
            schema_version: PLAN_REVIEW_RESULT_SCHEMA_VERSION.to_owned(),
            plan: plan.clone(),
            assessment,
            persistence,
        })
    }

    /// Validate a PRESS review and optionally persist one explicitly confirmed immutable record.
    ///
    /// This records reviewer-supplied evidence and does not certify search completeness.
    pub fn press_review_strategy(
        review: &PressReview,
        operation: LocalReviewOperation<'_>,
    ) -> Result<PressReviewOutcome, EngineError> {
        review.validate()?;
        let persistence = persist_confirmed_artifact(
            "press-review",
            "press_review_strategy",
            &review.press_review_id,
            review,
            operation,
            Some(&review.reviewer_id),
        )?;
        Ok(PressReviewOutcome {
            schema_version: PRESS_REVIEW_RESULT_SCHEMA_VERSION.to_owned(),
            review: review.clone(),
            persistence,
        })
    }

    /// Validate or execute one deterministic fixture search under an explicit envelope.
    ///
    /// Live/network execution is deliberately denied until the provider transport closes H-002.
    pub async fn execute_search(
        provider_id: &str,
        source_label: &str,
        request: SearchRequest,
        envelope: &ExecutionEnvelope,
        operation: SearchExecutionOperation<'_>,
    ) -> Result<SearchRun, EngineError> {
        envelope.validate()?;
        if envelope.review_id != request.review_id
            || envelope.network != NetworkCapability::Disabled
            || request.policy.live_enabled
            || request.policy.max_records > envelope.maximum_records
            || request.policy.timeout_seconds > envelope.maximum_seconds
            || request
                .policy
                .total_timeout_seconds
                .is_some_and(|seconds| seconds > envelope.maximum_seconds)
        {
            return Err(EngineError::ExecutionAuthorityDenied);
        }
        let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
        if matches!(operation, SearchExecutionOperation::Preview) {
            if !envelope.dry_run {
                return Err(EngineError::ExecutionAuthorityDenied);
            }
            let run = SearchRun {
                schema_version: searchright_contracts::SEARCH_RUN_SCHEMA_VERSION.to_owned(),
                review_id: request.review_id,
                run_id: request.run_id,
                purpose: "fixture_execution_preview".to_owned(),
                started_at: now,
                completed_at: None,
                receipts: Vec::new(),
                supersedes_run_id: None,
            };
            run.validate()?;
            return Ok(run);
        }
        let SearchExecutionOperation::Apply {
            store,
            commit_id,
            confirmed_by,
            authority,
        } = operation
        else {
            unreachable!("preview returned above")
        };
        if envelope.dry_run
            || !bounded_identifier(commit_id)
            || !bounded_identity(confirmed_by)
            || envelope.approved_by.as_deref() != Some(confirmed_by)
            || !authority.permits("execute_search", &request.review_id, confirmed_by)
        {
            return Err(EngineError::ExecutionAuthorityDenied);
        }
        let binding = ExecutionBinding {
            schema_version: "org.searchright.execution-binding.v1",
            provider_id,
            source_label,
            request: &request,
            envelope,
            confirmed_by,
        };
        let binding_digest = sha256_hex(&serde_json::to_vec(&binding)?);
        if let Some(existing) = store.read_execution_commit(commit_id)? {
            if existing.binding_digest != binding_digest {
                return Err(EngineError::ExecutionCommitConflict);
            }
            return search_run_from_commit(&existing);
        }
        let mut registry = ProviderRegistry::new();
        searchright_connectors::register_mvp_fixtures(&mut registry)
            .map_err(|error| EngineError::Provider(error.to_string()))?;
        let result = registry
            .execute(provider_id, request, source_label)
            .await
            .map_err(|error| EngineError::Provider(error.to_string()))?;
        let mut ledger = AuditLedger::new();
        let audit_event = ledger
            .append(AuditEventDraft {
                schema_version: searchright_contracts::AUDIT_EVENT_SCHEMA_VERSION.to_owned(),
                event_id: format!("execution-{commit_id}"),
                review_id: result.receipt.review_id.clone(),
                event_type: "execution_committed".to_owned(),
                occurred_at: result.receipt.executed_at.clone(),
                actor: Actor {
                    actor_id: confirmed_by.to_owned(),
                    actor_type: "human".to_owned(),
                    provenance: Some("local-current-mcp-confirmation".to_owned()),
                },
                payload: serde_json::json!({
                    "_schema_version": 1,
                    "commit_id": commit_id,
                    "binding_digest": binding_digest,
                    "receipt_id": result.receipt.receipt_id,
                    "record_count": result.records.len(),
                    "run_id": result.receipt.run_id,
                }),
            })?
            .clone();
        store.append_execution_commit(&ExecutionCommit {
            commit_id: commit_id.to_owned(),
            binding_digest,
            receipt: result.receipt.clone(),
            records: result.records,
            audit_event,
        })?;
        let persisted = store
            .read_execution_commit(commit_id)?
            .ok_or(EngineError::ExecutionCommitMissing)?;
        search_run_from_commit(&persisted)
    }

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

    /// Persist one complete, policy-governed screening decision immutably.
    ///
    /// Exact retries are idempotent. The canonical record is the immutable
    /// decision commit in the store, never a replaceable derived snapshot.
    pub fn record_screening_decision(
        store: &FileReviewStore,
        policy: &ScreeningPolicy,
        decision: &ScreeningDecision,
        authority: &VerifiedEffectAuthority,
    ) -> Result<ScreeningDecision, EngineError> {
        if !authority.permits(
            "record_screening_decision",
            &decision.review_id,
            &decision.reviewer_id,
        ) {
            return Err(EngineError::ExecutionAuthorityDenied);
        }
        store.append_screening_decision(policy, decision)?;
        Ok(decision.clone())
    }

    /// Read complete canonical screening decisions for a derived consumer view.
    #[cfg(test)]
    pub(crate) fn screening_decisions(
        store: &FileReviewStore,
    ) -> Result<Vec<ScreeningDecision>, EngineError> {
        Ok(store
            .read_screening_decisions()?
            .into_iter()
            .map(|commit| commit.decision)
            .collect())
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
    ) -> Result<String, EngineError> {
        let endpoint = url::Url::parse(endpoint)?;
        if !endpoint.username().is_empty() || endpoint.password().is_some() {
            return Err(EngineError::CredentialBearingEndpoint);
        }
        searchright_policy::authorise_endpoint(envelope, &endpoint)?;
        Ok(endpoint.origin().ascii_serialization())
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
        let mut registry = ProviderRegistry::new();
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
    /// Endpoint embedded credentials in URL user information.
    #[error("endpoint must not embed credentials")]
    CredentialBearingEndpoint,
    /// JSON serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// RFC 3339 timestamp formatting failed.
    #[error(transparent)]
    Time(#[from] time::error::Format),
    /// Provider execution was not authorised by the bounded fixture envelope.
    #[error("fixture execution authority was absent or inconsistent")]
    ExecutionAuthorityDenied,
    /// An execution idempotency key was already bound to different approved inputs.
    #[error("execution commit id is already bound to different approved inputs")]
    ExecutionCommitConflict,
    /// A just-written execution commit could not be read back and validated.
    #[error("execution commit was not durably observable after persistence")]
    ExecutionCommitMissing,
    /// Durable screening-decision validation or persistence failed.
    #[error(transparent)]
    Store(#[from] searchright_store::StoreError),
    /// Human confirmation was absent, malformed or did not match the artifact reviewer.
    #[error("local review apply requires valid matching human confirmation")]
    InvalidHumanConfirmation,
    /// A local artifact exceeded the bounded immutable-store limit.
    #[error("local review artifact exceeds the 16 MiB persistence limit")]
    LocalArtifactTooLarge,
    /// The immutable local store rejected the write.
    #[error("local review persistence rejected the immutable write: {0}")]
    LocalPersistence(String),
    /// The exact submitted document could not be reparsed at the facade boundary.
    #[error("submitted review document is invalid: {0}")]
    InvalidSubmittedDocument(String),
    /// The exact submitted document used an unsupported encoding label.
    #[error("submitted review document format must be json, yaml, or yml")]
    UnsupportedDocumentFormat,
    /// The exact submitted document did not represent the validated artifact.
    #[error("submitted review document does not match the validated artifact")]
    SubmittedDocumentMismatch,
}

fn persist_confirmed_artifact<T: Serialize + DeserializeOwned + PartialEq>(
    artifact_kind: &'static str,
    authority_tool: &'static str,
    authority_review_id: &str,
    artifact: &T,
    operation: LocalReviewOperation<'_>,
    required_confirmer: Option<&str>,
) -> Result<LocalPersistenceOutcome, EngineError> {
    let artifact_bytes = serde_json::to_vec(artifact)?;
    ensure_local_artifact_bound(&artifact_bytes)?;
    let LocalReviewOperation::Apply {
        store,
        confirmation,
        authority,
        submitted_document,
        document_format,
    } = operation
    else {
        return Ok(LocalPersistenceOutcome {
            applied: false,
            object_id: None,
            digest: None,
            confirmed_by: None,
        });
    };
    validate_human_confirmation(confirmation)?;
    if !authority.permits(
        authority_tool,
        authority_review_id,
        &confirmation.confirmed_by,
    ) {
        return Err(EngineError::ExecutionAuthorityDenied);
    }
    let reparsed: T = match document_format {
        "json" => serde_json::from_str(submitted_document)?,
        "yaml" | "yml" => serde_yaml::from_str(submitted_document)
            .map_err(|error| EngineError::InvalidSubmittedDocument(error.to_string()))?,
        _ => return Err(EngineError::UnsupportedDocumentFormat),
    };
    if &reparsed != artifact {
        return Err(EngineError::SubmittedDocumentMismatch);
    }
    if required_confirmer.is_some_and(|reviewer| reviewer != confirmation.confirmed_by) {
        return Err(EngineError::InvalidHumanConfirmation);
    }
    let envelope = ConfirmedLocalArtifact {
        schema_version: CONFIRMED_LOCAL_ARTIFACT_SCHEMA_VERSION,
        artifact_kind,
        confirmation,
        submitted_document_format: document_format,
        submitted_document_sha256: sha256_hex(submitted_document.as_bytes()),
        submitted_document,
        artifact,
    };
    let bytes = serde_json::to_vec(&envelope)?;
    ensure_local_artifact_bound(&bytes)?;
    let object_id = format!("{artifact_kind}.{}", confirmation.confirmation_id);
    let receipt = store
        .put_managed_object(&object_id, &bytes)
        .map_err(EngineError::LocalPersistence)?;
    Ok(LocalPersistenceOutcome {
        applied: true,
        object_id: Some(receipt.object_id),
        digest: Some(receipt.digest),
        confirmed_by: Some(confirmation.confirmed_by.clone()),
    })
}

fn search_run_from_commit(commit: &ExecutionCommit) -> Result<SearchRun, EngineError> {
    let run = SearchRun {
        schema_version: searchright_contracts::SEARCH_RUN_SCHEMA_VERSION.to_owned(),
        review_id: commit.receipt.review_id.clone(),
        run_id: commit.receipt.run_id.clone(),
        purpose: "fixture_execution".to_owned(),
        started_at: commit.receipt.executed_at.clone(),
        completed_at: Some(commit.receipt.executed_at.clone()),
        receipts: vec![commit.receipt.clone()],
        supersedes_run_id: None,
    };
    run.validate()?;
    Ok(run)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_digit(byte >> 4));
        encoded.push(hex_digit(byte & 0x0f));
    }
    encoded
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + nibble - 10),
        _ => '?',
    }
}

const fn ensure_local_artifact_bound(bytes: &[u8]) -> Result<(), EngineError> {
    if bytes.len() > MAXIMUM_LOCAL_ARTIFACT_BYTES {
        Err(EngineError::LocalArtifactTooLarge)
    } else {
        Ok(())
    }
}

fn validate_human_confirmation(confirmation: &HumanConfirmation) -> Result<(), EngineError> {
    if !bounded_identifier(&confirmation.confirmation_id)
        || !bounded_identity(&confirmation.confirmed_by)
        || OffsetDateTime::parse(&confirmation.confirmed_at, &Rfc3339).is_err()
    {
        return Err(EngineError::InvalidHumanConfirmation);
    }
    Ok(())
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with('.')
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn bounded_identity(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use searchright_contracts::{
        EligibilitySet, ExecutionPolicy, FrameworkKind, FullTextHandling, ProtocolRegistration,
        QuestionFramework, ResearchQuestion, ReviewGovernance, ReviewKind, SecretHandling,
    };

    use super::*;

    static TEMPORARY_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestVerifier;

    impl crate::EffectAuthorityVerifier for TestVerifier {
        fn verify(
            &self,
            request: &crate::EffectAuthorityRequest,
        ) -> Result<crate::EffectAuthorityAttestation, crate::EffectAuthorityError> {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| crate::EffectAuthorityError)?
                .as_secs();
            Ok(crate::EffectAuthorityAttestation {
                tool_name: request.tool_name.clone(),
                request_digest: request.request_digest.clone(),
                review_id: request.review_id.clone(),
                principal: request.principal_hint.clone(),
                policy_digest: request.policy_digest.clone(),
                store_state_digest: request.store_state_digest.clone(),
                nonce: format!(
                    "test-authority-{:016}",
                    TEMPORARY_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
                ),
                issued_at_unix_seconds: now,
                expires_at_unix_seconds: now + 60,
            })
        }
    }

    fn authority(tool: &str, review_id: &str, principal: &str) -> VerifiedEffectAuthority {
        crate::verify_effect_authority(
            &TestVerifier,
            &crate::EffectAuthorityRequest {
                tool_name: tool.to_owned(),
                request_digest: "test-request-digest".to_owned(),
                review_id: review_id.to_owned(),
                principal_hint: principal.to_owned(),
                policy_digest: None,
                store_state_digest: "test-store-state".to_owned(),
            },
        )
        .unwrap_or_else(|_| unreachable!())
    }

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new(label: &str) -> Result<Self, std::io::Error> {
            let sequence = TEMPORARY_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "searchright-engine-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _cleanup_result = fs::remove_dir_all(&self.0);
        }
    }

    fn valid_plan() -> Result<ReviewPlan, serde_yaml::Error> {
        serde_yaml::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))
    }

    fn valid_press_review() -> Result<PressReview, serde_json::Error> {
        serde_json::from_str(include_str!(
            "../../../contracts/examples/press-review.json"
        ))
    }

    fn valid_screening_policy() -> Result<ScreeningPolicy, serde_yaml::Error> {
        serde_yaml::from_str(include_str!(
            "../../../contracts/examples/screening-policy.yaml"
        ))
    }

    fn valid_screening_decision() -> Result<ScreeningDecision, serde_yaml::Error> {
        serde_yaml::from_str(include_str!(
            "../../../contracts/examples/screening-decision.yaml"
        ))
    }

    fn confirmation(id: &str, confirmed_by: &str) -> HumanConfirmation {
        HumanConfirmation {
            confirmation_id: id.to_owned(),
            confirmed_by: confirmed_by.to_owned(),
            confirmed_at: "2026-08-29T00:00:00Z".to_owned(),
        }
    }

    fn fixture_request() -> Result<SearchRequest, serde_yaml::Error> {
        Ok(SearchRequest {
            review_id: "fixture-review".to_owned(),
            run_id: "fixture-run".to_owned(),
            strategy: serde_yaml::from_str(include_str!(
                "../../../contracts/examples/compiled-strategy.yaml"
            ))?,
            cursor: None,
            page_size: 10,
            policy: ExecutionPolicy {
                live_enabled: false,
                max_records: 10,
                max_pages: 1,
                timeout_seconds: 10,
                total_timeout_seconds: Some(10),
                max_retries: 0,
                min_interval_ms: 0,
                retry_base_delay_ms: None,
                retry_max_delay_ms: None,
                max_response_bytes: Some(1024 * 1024),
                replay_enabled: true,
                cache_write_enabled: false,
            },
        })
    }

    fn fixture_envelope(dry_run: bool) -> ExecutionEnvelope {
        ExecutionEnvelope {
            schema_version: searchright_contracts::EXECUTION_ENVELOPE_SCHEMA_VERSION.to_owned(),
            operation_id: "fixture-operation".to_owned(),
            review_id: "fixture-review".to_owned(),
            network: NetworkCapability::Disabled,
            allowed_hosts: Vec::new(),
            secret_handling: SecretHandling::None,
            full_text_handling: FullTextHandling::MetadataOnly,
            untrusted_content: UntrustedContentPolicy::DataOnly,
            maximum_records: 10,
            maximum_seconds: 10,
            dry_run,
            approved_by: (!dry_run).then(|| "review-lead".to_owned()),
        }
    }

    #[tokio::test]
    async fn fixture_execution_previews_then_commits_without_network()
    -> Result<(), Box<dyn std::error::Error>> {
        let preview = SearchrightEngine::execute_search(
            "pubmed-fixture",
            "fixture",
            fixture_request()?,
            &fixture_envelope(true),
            SearchExecutionOperation::Preview,
        )
        .await?;
        assert!(preview.receipts.is_empty());

        let directory = TemporaryDirectory::new("fixture-execution")?;
        let store = FileReviewStore::open(&directory.0)?;
        let execution_authority = authority("execute_search", "fixture-review", "review-lead");
        let applied = SearchrightEngine::execute_search(
            "pubmed-fixture",
            "fixture",
            fixture_request()?,
            &fixture_envelope(false),
            SearchExecutionOperation::Apply {
                store: &store,
                commit_id: "fixture-commit",
                confirmed_by: "review-lead",
                authority: &execution_authority,
            },
        )
        .await?;
        assert_eq!(applied.receipts.len(), 1);
        assert!(store.root().join("commits/fixture-commit.json").is_file());
        let exact_retry = SearchrightEngine::execute_search(
            "pubmed-fixture",
            "fixture",
            fixture_request()?,
            &fixture_envelope(false),
            SearchExecutionOperation::Apply {
                store: &store,
                commit_id: "fixture-commit",
                confirmed_by: "review-lead",
                authority: &execution_authority,
            },
        )
        .await?;
        assert_eq!(exact_retry, applied);

        let mut changed_request = fixture_request()?;
        changed_request.page_size = changed_request.page_size.saturating_add(1);
        assert!(matches!(
            SearchrightEngine::execute_search(
                "pubmed-fixture",
                "fixture",
                changed_request,
                &fixture_envelope(false),
                SearchExecutionOperation::Apply {
                    store: &store,
                    commit_id: "fixture-commit",
                    confirmed_by: "review-lead",
                    authority: &execution_authority,
                },
            )
            .await,
            Err(EngineError::ExecutionCommitConflict)
        ));

        let mut live = fixture_request()?;
        live.policy.live_enabled = true;
        assert!(matches!(
            SearchrightEngine::execute_search(
                "pubmed-fixture",
                "fixture",
                live,
                &fixture_envelope(false),
                SearchExecutionOperation::Apply {
                    store: &store,
                    commit_id: "live-denied",
                    confirmed_by: "review-lead",
                    authority: &execution_authority,
                },
            )
            .await,
            Err(EngineError::ExecutionAuthorityDenied)
        ));
        Ok(())
    }

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

    #[test]
    fn plan_review_preview_validates_without_creating_managed_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = TemporaryDirectory::new("plan-preview")?;
        let store = FileReviewStore::open(&directory.0)?;
        let plan = valid_plan()?;

        let outcome = SearchrightEngine::plan_review(&plan, LocalReviewOperation::Preview)?;

        assert_eq!(outcome.plan, plan);
        assert!(!outcome.persistence.applied);
        assert!(outcome.persistence.object_id.is_none());
        assert!(!store.root().join("managed").exists());
        Ok(())
    }

    #[test]
    fn confirmed_plan_apply_is_exact_idempotent_and_conflicts_fail_closed()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = TemporaryDirectory::new("plan-apply")?;
        let store = FileReviewStore::open(&directory.0)?;
        let plan = valid_plan()?;
        let submitted_document = include_str!("../../../contracts/examples/review-plan.yaml");
        let confirmation = confirmation("confirm-plan-1", "principal-investigator-1");
        let plan_authority = authority("plan_review", &plan.review_id, "principal-investigator-1");
        let operation = LocalReviewOperation::Apply {
            store: &store,
            confirmation: &confirmation,
            authority: &plan_authority,
            submitted_document,
            document_format: "yaml",
        };

        let first = SearchrightEngine::plan_review(&plan, operation)?;
        let second = SearchrightEngine::plan_review(&plan, operation)?;
        assert_eq!(first.persistence, second.persistence);
        assert_eq!(
            first.persistence.object_id.as_deref(),
            Some("review-plan.confirm-plan-1")
        );
        let envelope = ConfirmedLocalArtifact {
            schema_version: CONFIRMED_LOCAL_ARTIFACT_SCHEMA_VERSION,
            artifact_kind: "review-plan",
            confirmation: &confirmation,
            submitted_document_format: "yaml",
            submitted_document_sha256: sha256_hex(submitted_document.as_bytes()),
            submitted_document,
            artifact: &plan,
        };
        let expected = serde_json::to_vec(&envelope)?;
        assert_eq!(
            fs::read(store.root().join("managed/review-plan.confirm-plan-1"))?,
            expected
        );

        let mut changed = plan;
        changed.title.push_str(" revised");
        let changed_document = serde_json::to_string(&changed)?;
        assert!(matches!(
            SearchrightEngine::plan_review(
                &changed,
                LocalReviewOperation::Apply {
                    store: &store,
                    confirmation: &confirmation,
                    authority: &plan_authority,
                    submitted_document: &changed_document,
                    document_format: "json",
                },
            ),
            Err(EngineError::LocalPersistence(_))
        ));
        assert_eq!(
            fs::read(store.root().join("managed/review-plan.confirm-plan-1"))?,
            expected
        );
        Ok(())
    }

    #[test]
    fn malformed_human_confirmation_is_rejected_before_any_write()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = TemporaryDirectory::new("invalid-confirmation")?;
        let store = FileReviewStore::open(&directory.0)?;
        let plan = valid_plan()?;
        let submitted_document = include_str!("../../../contracts/examples/review-plan.yaml");
        let confirmation = HumanConfirmation {
            confirmation_id: "../escape".to_owned(),
            confirmed_by: "principal-investigator-1".to_owned(),
            confirmed_at: "not-a-time".to_owned(),
        };
        let plan_authority = authority("plan_review", &plan.review_id, "principal-investigator-1");

        assert!(matches!(
            SearchrightEngine::plan_review(
                &plan,
                LocalReviewOperation::Apply {
                    store: &store,
                    confirmation: &confirmation,
                    authority: &plan_authority,
                    submitted_document,
                    document_format: "yaml",
                },
            ),
            Err(EngineError::InvalidHumanConfirmation)
        ));
        assert!(!store.root().join("managed").exists());
        Ok(())
    }

    #[test]
    fn press_apply_binds_confirmation_to_reviewer_and_preserves_exact_bytes()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = TemporaryDirectory::new("press-apply")?;
        let store = FileReviewStore::open(&directory.0)?;
        let review = valid_press_review()?;
        let submitted_document = include_str!("../../../contracts/examples/press-review.json");
        let wrong = confirmation("confirm-press-wrong", "different-reviewer");
        let wrong_authority = authority(
            "press_review_strategy",
            &review.press_review_id,
            "different-reviewer",
        );
        assert!(matches!(
            SearchrightEngine::press_review_strategy(
                &review,
                LocalReviewOperation::Apply {
                    store: &store,
                    confirmation: &wrong,
                    authority: &wrong_authority,
                    submitted_document,
                    document_format: "json",
                },
            ),
            Err(EngineError::InvalidHumanConfirmation)
        ));
        assert!(!store.root().join("managed").exists());

        let confirmation = confirmation("confirm-press-1", &review.reviewer_id);
        let press_authority = authority(
            "press_review_strategy",
            &review.press_review_id,
            &review.reviewer_id,
        );
        let outcome = SearchrightEngine::press_review_strategy(
            &review,
            LocalReviewOperation::Apply {
                store: &store,
                confirmation: &confirmation,
                authority: &press_authority,
                submitted_document,
                document_format: "json",
            },
        )?;
        assert!(outcome.persistence.applied);
        let envelope = ConfirmedLocalArtifact {
            schema_version: CONFIRMED_LOCAL_ARTIFACT_SCHEMA_VERSION,
            artifact_kind: "press-review",
            confirmation: &confirmation,
            submitted_document_format: "json",
            submitted_document_sha256: sha256_hex(submitted_document.as_bytes()),
            submitted_document,
            artifact: &review,
        };
        assert_eq!(
            fs::read(store.root().join("managed/press-review.confirm-press-1"))?,
            serde_json::to_vec(&envelope)?
        );
        Ok(())
    }

    #[test]
    fn screening_facade_uses_immutable_store_and_denies_agent_exclusion()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = TemporaryDirectory::new("screening-decision")?;
        let store = FileReviewStore::open(&directory.0)?;
        let policy = valid_screening_policy()?;
        let decision = valid_screening_decision()?;
        let screening_authority = authority(
            "record_screening_decision",
            &decision.review_id,
            &decision.reviewer_id,
        );

        assert_eq!(
            SearchrightEngine::record_screening_decision(
                &store,
                &policy,
                &decision,
                &screening_authority,
            )?,
            decision
        );
        assert_eq!(
            SearchrightEngine::record_screening_decision(
                &store,
                &policy,
                &decision,
                &screening_authority,
            )?,
            decision
        );
        assert_eq!(
            SearchrightEngine::screening_decisions(&store)?,
            vec![decision.clone()]
        );
        assert!(!store.root().join("snapshots/screening").exists());

        let mut agent = decision;
        agent.decision_id = "agent-exclusion".to_owned();
        agent.reviewer_id = "agent-1".to_owned();
        agent.reviewer_kind = searchright_contracts::ReviewerKind::Agent;
        agent.confidence = Some(0.995);
        agent.agent_provenance = Some("model=fixture;prompt=sha256:test".to_owned());
        let agent_authority = authority(
            "record_screening_decision",
            &agent.review_id,
            &agent.reviewer_id,
        );
        assert!(matches!(
            SearchrightEngine::record_screening_decision(&store, &policy, &agent, &agent_authority,),
            Err(EngineError::Store(
                searchright_store::StoreError::InvalidScreeningDecision(_)
            ))
        ));
        assert_eq!(SearchrightEngine::screening_decisions(&store)?.len(), 1);
        Ok(())
    }
}
