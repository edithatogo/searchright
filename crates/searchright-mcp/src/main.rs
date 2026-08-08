#![forbid(unsafe_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use rmcp::{
    ErrorData as McpError, ServiceExt,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
    transport::stdio,
};
use searchright::{PrismaArtifact, PrismaOutput, SearchrightEngine};
use searchright::contracts::{
    AuditEvent, BenchmarkReport, BibliographicRecord, CompiledStrategy, DataHandlingRequest,
    Diagnostic, DiscoveryRun, DocumentEvidence, ExecutionEnvelope, InstitutionalPolicy, InterchangeFormat,
    LicensedAdapterProfile, LivingUpdateRun, PrismaFlow, ProtocolAmendment,
    ProviderComponentManifest, RankingCalibration, ReviewPlan, SearchDialect, SearchStrategy,
    SearchValidationReport, SourceReceipt, StandardAssessment, StandardPack, StudyGraph,
    UntrustedContentPolicy, WorkflowTrace,
};
use searchright::dedup::DedupConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DocumentInput {
    /// JSON or YAML document text.
    document: String,
    /// `json` or `yaml`; defaults to JSON.
    format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CompileInput {
    /// JSON or YAML SearchStrategy document.
    document: String,
    /// `json` or `yaml`; defaults to JSON.
    format: Option<String>,
    /// Target dialect such as pubmed, embase or europe_pmc.
    dialect: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DedupInput {
    /// JSON array of BibliographicRecord contracts.
    records_json: String,
    /// Optional title-similarity threshold, zero to one.
    title_threshold: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrismaInput {
    /// JSON PrismaFlow contract.
    flow_json: String,
    /// `json`, `mermaid` or `prisma_s_ledger`.
    output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AuditInput {
    /// JSONL audit events.
    audit_jsonl: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ImportInput {
    /// Bibliographic interchange document.
    document: String,
    /// `searchright_json`, `json_lines`, `csl_json`, `ris`, `nbib` or `csv`.
    input_format: String,
    /// Source receipt that proves where the import came from.
    source_receipt_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExportInput {
    /// JSON array of canonical BibliographicRecord contracts.
    records_json: String,
    /// Review identifier for the conversion receipt.
    review_id: String,
    /// Declared canonical input format.
    input_format: String,
    /// Requested output format.
    output_format: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LivingDiffInput {
    /// JSON array from the parent run.
    previous_records_json: String,
    /// JSON array from the current run.
    current_records_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProvenanceInput {
    /// JSON ReviewPlan contract.
    plan_json: String,
    /// JSON array of SourceReceipt contracts.
    receipts_json: String,
    /// JSON array of AuditEvent contracts.
    events_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RankInput {
    /// JSON array of canonical BibliographicRecord contracts.
    records_json: String,
    /// Transparent query terms used for advisory ranking.
    query_terms: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ContentInput {
    /// Stable identifier for the source object.
    subject_id: String,
    /// Untrusted text to inspect as inert data.
    text: String,
    /// `data_only`, `sanitise_then_data_only` or `human_inspection_required`.
    policy: Option<String>,
}


#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DiagnosticsInput {
    /// JSON or YAML array of Diagnostic contracts.
    document: String,
    /// `json` or `yaml`; defaults to JSON.
    format: Option<String>,
    /// `plain_text`, `json` or `json_lines`.
    output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GovernanceInput {
    /// JSON InstitutionalPolicy contract.
    policy_json: String,
    /// JSON DataHandlingRequest contract.
    request_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EndpointInput {
    /// JSON ExecutionEnvelope contract.
    envelope_json: String,
    /// Requested HTTPS endpoint.
    endpoint: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ComponentInput {
    /// JSON ProviderComponentManifest contract.
    manifest_json: String,
    /// Base64-encoded exact component bytes.
    component_base64: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LicensedPlanInput {
    /// JSON LicensedAdapterProfile contract.
    profile_json: String,
    /// JSON CompiledStrategy contract.
    compiled_strategy_json: String,
    /// Redacted HTTPS endpoint to authorise.
    endpoint: String,
}

#[derive(Clone, Default)]
struct SearchrightServer;

#[tool_router(server_handler)]
impl SearchrightServer {
    #[tool(description = "Read-only: validate a review plan and return conservative readiness findings")]
    fn validate_plan(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let plan: ReviewPlan = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        operation_result(SearchrightEngine::validate_plan(&plan))
    }

    #[tool(description = "Read-only: validate a source-specific search strategy without execution")]
    fn validate_strategy(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let strategy: SearchStrategy = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result(
            "search_strategy",
            SearchrightEngine::validate_strategy(&strategy),
        )
    }

    #[tool(description = "Read-only: validate neutral CiteWeft-compatible document evidence; canonical writes are forbidden")]
    fn validate_document_evidence(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let evidence: DocumentEvidence = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result(
            "document_evidence",
            SearchrightEngine::validate_document_evidence(&evidence),
        )
    }

    #[tool(description = "Read-only: compile a portable query AST and expose fidelity, loss codes and the human-review gate")]
    fn compile_strategy(
        &self,
        Parameters(input): Parameters<CompileInput>,
    ) -> Result<CallToolResult, McpError> {
        let strategy: SearchStrategy = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        let dialect = parse_dialect(&input.dialect).map_err(invalid_params)?;
        operation_result(SearchrightEngine::compile_strategy(&strategy, dialect))
    }

    #[tool(description = "Read-only preview: generate reviewable duplicate clusters without deleting records")]
    fn deduplicate_records(
        &self,
        Parameters(input): Parameters<DedupInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(json_invalid_params)?;
        let threshold = input.title_threshold.unwrap_or(0.92);
        if !(0.0..=1.0).contains(&threshold) {
            return Err(invalid_params(
                "title_threshold must be between zero and one".to_owned(),
            ));
        }
        operation_result(SearchrightEngine::deduplicate(
            &records,
            DedupConfig {
                title_similarity_threshold: threshold,
                ..DedupConfig::default()
            },
        ))
    }

    #[tool(description = "Read-only: validate PRISMA arithmetic and render JSON, Mermaid or the PRISMA-S ledger")]
    fn generate_prisma(
        &self,
        Parameters(input): Parameters<PrismaInput>,
    ) -> Result<CallToolResult, McpError> {
        let flow: PrismaFlow = serde_json::from_str(&input.flow_json).map_err(json_invalid_params)?;
        let output = match input.output.as_str() {
            "json" => PrismaOutput::Json,
            "mermaid" => PrismaOutput::Mermaid,
            "prisma_s_ledger" => PrismaOutput::PrismaSLedger,
            _ => {
                return Err(invalid_params(
                    "output must be json, mermaid or prisma_s_ledger".to_owned(),
                ));
            }
        };
        match SearchrightEngine::prisma(&flow, output) {
            Ok(PrismaArtifact::Mermaid(document)) => {
                Ok(CallToolResult::success(vec![ContentBlock::text(document)]))
            }
            Ok(artifact) => json_success(&artifact),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only: verify a hash-chained Searchright audit JSONL stream")]
    fn verify_audit(
        &self,
        Parameters(input): Parameters<AuditInput>,
    ) -> Result<CallToolResult, McpError> {
        let events = parse_jsonl::<AuditEvent>(&input.audit_jsonl).map_err(invalid_params)?;
        operation_result(SearchrightEngine::verify_audit(events))
    }

    #[tool(description = "Read-only import: normalise a bibliographic interchange document while retaining provenance")]
    fn import_records(
        &self,
        Parameters(input): Parameters<ImportInput>,
    ) -> Result<CallToolResult, McpError> {
        let format = parse_interchange(&input.input_format).map_err(invalid_params)?;
        operation_result(SearchrightEngine::import_records(
            &input.document,
            format,
            &input.source_receipt_id,
        ))
    }

    #[tool(description = "Read-only conversion: export canonical records and return a deterministic conversion receipt")]
    fn export_records(
        &self,
        Parameters(input): Parameters<ExportInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(json_invalid_params)?;
        let input_format = parse_interchange(&input.input_format).map_err(invalid_params)?;
        let output_format = parse_interchange(&input.output_format).map_err(invalid_params)?;
        operation_result(SearchrightEngine::export_records(
            &input.review_id,
            &records,
            input_format,
            output_format,
        ))
    }

    #[tool(description = "Read-only: validate and summarise a record-report-study/full-text graph")]
    fn assess_study_graph(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let graph: StudyGraph = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        operation_result(SearchrightEngine::assess_study_graph(&graph))
    }

    #[tool(description = "Read-only gate: assess PRESS findings, seed recall and translation-loss approval")]
    fn assess_search_validation(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let report: SearchValidationReport =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result(SearchrightEngine::assess_search_validation(&report))
    }

    #[tool(description = "Read-only: compare parent and current records for a living-review update")]
    fn living_diff(
        &self,
        Parameters(input): Parameters<LivingDiffInput>,
    ) -> Result<CallToolResult, McpError> {
        let previous: Vec<BibliographicRecord> =
            serde_json::from_str(&input.previous_records_json).map_err(json_invalid_params)?;
        let current: Vec<BibliographicRecord> =
            serde_json::from_str(&input.current_records_json).map_err(json_invalid_params)?;
        operation_result(SearchrightEngine::diff_living_records(&previous, &current))
    }

    #[tool(description = "Read-only: validate immutable living-review run lineage")]
    fn validate_living_lineage(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let runs: Vec<LivingUpdateRun> = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result(
            "living_lineage",
            SearchrightEngine::validate_living_lineage(&runs),
        )
    }

    #[tool(description = "Read-only: build RO-Crate 1.3 and W3C PROV-compatible review provenance")]
    fn build_provenance(
        &self,
        Parameters(input): Parameters<ProvenanceInput>,
    ) -> Result<CallToolResult, McpError> {
        let plan: ReviewPlan = serde_json::from_str(&input.plan_json).map_err(json_invalid_params)?;
        let receipts: Vec<SourceReceipt> =
            serde_json::from_str(&input.receipts_json).map_err(json_invalid_params)?;
        let events: Vec<AuditEvent> =
            serde_json::from_str(&input.events_json).map_err(json_invalid_params)?;
        operation_result(SearchrightEngine::provenance(&plan, &receipts, &events))
    }

    #[tool(description = "Read-only advisory ranking: prioritise records transparently; never make final exclusions")]
    fn rank_records(
        &self,
        Parameters(input): Parameters<RankInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(json_invalid_params)?;
        operation_result(SearchrightEngine::rank_records(&records, &input.query_terms))
    }

    #[tool(description = "Read-only safety check: treat provider/full-text content as inert data and flag instruction-like markers")]
    fn inspect_untrusted_content(
        &self,
        Parameters(input): Parameters<ContentInput>,
    ) -> Result<CallToolResult, McpError> {
        let policy = parse_content_policy(input.policy.as_deref().unwrap_or("data_only"))
            .map_err(invalid_params)?;
        json_success(&SearchrightEngine::inspect_content(
            &input.subject_id,
            &input.text,
            policy,
        ))
    }


    #[tool(description = "Read-only accessibility surface: validate and render stable diagnostics without ANSI-dependent output")]
    fn render_diagnostics(
        &self,
        Parameters(input): Parameters<DiagnosticsInput>,
    ) -> Result<CallToolResult, McpError> {
        let diagnostics: Vec<Diagnostic> = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        let output = match input.output.as_str() {
            "plain_text" => searchright::diagnostics::DiagnosticOutput::PlainText,
            "json" => searchright::diagnostics::DiagnosticOutput::Json,
            "json_lines" => searchright::diagnostics::DiagnosticOutput::JsonLines,
            _ => {
                return Err(invalid_params(
                    "output must be plain_text, json or json_lines".to_owned(),
                ));
            }
        };
        match SearchrightEngine::render_diagnostics(&diagnostics, output) {
            Ok(document) => Ok(CallToolResult::success(vec![ContentBlock::text(document)])),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only governance gate: evaluate a data-handling request against an institutional policy")]
    fn evaluate_governance(
        &self,
        Parameters(input): Parameters<GovernanceInput>,
    ) -> Result<CallToolResult, McpError> {
        let policy: InstitutionalPolicy =
            serde_json::from_str(&input.policy_json).map_err(json_invalid_params)?;
        let request: DataHandlingRequest =
            serde_json::from_str(&input.request_json).map_err(json_invalid_params)?;
        operation_result(SearchrightEngine::evaluate_governance(&policy, &request))
    }

    #[tool(description = "Read-only policy gate: verify that an HTTPS endpoint is permitted by an execution envelope")]
    fn authorise_endpoint(
        &self,
        Parameters(input): Parameters<EndpointInput>,
    ) -> Result<CallToolResult, McpError> {
        let envelope: ExecutionEnvelope =
            serde_json::from_str(&input.envelope_json).map_err(json_invalid_params)?;
        match SearchrightEngine::authorise_endpoint(&envelope, &input.endpoint) {
            Ok(()) => json_success(&serde_json::json!({
                "authorised": true,
                "endpoint": input.endpoint,
            })),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only: validate a versioned protocol amendment")]
    fn validate_amendment(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: ProtocolAmendment = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result("protocol_amendment", SearchrightEngine::validate_amendment(&value))
    }

    #[tool(description = "Read-only: validate a methodological standards pack")]
    fn validate_standard_pack(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: StandardPack = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result("standard_pack", SearchrightEngine::validate_standard_pack(&value))
    }

    #[tool(description = "Read-only: validate an evidence-linked standards assessment")]
    fn validate_standard_assessment(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: StandardAssessment =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "standard_assessment",
            SearchrightEngine::validate_standard_assessment(&value),
        )
    }

    #[tool(description = "Read-only: validate ranking calibration and the no-auto-exclusion invariant")]
    fn validate_ranking_calibration(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: RankingCalibration =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "ranking_calibration",
            SearchrightEngine::validate_ranking_calibration(&value),
        )
    }

    #[tool(description = "Read-only: validate one bounded supplementary-discovery run")]
    fn validate_discovery_run(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: DiscoveryRun = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        validation_result("discovery_run", SearchrightEngine::validate_discovery_run(&value))
    }

    #[tool(description = "Read-only assurance: verify lifecycle continuity, declared transitions and human approval gates")]
    fn verify_workflow_trace(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let trace: WorkflowTrace = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        operation_result(SearchrightEngine::verify_workflow_trace(&trace))
    }

    #[tool(description = "Read-only discovery: resolve bounded citation/grey-literature candidates pending human release")]
    fn discovery_candidates(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let run: DiscoveryRun = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        operation_result(SearchrightEngine::discovery_candidates(&run))
    }

    #[tool(description = "Read-only supply-chain gate: verify a provider manifest against exact base64 component bytes")]
    fn verify_provider_component(
        &self,
        Parameters(input): Parameters<ComponentInput>,
    ) -> Result<CallToolResult, McpError> {
        let manifest: ProviderComponentManifest =
            serde_json::from_str(&input.manifest_json).map_err(json_invalid_params)?;
        let bytes = BASE64_STANDARD
            .decode(input.component_base64.as_bytes())
            .map_err(|error| invalid_params(error.to_string()))?;
        match SearchrightEngine::verify_provider_component(&manifest, &bytes) {
            Ok(()) => json_success(&serde_json::json!({
                "valid": true,
                "component_id": manifest.component_id,
                "bytes": bytes.len(),
            })),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only licensed-source gate: build a redacted BYO-access request plan without exposing credentials")]
    fn plan_licensed_request(
        &self,
        Parameters(input): Parameters<LicensedPlanInput>,
    ) -> Result<CallToolResult, McpError> {
        let profile: LicensedAdapterProfile =
            serde_json::from_str(&input.profile_json).map_err(json_invalid_params)?;
        let strategy: CompiledStrategy =
            serde_json::from_str(&input.compiled_strategy_json).map_err(json_invalid_params)?;
        operation_result(SearchrightEngine::plan_licensed_request(
            &profile,
            &strategy,
            &input.endpoint,
        ))
    }

    #[tool(description = "Read-only evidence gate: validate a benchmark report and its explicit claim boundary")]
    fn validate_benchmark_report(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let report: BenchmarkReport = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        match SearchrightEngine::validate_benchmark_report(&report) {
            Ok(()) => json_success(&serde_json::json!({
                "valid": true,
                "benchmark_id": report.benchmark_id,
            })),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only: list deterministic no-network provider manifests available by default")]
    fn list_providers(&self) -> Result<CallToolResult, McpError> {
        operation_result(SearchrightEngine::default_provider_manifests())
    }

    #[tool(description = "Read-only: return the conservative planning, execution, screening, reporting and update workflow")]
    fn workflow(&self) -> Result<CallToolResult, McpError> {
        json_success(&SearchrightEngine::workflow())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    let service = SearchrightServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

fn parse_document<T: serde::de::DeserializeOwned>(
    document: &str,
    format: Option<&str>,
) -> Result<T, String> {
    match format.unwrap_or("json") {
        "yaml" | "yml" => serde_yaml::from_str(document).map_err(|error| error.to_string()),
        "json" => serde_json::from_str(document).map_err(|error| error.to_string()),
        other => Err(format!("unsupported document format `{other}`")),
    }
}

fn parse_jsonl<T: serde::de::DeserializeOwned>(document: &str) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    for (index, line) in document.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            format!(
                "invalid JSONL value at line {}: {error}",
                index.saturating_add(1)
            )
        })?);
    }
    Ok(values)
}

fn parse_dialect(value: &str) -> Result<SearchDialect, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pubmed" | "pub_med" => Ok(SearchDialect::PubMed),
        "ovid_medline" | "ovid-medline" => Ok(SearchDialect::OvidMedline),
        "embase" => Ok(SearchDialect::Embase),
        "europe_pmc" | "europe-pmc" => Ok(SearchDialect::EuropePmc),
        "cinahl_ebsco" | "cinahl-ebsco" => Ok(SearchDialect::CinahlEbsco),
        "psycinfo_ovid" | "psycinfo-ovid" => Ok(SearchDialect::PsycInfoOvid),
        "scopus" => Ok(SearchDialect::Scopus),
        "web_of_science" | "web-of-science" => Ok(SearchDialect::WebOfScience),
        "crossref" => Ok(SearchDialect::Crossref),
        "openalex" => Ok(SearchDialect::OpenAlex),
        "clinicaltrials_gov" | "clinicaltrials-gov" => Ok(SearchDialect::ClinicalTrialsGov),
        "generic_boolean" | "generic-boolean" => Ok(SearchDialect::GenericBoolean),
        _ => Err(format!("unsupported dialect `{value}`")),
    }
}

fn parse_interchange(value: &str) -> Result<InterchangeFormat, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "searchright_json" | "searchright-json" => Ok(InterchangeFormat::SearchrightJson),
        "json_lines" | "json-lines" | "jsonl" => Ok(InterchangeFormat::JsonLines),
        "csl_json" | "csl-json" => Ok(InterchangeFormat::CslJson),
        "ris" => Ok(InterchangeFormat::Ris),
        "nbib" | "pubmed" => Ok(InterchangeFormat::Nbib),
        "csv" => Ok(InterchangeFormat::Csv),
        _ => Err(format!("unsupported interchange format `{value}`")),
    }
}

fn parse_content_policy(value: &str) -> Result<UntrustedContentPolicy, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "data_only" | "data-only" => Ok(UntrustedContentPolicy::DataOnly),
        "sanitise_then_data_only" | "sanitise-then-data-only" => {
            Ok(UntrustedContentPolicy::SanitiseThenDataOnly)
        }
        "human_inspection_required" | "human-inspection-required" => {
            Ok(UntrustedContentPolicy::HumanInspectionRequired)
        }
        _ => Err(format!("unsupported untrusted-content policy `{value}`")),
    }
}

fn validation_result<E: std::fmt::Display>(
    kind: &str,
    result: Result<(), E>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(()) => json_success(&serde_json::json!({"valid": true, "contract": kind})),
        Err(error) => Ok(tool_error(error.to_string())),
    }
}

fn operation_result<T: serde::Serialize>(
    result: Result<T, searchright::EngineError>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(value) => json_success(&value),
        Err(error) => Ok(tool_error(error.to_string())),
    }
}

fn json_success(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    serde_json::to_string_pretty(value)
        .map(|json| CallToolResult::success(vec![ContentBlock::text(json)]))
        .map_err(|error| McpError::internal_error(error.to_string(), None))
}

fn tool_error(message: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

fn json_invalid_params(error: serde_json::Error) -> McpError {
    invalid_params(error.to_string())
}

fn invalid_params(message: String) -> McpError {
    McpError::invalid_params(message, None)
}
