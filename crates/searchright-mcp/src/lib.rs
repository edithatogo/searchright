//! MCP server implementation for Searchright's governed review operations.
//!
//! The server maps MCP tools onto the shared [`SearchrightEngine`] while
//! retaining Searchright's contract validation and explicit operation inputs.

#![forbid(unsafe_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CacheScope, CallToolResponse, CallToolResult, CancelTaskParams, CompleteRequestParams,
        CompleteResult, CompletionInfo, ContentBlock, CreateTaskResult, ElicitRequest,
        ElicitRequestParams, ElicitationAction, GetPromptRequestParams, GetPromptResponse,
        GetPromptResult, GetTaskParams, GetTaskResult, InitializeRequestParams, InitializeResult,
        InputRequest, InputRequests, InputRequiredResult, JsonObject, ListPromptsResult,
        ListResourcesResult, ListToolsResult, PaginatedRequestParams, Prompt, PromptArgument,
        PromptMessage, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
        ReadResourceResult, Resource, ResourceContents, Role, ServerCapabilities, ServerInfo,
        SubscriptionFilter, ToolAnnotations, UpdateTaskParams,
    },
    schemars,
    service::{RequestContext, RoleServer, SubscriptionContext},
    task_manager::{TaskExit, TaskManager, TaskOptions},
    tool, tool_router,
    transport::stdio,
};
use searchright::contracts::{
    AuditEvent, BenchmarkReport, BibliographicRecord, CompiledStrategy, DataHandlingRequest,
    Diagnostic, DiscoveryRun, DocumentEvidence, ExecutionEnvelope, InstitutionalPolicy,
    InterchangeFormat, LicensedAdapterProfile, LivingUpdateRun, PrismaFlow, ProtocolAmendment,
    ProviderComponentManifest, RankingCalibration, ReviewPlan, SearchDialect, SearchStrategy,
    SearchValidationReport, SourceReceipt, StandardAssessment, StandardPack, StudyGraph,
    UntrustedContentPolicy, WorkflowTrace,
};
use searchright::dedup::DedupConfig;
use searchright::{PrismaArtifact, PrismaOutput, SearchrightEngine};
use serde::Deserialize;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};

const LOCAL_TASK_LIMIT: usize = 4;
const TASK_ACTIVITY_URI: &str = "searchright://runtime/task-activity";

/// One deterministic, local-only successful tool invocation for client conformance tests.
///
/// This fixture matrix is not a live-provider receipt and must never be used to
/// claim remote support. It keeps typed MCP clients on the same rights-clear
/// contract examples as the stdio conformance harness.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct McpToolSuccessCase {
    /// Advertised MCP tool name.
    pub tool_name: &'static str,
    /// Valid, deterministic arguments for the named tool.
    pub arguments: JsonObject,
}

/// Return a full local-only success matrix for every advertised MCP tool.
///
/// `generate_prisma` has two rows because its structured JSON and text-backed
/// Mermaid result branches use the same declared union output contract.
#[doc(hidden)]
pub fn live_client_success_cases() -> Result<Vec<McpToolSuccessCase>, String> {
    let review_plan = json_example(include_str!("../../../contracts/examples/review-plan.yaml"))?;
    let strategy = json_example(include_str!(
        "../../../contracts/examples/search-strategy.yaml"
    ))?;
    let document_evidence = json_example(include_str!(
        "../../../contracts/examples/document-evidence.json"
    ))?;
    let prisma_flow = include_str!("../../../contracts/examples/prisma-flow.json").to_owned();
    let record = json_example(include_str!(
        "../../../contracts/examples/bibliographic-record.yaml"
    ))?;
    let study_graph = json_example(include_str!("../../../contracts/examples/study-graph.yaml"))?;
    let search_validation = json_example(include_str!(
        "../../../contracts/examples/search-validation.yaml"
    ))?;
    let source_receipt = json_example_with_review_id(
        include_str!("../../../contracts/examples/source-receipt.yaml"),
        "demo-paediatric-metabolic-search",
    )?;
    let institutional_policy = json_example(include_str!(
        "../../../contracts/examples/institutional-policy.yaml"
    ))?;
    let handling_request = json_example(include_str!(
        "../../../contracts/examples/data-handling-request.yaml"
    ))?;
    let execution_envelope = json_example(include_str!(
        "../../../contracts/examples/execution-envelope.yaml"
    ))?;
    let amendment = json_example(include_str!(
        "../../../contracts/examples/protocol-amendment.yaml"
    ))?;
    let standard_pack = json_example(include_str!(
        "../../../contracts/examples/standard-pack.yaml"
    ))?;
    let standard_assessment = json_example(include_str!(
        "../../../contracts/examples/standard-assessment.yaml"
    ))?;
    let ranking_calibration = json_example(include_str!(
        "../../../contracts/examples/ranking-calibration.yaml"
    ))?;
    let discovery_run = json_example(include_str!(
        "../../../contracts/examples/discovery-run.yaml"
    ))?;
    let workflow_trace = json_example(include_str!(
        "../../../contracts/examples/workflow-trace.yaml"
    ))?;
    let licensed_adapter = json_example(include_str!(
        "../../../contracts/examples/licensed-adapter.yaml"
    ))?;
    let mut compiled_strategy = serde_yaml::from_str::<serde_json::Value>(include_str!(
        "../../../contracts/examples/compiled-strategy.yaml"
    ))
    .map_err(|error| format!("compiled strategy fixture must parse: {error}"))?;
    // The licensed adapter fixture is an Embase profile; keep the strategy
    // dialect aligned so this row exercises the successful no-network plan.
    let Some(compiled_strategy) = compiled_strategy.as_object_mut() else {
        return Err("compiled strategy fixture must be an object".to_owned());
    };
    compiled_strategy.insert(
        "dialect".to_owned(),
        serde_json::Value::String("embase".to_owned()),
    );
    let compiled_strategy = serde_json::to_string(&compiled_strategy)
        .map_err(|error| format!("compiled strategy fixture must serialize: {error}"))?;
    let benchmark_report = json_example(include_str!(
        "../../../contracts/examples/benchmark-report.yaml"
    ))?;
    let diagnostic = json_example(include_str!("../../../contracts/examples/diagnostic.yaml"))?;

    let component_bytes = b"searchright-mcp-conformance-component";
    let mut component_manifest = serde_yaml::from_str::<serde_json::Value>(include_str!(
        "../../../contracts/examples/provider-component.yaml"
    ))
    .map_err(|error| format!("provider component fixture must parse: {error}"))?;
    let Some(component_manifest) = component_manifest.as_object_mut() else {
        return Err("provider component fixture must be an object".to_owned());
    };
    component_manifest.insert(
        "component_digest".to_owned(),
        serde_json::Value::String(
            "2171682ec6aa9f6e7ddc24c7a49be6b08648bc0b63d10c7dfc6b910f8e2aea64".to_owned(),
        ),
    );
    let component_manifest = serde_json::to_string(&component_manifest)
        .map_err(|error| format!("provider component fixture must serialize: {error}"))?;

    let mut cases = vec![
        fixture(
            "validate_plan",
            [
                ("document", review_plan.clone()),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "validate_strategy",
            [
                ("document", strategy.clone()),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "validate_document_evidence",
            [
                ("document", document_evidence),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "compile_strategy",
            [
                ("document", strategy),
                ("format", "json".to_owned()),
                ("dialect", "pubmed".to_owned()),
            ],
        ),
        fixture(
            "deduplicate_records",
            [("records_json", format!("[{record}]"))],
        ),
        fixture(
            "generate_prisma",
            [
                ("flow_json", prisma_flow.clone()),
                ("output", "json".to_owned()),
            ],
        ),
        fixture(
            "generate_prisma",
            [("flow_json", prisma_flow), ("output", "mermaid".to_owned())],
        ),
        fixture("verify_audit", [("audit_jsonl", String::new())]),
        fixture(
            "import_records",
            [
                ("document", "[]".to_owned()),
                ("input_format", "searchright_json".to_owned()),
                ("source_receipt_id", "fixture-receipt".to_owned()),
            ],
        ),
        fixture(
            "export_records",
            [
                ("records_json", "[]".to_owned()),
                ("review_id", "fixture-review".to_owned()),
                ("input_format", "searchright_json".to_owned()),
                ("output_format", "searchright_json".to_owned()),
            ],
        ),
        fixture(
            "assess_study_graph",
            [("document", study_graph), ("format", "json".to_owned())],
        ),
        fixture(
            "assess_search_validation",
            [
                ("document", search_validation),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "living_diff",
            [
                ("previous_records_json", "[]".to_owned()),
                ("current_records_json", "[]".to_owned()),
            ],
        ),
        fixture(
            "validate_living_lineage",
            [
                // An empty, explicit lineage is the valid bounded baseline;
                // the single checked-in update references an earlier run that
                // is intentionally not bundled with this fixture matrix.
                ("document", "[]".to_owned()),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "build_provenance",
            [
                ("plan_json", review_plan),
                ("receipts_json", format!("[{source_receipt}]")),
                ("events_json", "[]".to_owned()),
            ],
        ),
        fixture(
            "rank_records",
            [
                ("records_json", "[]".to_owned()),
                ("query_terms", "[]".to_owned()),
            ],
        ),
        fixture(
            "inspect_untrusted_content",
            [
                ("subject_id", "fixture-record".to_owned()),
                ("text", "synthetic content".to_owned()),
                ("policy", "data_only".to_owned()),
            ],
        ),
        fixture(
            "render_diagnostics",
            [
                ("document", format!("[{diagnostic}]")),
                ("format", "json".to_owned()),
                ("output", "plain_text".to_owned()),
            ],
        ),
        fixture(
            "evaluate_governance",
            [
                ("policy_json", institutional_policy),
                ("request_json", handling_request),
            ],
        ),
        fixture(
            "authorise_endpoint",
            [
                ("envelope_json", execution_envelope),
                (
                    "endpoint",
                    "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi".to_owned(),
                ),
            ],
        ),
        fixture(
            "validate_amendment",
            [("document", amendment), ("format", "json".to_owned())],
        ),
        fixture(
            "validate_standard_pack",
            [("document", standard_pack), ("format", "json".to_owned())],
        ),
        fixture(
            "validate_standard_assessment",
            [
                ("document", standard_assessment),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "validate_ranking_calibration",
            [
                ("document", ranking_calibration),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "validate_discovery_run",
            [
                ("document", discovery_run.clone()),
                ("format", "json".to_owned()),
            ],
        ),
        fixture(
            "verify_workflow_trace",
            [("document", workflow_trace), ("format", "json".to_owned())],
        ),
        fixture(
            "discovery_candidates",
            [("document", discovery_run), ("format", "json".to_owned())],
        ),
        fixture(
            "verify_provider_component",
            [
                ("manifest_json", component_manifest),
                ("component_base64", BASE64_STANDARD.encode(component_bytes)),
            ],
        ),
        fixture(
            "plan_licensed_request",
            [
                ("profile_json", licensed_adapter),
                ("compiled_strategy_json", compiled_strategy),
                ("endpoint", "https://embase.com/search".to_owned()),
            ],
        ),
        fixture(
            "validate_benchmark_report",
            [
                ("document", benchmark_report),
                ("format", "json".to_owned()),
            ],
        ),
        fixture("list_providers", []),
        fixture("workflow", []),
    ];
    let rank_case = cases
        .iter_mut()
        .find(|case| case.tool_name == "rank_records")
        .ok_or_else(|| "rank fixture must be registered".to_owned())?;
    rank_case.arguments.insert(
        "query_terms".to_owned(),
        serde_json::Value::Array(Vec::new()),
    );
    Ok(cases)
}

fn json_example(document: &str) -> Result<String, String> {
    let value = serde_yaml::from_str::<serde_json::Value>(document)
        .map_err(|error| format!("MCP conformance fixture must parse: {error}"))?;
    serde_json::to_string(&value)
        .map_err(|error| format!("MCP conformance fixture must serialize: {error}"))
}

fn json_example_with_review_id(document: &str, review_id: &str) -> Result<String, String> {
    let mut value = serde_yaml::from_str::<serde_json::Value>(document)
        .map_err(|error| format!("MCP conformance fixture must parse: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "MCP conformance fixture must be an object".to_owned())?;
    object.insert(
        "review_id".to_owned(),
        serde_json::Value::String(review_id.to_owned()),
    );
    serde_json::to_string(&value)
        .map_err(|error| format!("MCP conformance fixture must serialize: {error}"))
}

fn fixture<const N: usize>(
    tool_name: &'static str,
    arguments: [(&str, String); N],
) -> McpToolSuccessCase {
    let arguments = arguments
        .into_iter()
        .map(|(name, value)| (name.to_owned(), serde_json::Value::String(value)))
        .collect();
    McpToolSuccessCase {
        tool_name,
        arguments,
    }
}

struct LocalTaskActivityLease {
    permit: Option<OwnedSemaphorePermit>,
    active_tasks: Arc<AtomicUsize>,
    activity: watch::Sender<u64>,
}

impl Drop for LocalTaskActivityLease {
    fn drop(&mut self) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
        self.activity.send_modify(|revision| *revision += 1);
        drop(self.permit.take());
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DocumentInput {
    /// JSON or YAML document text.
    document: String,
    /// `json` or `yaml`; defaults to JSON.
    format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CompileInput {
    /// JSON or YAML `SearchStrategy` document.
    document: String,
    /// `json` or `yaml`; defaults to JSON.
    format: Option<String>,
    /// Target dialect such as pubmed, embase or `europe_pmc`.
    dialect: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DedupInput {
    /// JSON array of `BibliographicRecord` contracts.
    records_json: String,
    /// Optional title-similarity threshold, zero to one.
    title_threshold: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrismaInput {
    /// JSON `PrismaFlow` contract.
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
    /// JSON array of canonical `BibliographicRecord` contracts.
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
    /// JSON `ReviewPlan` contract.
    plan_json: String,
    /// JSON array of `SourceReceipt` contracts.
    receipts_json: String,
    /// JSON array of `AuditEvent` contracts.
    events_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RankInput {
    /// JSON array of canonical `BibliographicRecord` contracts.
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
    /// JSON `InstitutionalPolicy` contract.
    policy_json: String,
    /// JSON `DataHandlingRequest` contract.
    request_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EndpointInput {
    /// JSON `ExecutionEnvelope` contract.
    envelope_json: String,
    /// Requested HTTPS endpoint.
    endpoint: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ComponentInput {
    /// JSON `ProviderComponentManifest` contract.
    manifest_json: String,
    /// Base64-encoded exact component bytes.
    component_base64: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LicensedPlanInput {
    /// JSON `LicensedAdapterProfile` contract.
    profile_json: String,
    /// JSON `CompiledStrategy` contract.
    compiled_strategy_json: String,
    /// Redacted HTTPS endpoint to authorise.
    endpoint: String,
}

#[derive(Clone)]
/// Governed Searchright MCP tool server shared by local and remote adapters.
pub struct SearchrightServer {
    tool_router: ToolRouter<Self>,
    tasks: TaskManager,
    task_slots: Arc<Semaphore>,
    active_tasks: Arc<AtomicUsize>,
    task_activity: watch::Sender<u64>,
    remote_http: bool,
}

impl Default for SearchrightServer {
    fn default() -> Self {
        let (task_activity, _) = watch::channel(0);
        Self {
            tool_router: Self::governed_tool_router(),
            tasks: TaskManager::new(),
            task_slots: Arc::new(Semaphore::new(LOCAL_TASK_LIMIT)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            task_activity,
            remote_http: false,
        }
    }
}

impl SearchrightServer {
    /// Create the server variant used by authenticated Streamable HTTP.
    #[must_use]
    pub fn remote_http() -> Self {
        let (task_activity, _) = watch::channel(0);
        Self {
            tool_router: Self::governed_tool_router(),
            tasks: TaskManager::new(),
            task_slots: Arc::new(Semaphore::new(LOCAL_TASK_LIMIT)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            task_activity,
            remote_http: true,
        }
    }

    fn require_local_current(&self, context: &RequestContext<RoleServer>) -> Result<(), McpError> {
        self.require_local()?;
        if context.protocol_version() != Some(ProtocolVersion::V_2026_07_28) {
            return Err(McpError::invalid_params(
                "tasks are available only to local 2026-07-28 clients",
                None,
            ));
        }
        Ok(())
    }

    fn require_local(&self) -> Result<(), McpError> {
        if self.remote_http {
            return Err(McpError::invalid_params(
                "advanced capabilities are available only over local stdio",
                None,
            ));
        }
        Ok(())
    }
}

#[tool_router]
impl SearchrightServer {
    fn governed_tool_router() -> ToolRouter<Self> {
        let mut router = Self::tool_router();
        for route in router.map.values_mut() {
            route.attr.output_schema = Some(output_schema_for(&route.attr.name));
            route.attr.annotations = Some(
                ToolAnnotations::new()
                    .read_only(true)
                    .destructive(false)
                    .idempotent(true)
                    .open_world(false),
            );
        }
        router
    }

    #[tool(
        description = "Read-only: validate a review plan and return conservative readiness findings"
    )]
    fn validate_plan(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let plan: ReviewPlan =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result("validate_plan", SearchrightEngine::validate_plan(&plan))
    }

    #[tool(description = "Read-only: validate a source-specific search strategy without execution")]
    fn validate_strategy(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let strategy: SearchStrategy =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_strategy",
            "search_strategy",
            SearchrightEngine::validate_strategy(&strategy),
        )
    }

    #[tool(
        description = "Read-only: validate neutral CiteWeft-compatible document evidence; canonical writes are forbidden"
    )]
    fn validate_document_evidence(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let evidence: DocumentEvidence =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_document_evidence",
            "document_evidence",
            SearchrightEngine::validate_document_evidence(&evidence),
        )
    }

    #[tool(
        description = "Read-only: compile a portable query AST and expose fidelity, loss codes and the human-review gate"
    )]
    fn compile_strategy(
        &self,
        Parameters(input): Parameters<CompileInput>,
    ) -> Result<CallToolResult, McpError> {
        let strategy: SearchStrategy =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        let dialect = parse_dialect(&input.dialect).map_err(invalid_params)?;
        operation_result(
            "compile_strategy",
            SearchrightEngine::compile_strategy(&strategy, dialect),
        )
    }

    #[tool(
        description = "Read-only preview: generate reviewable duplicate clusters without deleting records"
    )]
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
        operation_result(
            "deduplicate_records",
            SearchrightEngine::deduplicate(
                &records,
                DedupConfig {
                    title_similarity_threshold: threshold,
                    ..DedupConfig::default()
                },
            ),
        )
    }

    #[tool(
        description = "Read-only: validate PRISMA arithmetic and render JSON, Mermaid or the PRISMA-S ledger"
    )]
    fn generate_prisma(
        &self,
        Parameters(input): Parameters<PrismaInput>,
    ) -> Result<CallToolResult, McpError> {
        let flow: PrismaFlow =
            serde_json::from_str(&input.flow_json).map_err(json_invalid_params)?;
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
                text_success("generate_prisma", "mermaid", document)
            }
            Ok(artifact) => json_success("generate_prisma", &artifact),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only: verify a hash-chained Searchright audit JSONL stream")]
    fn verify_audit(
        &self,
        Parameters(input): Parameters<AuditInput>,
    ) -> Result<CallToolResult, McpError> {
        let events = parse_jsonl::<AuditEvent>(&input.audit_jsonl).map_err(invalid_params)?;
        operation_result("verify_audit", SearchrightEngine::verify_audit(events))
    }

    #[tool(
        description = "Read-only import: normalise a bibliographic interchange document while retaining provenance"
    )]
    fn import_records(
        &self,
        Parameters(input): Parameters<ImportInput>,
    ) -> Result<CallToolResult, McpError> {
        let format = parse_interchange(&input.input_format).map_err(invalid_params)?;
        operation_result(
            "import_records",
            SearchrightEngine::import_records(&input.document, format, &input.source_receipt_id),
        )
    }

    #[tool(
        description = "Read-only conversion: export canonical records and return a deterministic conversion receipt"
    )]
    fn export_records(
        &self,
        Parameters(input): Parameters<ExportInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(json_invalid_params)?;
        let input_format = parse_interchange(&input.input_format).map_err(invalid_params)?;
        let output_format = parse_interchange(&input.output_format).map_err(invalid_params)?;
        operation_result(
            "export_records",
            SearchrightEngine::export_records(
                &input.review_id,
                &records,
                input_format,
                output_format,
            ),
        )
    }

    #[tool(description = "Read-only: validate and summarise a record-report-study/full-text graph")]
    fn assess_study_graph(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let graph: StudyGraph =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result(
            "assess_study_graph",
            SearchrightEngine::assess_study_graph(&graph),
        )
    }

    #[tool(
        description = "Read-only gate: assess PRESS findings, seed recall and translation-loss approval"
    )]
    fn assess_search_validation(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let report: SearchValidationReport =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result(
            "assess_search_validation",
            SearchrightEngine::assess_search_validation(&report),
        )
    }

    #[tool(
        description = "Read-only: compare parent and current records for a living-review update"
    )]
    fn living_diff(
        &self,
        Parameters(input): Parameters<LivingDiffInput>,
    ) -> Result<CallToolResult, McpError> {
        let previous: Vec<BibliographicRecord> =
            serde_json::from_str(&input.previous_records_json).map_err(json_invalid_params)?;
        let current: Vec<BibliographicRecord> =
            serde_json::from_str(&input.current_records_json).map_err(json_invalid_params)?;
        operation_result(
            "living_diff",
            SearchrightEngine::diff_living_records(&previous, &current),
        )
    }

    #[tool(description = "Read-only: validate immutable living-review run lineage")]
    fn validate_living_lineage(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let runs: Vec<LivingUpdateRun> =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_living_lineage",
            "living_lineage",
            SearchrightEngine::validate_living_lineage(&runs),
        )
    }

    #[tool(description = "Read-only: build RO-Crate 1.3 and W3C PROV-compatible review provenance")]
    fn build_provenance(
        &self,
        Parameters(input): Parameters<ProvenanceInput>,
    ) -> Result<CallToolResult, McpError> {
        let plan: ReviewPlan =
            serde_json::from_str(&input.plan_json).map_err(json_invalid_params)?;
        let receipts: Vec<SourceReceipt> =
            serde_json::from_str(&input.receipts_json).map_err(json_invalid_params)?;
        let events: Vec<AuditEvent> =
            serde_json::from_str(&input.events_json).map_err(json_invalid_params)?;
        operation_result(
            "build_provenance",
            SearchrightEngine::provenance(&plan, &receipts, &events),
        )
    }

    #[tool(
        description = "Read-only advisory ranking: prioritise records transparently; never make final exclusions"
    )]
    fn rank_records(
        &self,
        Parameters(input): Parameters<RankInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(json_invalid_params)?;
        operation_result(
            "rank_records",
            SearchrightEngine::rank_records(&records, &input.query_terms),
        )
    }

    #[tool(
        description = "Read-only safety check: treat provider/full-text content as inert data and flag instruction-like markers"
    )]
    fn inspect_untrusted_content(
        &self,
        Parameters(input): Parameters<ContentInput>,
    ) -> Result<CallToolResult, McpError> {
        let policy = parse_content_policy(input.policy.as_deref().unwrap_or("data_only"))
            .map_err(invalid_params)?;
        json_success(
            "inspect_untrusted_content",
            &SearchrightEngine::inspect_content(&input.subject_id, &input.text, policy),
        )
    }

    #[tool(
        description = "Read-only accessibility surface: validate and render stable diagnostics without ANSI-dependent output"
    )]
    fn render_diagnostics(
        &self,
        Parameters(input): Parameters<DiagnosticsInput>,
    ) -> Result<CallToolResult, McpError> {
        let diagnostics: Vec<Diagnostic> =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
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
            Ok(document) => text_success("render_diagnostics", &input.output, document),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(
        description = "Read-only governance gate: evaluate a data-handling request against an institutional policy"
    )]
    fn evaluate_governance(
        &self,
        Parameters(input): Parameters<GovernanceInput>,
    ) -> Result<CallToolResult, McpError> {
        let policy: InstitutionalPolicy =
            serde_json::from_str(&input.policy_json).map_err(json_invalid_params)?;
        let request: DataHandlingRequest =
            serde_json::from_str(&input.request_json).map_err(json_invalid_params)?;
        operation_result(
            "evaluate_governance",
            SearchrightEngine::evaluate_governance(&policy, &request),
        )
    }

    #[tool(
        description = "Read-only policy gate: verify that an HTTPS endpoint is permitted by an execution envelope"
    )]
    fn authorise_endpoint(
        &self,
        Parameters(input): Parameters<EndpointInput>,
    ) -> Result<CallToolResult, McpError> {
        let envelope: ExecutionEnvelope =
            serde_json::from_str(&input.envelope_json).map_err(json_invalid_params)?;
        match SearchrightEngine::authorise_endpoint(&envelope, &input.endpoint) {
            Ok(()) => json_success(
                "authorise_endpoint",
                &serde_json::json!({
                    "authorised": true,
                    "endpoint": input.endpoint,
                }),
            ),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Read-only: validate a versioned protocol amendment")]
    fn validate_amendment(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: ProtocolAmendment =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_amendment",
            "protocol_amendment",
            SearchrightEngine::validate_amendment(&value),
        )
    }

    #[tool(description = "Read-only: validate a methodological standards pack")]
    fn validate_standard_pack(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: StandardPack =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_standard_pack",
            "standard_pack",
            SearchrightEngine::validate_standard_pack(&value),
        )
    }

    #[tool(description = "Read-only: validate an evidence-linked standards assessment")]
    fn validate_standard_assessment(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: StandardAssessment =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_standard_assessment",
            "standard_assessment",
            SearchrightEngine::validate_standard_assessment(&value),
        )
    }

    #[tool(
        description = "Read-only: validate ranking calibration and the no-auto-exclusion invariant"
    )]
    fn validate_ranking_calibration(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: RankingCalibration =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_ranking_calibration",
            "ranking_calibration",
            SearchrightEngine::validate_ranking_calibration(&value),
        )
    }

    #[tool(description = "Read-only: validate one bounded supplementary-discovery run")]
    fn validate_discovery_run(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let value: DiscoveryRun =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        validation_result(
            "validate_discovery_run",
            "discovery_run",
            SearchrightEngine::validate_discovery_run(&value),
        )
    }

    #[tool(
        description = "Read-only assurance: verify lifecycle continuity, declared transitions and human approval gates"
    )]
    fn verify_workflow_trace(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let trace: WorkflowTrace =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result(
            "verify_workflow_trace",
            SearchrightEngine::verify_workflow_trace(&trace),
        )
    }

    #[tool(
        description = "Read-only discovery: resolve bounded citation/grey-literature candidates pending human release"
    )]
    fn discovery_candidates(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let run: DiscoveryRun =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        operation_result(
            "discovery_candidates",
            SearchrightEngine::discovery_candidates(&run),
        )
    }

    #[tool(
        description = "Read-only supply-chain gate: verify a provider manifest against exact base64 component bytes"
    )]
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
            Ok(()) => json_success(
                "verify_provider_component",
                &serde_json::json!({
                    "valid": true,
                    "component_id": manifest.component_id,
                    "bytes": bytes.len(),
                }),
            ),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(
        description = "Read-only licensed-source gate: build a redacted BYO-access request plan without exposing credentials"
    )]
    fn plan_licensed_request(
        &self,
        Parameters(input): Parameters<LicensedPlanInput>,
    ) -> Result<CallToolResult, McpError> {
        let profile: LicensedAdapterProfile =
            serde_json::from_str(&input.profile_json).map_err(json_invalid_params)?;
        let strategy: CompiledStrategy =
            serde_json::from_str(&input.compiled_strategy_json).map_err(json_invalid_params)?;
        operation_result(
            "plan_licensed_request",
            SearchrightEngine::plan_licensed_request(&profile, &strategy, &input.endpoint),
        )
    }

    #[tool(
        description = "Read-only evidence gate: validate a benchmark report and its explicit claim boundary"
    )]
    fn validate_benchmark_report(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let report: BenchmarkReport =
            parse_document(&input.document, input.format.as_deref()).map_err(invalid_params)?;
        match SearchrightEngine::validate_benchmark_report(&report) {
            Ok(()) => json_success(
                "validate_benchmark_report",
                &serde_json::json!({
                    "valid": true,
                    "benchmark_id": report.benchmark_id,
                }),
            ),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(
        description = "Read-only: list deterministic no-network provider manifests available by default"
    )]
    fn list_providers(&self) -> Result<CallToolResult, McpError> {
        operation_result(
            "list_providers",
            SearchrightEngine::default_provider_manifests(),
        )
    }

    #[tool(
        description = "Read-only: return the conservative planning, execution, screening, reporting and update workflow"
    )]
    fn workflow(&self) -> Result<CallToolResult, McpError> {
        json_success("workflow", &SearchrightEngine::workflow())
    }
}

impl ServerHandler for SearchrightServer {
    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        context.peer.set_peer_info(request.clone());
        let mut info = self.get_info();
        let supported = self.supported_protocol_versions();
        info.protocol_version = if supported.contains(&request.protocol_version) {
            request.protocol_version
        } else {
            info.protocol_version
        };
        if info.protocol_version != ProtocolVersion::V_2026_07_28 {
            info.capabilities = ServerCapabilities::builder()
                .enable_completions()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build();
        }
        Ok(info)
    }

    fn get_info(&self) -> ServerInfo {
        let capabilities = if self.remote_http {
            ServerCapabilities::builder().enable_tools().build()
        } else {
            ServerCapabilities::builder()
                .enable_completions()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_resources_subscribe()
                .enable_tasks()
                .build()
        };
        ServerInfo::new(capabilities)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        if self.remote_http {
            Cow::Borrowed(&[ProtocolVersion::V_2026_07_28])
        } else {
            Cow::Borrowed(&[ProtocolVersion::V_2025_11_25, ProtocolVersion::V_2026_07_28])
        }
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        if request.and_then(|params| params.cursor).is_some() {
            return Err(McpError::invalid_params(
                "tools/list does not accept cursors",
                None,
            ));
        }
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all())
            .with_ttl_ms(60_000)
            .with_cache_scope(CacheScope::Public))
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        self.require_local()?;
        if request
            .context
            .as_ref()
            .is_some_and(rmcp::model::CompletionContext::has_arguments)
        {
            return Err(McpError::invalid_params(
                "completion context is not supported",
                None,
            ));
        }
        if request.argument.value.len() > 64 || !request.argument.value.is_ascii() {
            return Err(McpError::invalid_params(
                "completion prefix is not bounded ASCII",
                None,
            ));
        }
        let prompt = request
            .r#ref
            .as_prompt_name()
            .ok_or_else(|| McpError::invalid_params("only prompt completion is supported", None))?;
        let candidates: &[&str] = match (prompt, request.argument.name.as_str()) {
            ("plan-review", "mode") => &["evidence-gaps", "findings-only"],
            ("press-check", "focus") => &["line-by-line", "translation-loss"],
            _ => return Err(McpError::invalid_params("unknown completion target", None)),
        };
        let values = candidates
            .iter()
            .filter(|value| value.starts_with(&request.argument.value))
            .map(|value| (*value).to_owned())
            .collect();
        let completion = CompletionInfo::with_all_values(values)
            .map_err(|_| McpError::internal_error("completion bound failed", None))?;
        Ok(CompleteResult::new(completion))
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.tool_router.get(name).cloned()
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        if !self.remote_http
            && context.protocol_version() == Some(ProtocolVersion::V_2026_07_28)
            && request.name == "workflow"
            && context
                .client_capabilities()
                .is_some_and(|capabilities| capabilities.supports_tasks())
        {
            let permit = self.task_slots.clone().try_acquire_owned().map_err(|_| {
                McpError::invalid_request("local task concurrency limit reached", None)
            })?;
            self.active_tasks.fetch_add(1, Ordering::SeqCst);
            self.task_activity.send_modify(|revision| *revision += 1);
            let activity_lease = LocalTaskActivityLease {
                permit: Some(permit),
                active_tasks: self.active_tasks.clone(),
                activity: self.task_activity.clone(),
            };
            let task = self.tasks.spawn(
                TaskOptions::new()
                    .with_poll_interval_ms(10)
                    .with_ttl_ms(60_000),
                move |task_context| {
                    Box::pin(async move {
                        let result = tokio::select! {
                            () = task_context.cancelled() => Err(TaskExit::Cancelled),
                            () = tokio::time::sleep(Duration::from_secs(1)) => {
                                json_success("workflow", &SearchrightEngine::workflow())
                                    .map_err(TaskExit::Error)
                            }
                        };
                        drop(activity_lease);
                        result
                    })
                },
            );
            return Ok(CallToolResponse::Task(CreateTaskResult::new(task)));
        }
        let call = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        self.tool_router.call(call).await
    }

    async fn get_task(
        &self,
        request: GetTaskParams,
        context: RequestContext<RoleServer>,
    ) -> Result<GetTaskResult, McpError> {
        self.require_local_current(&context)?;
        Ok(GetTaskResult::new(self.tasks.get_task(&request.task_id)?))
    }

    async fn update_task(
        &self,
        request: UpdateTaskParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        self.require_local_current(&context)?;
        self.tasks
            .update_task(&request.task_id, request.input_responses)
    }

    async fn cancel_task(
        &self,
        request: CancelTaskParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        self.require_local_current(&context)?;
        self.tasks.cancel_task(&request.task_id)
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        self.require_local()?;
        let (resources, next_cursor) = match request.and_then(|params| params.cursor).as_deref() {
            None => {
                let [workflow, _, _] = advanced_resources();
                (vec![workflow], Some("resources:2".to_owned()))
            }
            Some("resources:2") => {
                let [_, claim_boundary, task_activity] = advanced_resources();
                (vec![claim_boundary, task_activity], None)
            }
            Some(_) => return Err(McpError::invalid_params("unknown resource cursor", None)),
        };
        let mut result = ListResourcesResult::with_all_items(resources)
            .with_ttl_ms(60_000)
            .with_cache_scope(CacheScope::Public);
        result.next_cursor = next_cursor;
        Ok(result)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        self.require_local()?;
        if request.uri == TASK_ACTIVITY_URI {
            let document = serde_json::json!({
                "activity_sequence": *self.task_activity.borrow(),
                "active_tasks": self.active_tasks.load(Ordering::SeqCst),
                "task_limit": LOCAL_TASK_LIMIT,
                "claim_boundary": "aggregate local-process activity only; no task identity, durability, tenant, remote, or production-scale claim"
            });
            return Ok(ReadResourceResult::new(vec![
                ResourceContents::text(
                    serde_json::to_string_pretty(&document).map_err(|_| {
                        McpError::internal_error("task activity serialization failed", None)
                    })?,
                    request.uri,
                )
                .with_mime_type("application/json"),
            ])
            .into());
        }
        read_advanced_resource(request, &context)
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.require_local()?;
        let (prompts, next_cursor) = match request.and_then(|params| params.cursor).as_deref() {
            None => {
                let [plan_review, _] = advanced_prompts();
                (vec![plan_review], Some("prompts:2".to_owned()))
            }
            Some("prompts:2") => {
                let [_, press_check] = advanced_prompts();
                (vec![press_check], None)
            }
            Some(_) => return Err(McpError::invalid_params("unknown prompt cursor", None)),
        };
        let mut result = ListPromptsResult::with_all_items(prompts)
            .with_ttl_ms(60_000)
            .with_cache_scope(CacheScope::Public);
        result.next_cursor = next_cursor;
        Ok(result)
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, McpError> {
        self.require_local()?;
        let review_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("review_id"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<review-id>");
        if review_id != "<review-id>"
            && (review_id.is_empty()
                || review_id.len() > 64
                || !review_id.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "-_.".contains(character)
                }))
        {
            return Err(McpError::invalid_params(
                "review_id must be a bounded identifier",
                None,
            ));
        }
        let quoted_review_id = serde_json::to_string(review_id)
            .map_err(|_| McpError::internal_error("review_id serialization failed", None))?;
        let text = match request.name.as_str() {
            "plan-review" => {
                let mode =
                    prompt_enum_argument(&request, "mode", &["findings-only", "evidence-gaps"])?
                        .unwrap_or("findings-only");
                format!(
                    "Review Searchright plan identified by {quoted_review_id} in {mode} mode. Return findings only; do not change eligibility criteria or grant execution authority."
                )
            }
            "press-check" => {
                let focus =
                    prompt_enum_argument(&request, "focus", &["line-by-line", "translation-loss"])?
                        .unwrap_or("line-by-line");
                format!(
                    "Assess the search strategy identified by {quoted_review_id} with {focus} focus using recorded PRESS evidence. Do not certify completeness or silently rewrite native syntax."
                )
            }
            _ => return Err(McpError::invalid_params("unknown prompt", None)),
        };
        Ok(GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)]).into())
    }

    fn accepted_subscription_filter(
        &self,
        requested: &SubscriptionFilter,
    ) -> Option<SubscriptionFilter> {
        if self.remote_http {
            return None;
        }
        let accepted = requested
            .resource_subscriptions
            .as_ref()
            .map(|uris| {
                uris.iter()
                    .filter(|uri| uri.as_str() == TASK_ACTIVITY_URI)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .filter(|uris| !uris.is_empty());
        accepted.map(|uris| {
            SubscriptionFilter::builder()
                .resource_subscriptions(uris)
                .build()
        })
    }

    async fn listen(&self, context: SubscriptionContext) -> Result<(), McpError> {
        self.require_local()?;
        let mut activity = self.task_activity.subscribe();
        loop {
            tokio::select! {
                () = context.cancelled() => return Ok(()),
                changed = activity.changed() => {
                    changed.map_err(|_| McpError::internal_error("task activity source closed", None))?;
                    context
                        .sink()
                        .notify_resource_updated(TASK_ACTIVITY_URI)
                        .await
                        .map_err(|_| McpError::internal_error("task activity notification failed", None))?;
                }
            }
        }
    }
}

fn prompt_enum_argument<'a>(
    request: &'a GetPromptRequestParams,
    name: &str,
    allowed: &[&str],
) -> Result<Option<&'a str>, McpError> {
    let Some(value) = request
        .arguments
        .as_ref()
        .and_then(|arguments| arguments.get(name))
    else {
        return Ok(None);
    };
    let value = value
        .as_str()
        .ok_or_else(|| McpError::invalid_params("prompt option must be a string", None))?;
    if !allowed.contains(&value) {
        return Err(McpError::invalid_params("unsupported prompt option", None));
    }
    Ok(Some(value))
}

fn advanced_resources() -> [Resource; 3] {
    [
        Resource::new("searchright://workflow", "workflow")
            .with_title("Governed workflow")
            .with_description("Static, network-free Searchright workflow contract")
            .with_mime_type("application/json"),
        Resource::new("searchright://claim-boundary", "claim-boundary")
            .with_title("MCP claim boundary")
            .with_description("Explicit authority and evidence limitations")
            .with_mime_type("text/plain"),
        Resource::new(TASK_ACTIVITY_URI, "task-activity")
            .with_title("Aggregate local task activity")
            .with_description("Bounded aggregate task activity without identifiers or payloads")
            .with_mime_type("application/json"),
    ]
}

fn advanced_prompts() -> [Prompt; 2] {
    let review_id = PromptArgument::new("review_id")
        .with_description("Review identifier; never a grant of authority")
        .with_required(false);
    [
        Prompt::new(
            "plan-review",
            Some("Review a plan without changing it"),
            Some(vec![
                review_id.clone(),
                PromptArgument::new("mode")
                    .with_description("findings-only or evidence-gaps")
                    .with_required(false),
            ]),
        ),
        Prompt::new(
            "press-check",
            Some("Assess recorded PRESS evidence without certifying completeness"),
            Some(vec![
                review_id,
                PromptArgument::new("focus")
                    .with_description("line-by-line or translation-loss")
                    .with_required(false),
            ]),
        ),
    ]
}

fn read_advanced_resource(
    request: ReadResourceRequestParams,
    context: &RequestContext<RoleServer>,
) -> Result<ReadResourceResponse, McpError> {
    match request.uri.as_str() {
        "searchright://workflow" => Ok(ReadResourceResult::new(vec![
            ResourceContents::text(
                serde_json::to_string_pretty(&SearchrightEngine::workflow())
                    .map_err(|_| McpError::internal_error("workflow serialization failed", None))?,
                request.uri,
            )
            .with_mime_type("application/json"),
        ])
        .into()),
        "searchright://claim-boundary" if request.request_state.is_none() => {
            let supports_form_elicitation =
                context.client_capabilities().is_some_and(|capabilities| {
                    capabilities
                        .elicitation
                        .is_some_and(|elicitation| elicitation.form.is_some())
                });
            if context.protocol_version() != Some(ProtocolVersion::V_2026_07_28)
                || !supports_form_elicitation
            {
                return Err(McpError::invalid_params(
                    "claim-boundary acknowledgement requires local current form elicitation",
                    None,
                ));
            }
            let requested_schema = serde_json::from_value(serde_json::json!({
                "type": "object",
                "properties": {"acknowledged": {"type": "boolean", "const": true}},
                "required": ["acknowledged"],
                "additionalProperties": false
            }))
            .map_err(|_| McpError::internal_error("elicitation schema failed", None))?;
            let elicitation = InputRequest::Elicitation(ElicitRequest::new(
                ElicitRequestParams::FormElicitationParams {
                    meta: None,
                    message: "Acknowledge that this resource is read-only and grants no execution, methodological, screening, or release authority.".to_owned(),
                    requested_schema,
                },
            ));
            let mut requests = InputRequests::new();
            requests.insert("acknowledgement".to_owned(), elicitation);
            Ok(
                InputRequiredResult::new(Some(requests), Some("claim-boundary-form-v1".to_owned()))
                    .into(),
            )
        }
        "searchright://claim-boundary" => {
            if request.request_state.as_deref() != Some("claim-boundary-form-v1")
                || !acknowledgement_is_accepted(request.input_responses.as_ref())
            {
                return Err(McpError::invalid_params(
                    "claim-boundary acknowledgement is absent or invalid",
                    None,
                ));
            }
            Ok(ReadResourceResult::new(vec![ResourceContents::text(
                "Read-only local evidence. No live-provider, remote-hosting, methodological-certification, external-write, or final-screening authority is implied.",
                request.uri,
            )])
            .into())
        }
        _ => Err(McpError::invalid_params("unknown resource URI", None)),
    }
}

fn acknowledgement_is_accepted(responses: Option<&rmcp::model::InputResponses>) -> bool {
    let Some(responses) = responses else {
        return false;
    };
    if responses.len() != 1 {
        return false;
    }
    let Some(response) = responses.get("acknowledgement") else {
        return false;
    };
    serde_json::from_value::<rmcp::model::ElicitResult>(response.clone()).is_ok_and(|result| {
        result.action == ElicitationAction::Accept
            && result.content.as_ref().is_some_and(|content| {
                content.get("acknowledged") == Some(&serde_json::Value::Bool(true))
                    && content.as_object().is_some_and(|object| object.len() == 1)
            })
    })
}

/// Serve the governed MCP tools over standard input/output.
pub async fn run_stdio() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    let service = SearchrightServer::default().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(feature = "remote-http")]
pub mod remote;

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
    tool_name: &str,
    kind: &str,
    result: Result<(), E>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(()) => json_success(
            tool_name,
            &serde_json::json!({"valid": true, "contract": kind}),
        ),
        Err(error) => Ok(tool_error(error.to_string())),
    }
}

fn operation_result<T: serde::Serialize>(
    tool_name: &str,
    result: Result<T, searchright::EngineError>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(value) => json_success(tool_name, &value),
        Err(error) => Ok(tool_error(error.to_string())),
    }
}

fn json_success(
    tool_name: &str,
    value: &impl serde::Serialize,
) -> Result<CallToolResult, McpError> {
    serde_json::to_value(value)
        .map_err(|_| McpError::internal_error("MCP output serialization failed", None))
        .and_then(|value| {
            validate_tool_output(tool_name, &value).map_err(|()| {
                McpError::internal_error("MCP output contract validation failed", None)
            })?;
            Ok(CallToolResult::structured(value))
        })
}

fn text_success(
    tool_name: &str,
    format: &str,
    document: String,
) -> Result<CallToolResult, McpError> {
    let mut result = CallToolResult::structured(serde_json::json!({
        "document": document,
        "format": format,
    }));
    let Some(structured_content) = result.structured_content.as_ref() else {
        return Err(McpError::internal_error(
            "MCP text output has no structured content",
            None,
        ));
    };
    validate_tool_output(tool_name, structured_content)
        .map_err(|()| McpError::internal_error("MCP output contract validation failed", None))?;
    result.content = vec![ContentBlock::text(document)];
    Ok(result)
}

fn output_schema_for(tool_name: &str) -> Arc<JsonObject> {
    output_schema_registry()
        .schemas
        .get(tool_name)
        .cloned()
        .unwrap_or_else(|| panic!("every MCP tool must have an output contract: {tool_name}"))
}

struct OutputSchemaRegistry {
    schemas: BTreeMap<String, Arc<JsonObject>>,
}

fn output_schema_registry() -> &'static OutputSchemaRegistry {
    static REGISTRY: OnceLock<OutputSchemaRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let catalogue: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/interface-catalog.json"))
                .unwrap_or_else(|error| {
                    panic!("canonical interface catalogue must parse: {error}")
                });
        let entries = catalogue
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("canonical interface catalogue must contain entries"));

        let mut schemas = BTreeMap::new();
        for entry in entries {
            let Some(tool_name) = entry.get("mcp_tool").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let contract = entry.get("output_contract").unwrap_or_else(|| {
                panic!("every MCP tool must have an output contract: {tool_name}")
            });
            let schema = output_schema_from_contract(contract, tool_name);
            let serde_json::Value::Object(schema) = schema else {
                panic!("MCP output schema must be an object: {tool_name}")
            };
            assert!(
                schemas
                    .insert(tool_name.to_owned(), Arc::new(schema))
                    .is_none(),
                "MCP output contract must not be duplicated: {tool_name}"
            );
        }
        assert!(
            !schemas.is_empty(),
            "canonical interface catalogue must register MCP output schemas"
        );
        OutputSchemaRegistry { schemas }
    })
}

fn output_schema_from_contract(contract: &serde_json::Value, tool_name: &str) -> serde_json::Value {
    let root = contract
        .get("root")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("MCP output contract must name a root: {tool_name}"));

    let schema = if let Some(path) = contract.get("schema").and_then(serde_json::Value::as_str) {
        referenced_output_schema(path)
    } else if root == "array" {
        let items = if let Some(path) = contract
            .get("items_schema")
            .and_then(serde_json::Value::as_str)
        {
            referenced_output_schema(path)
        } else {
            object_schema(
                contract.get("items_fields"),
                contract.get("items_required"),
                None,
            )
        };
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "array",
            "items": items,
        })
    } else {
        object_schema(
            contract.get("fields"),
            contract.get("required"),
            contract.get("oneOf"),
        )
    };
    let serde_json::Value::Object(schema) = schema else {
        panic!("MCP output schema must be an object: {tool_name}")
    };
    serde_json::Value::Object(schema)
}

fn object_schema(
    fields: Option<&serde_json::Value>,
    required: Option<&serde_json::Value>,
    one_of: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "properties": fields.cloned().unwrap_or_else(|| serde_json::json!({})),
        "required": required.cloned().unwrap_or_else(|| serde_json::json!([])),
    });
    if let Some(one_of) = one_of
        && let Some(object) = schema.as_object_mut()
    {
        object.insert("oneOf".to_owned(), one_of.clone());
    }
    schema
}

fn validate_tool_output(tool_name: &str, value: &serde_json::Value) -> Result<(), ()> {
    let schema = output_schema_registry().schemas.get(tool_name).ok_or(())?;
    validate_json_schema(&serde_json::Value::Object((**schema).clone()), value)
}

/// Validate the closed JSON Schema vocabulary used by the MCP output registry.
///
/// The registry deliberately accepts only the keywords present in the checked-in
/// output contracts. A future contract that needs wider JSON Schema semantics
/// must add that support here before a successful tool result can be emitted.
fn validate_json_schema(schema: &serde_json::Value, value: &serde_json::Value) -> Result<(), ()> {
    const SUPPORTED_KEYWORDS: &[&str] = &[
        "$schema",
        "$id",
        "title",
        "type",
        "additionalProperties",
        "required",
        "properties",
        "oneOf",
        "const",
        "enum",
        "items",
        "minLength",
        "pattern",
        "uniqueItems",
        "minimum",
        "minItems",
    ];

    let object = schema.as_object().ok_or(())?;
    if object
        .keys()
        .any(|keyword| !SUPPORTED_KEYWORDS.contains(&keyword.as_str()))
    {
        return Err(());
    }

    if let Some(expected) = object.get("const")
        && value != expected
    {
        return Err(());
    }
    if let Some(values) = object.get("enum").and_then(serde_json::Value::as_array)
        && !values.contains(value)
    {
        return Err(());
    }
    if let Some(one_of) = object.get("oneOf").and_then(serde_json::Value::as_array)
        && one_of
            .iter()
            .filter(|candidate| validate_json_schema(candidate, value).is_ok())
            .count()
            != 1
    {
        return Err(());
    }
    if object
        .get("type")
        .is_some_and(|value_type| !json_value_matches_declared_type(value, value_type))
    {
        return Err(());
    }

    match value {
        serde_json::Value::String(text) => validate_string_schema(object, text)?,
        serde_json::Value::Array(values) => validate_array_schema(object, values)?,
        serde_json::Value::Object(values) => validate_object_schema(object, values)?,
        serde_json::Value::Number(number) => validate_number_schema(object, number)?,
        serde_json::Value::Null | serde_json::Value::Bool(_) => {}
    }
    Ok(())
}

fn json_value_matches_declared_type(
    value: &serde_json::Value,
    value_type: &serde_json::Value,
) -> bool {
    match value_type {
        serde_json::Value::String(value_type) => json_value_has_type(value, value_type),
        serde_json::Value::Array(value_types) => value_types
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|value_type| json_value_has_type(value, value_type)),
        _ => false,
    }
}

fn json_value_has_type(value: &serde_json::Value, value_type: &str) -> bool {
    match value_type {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => value
            .as_number()
            .is_some_and(|number| number.is_i64() || number.is_u64()),
        _ => false,
    }
}

fn validate_string_schema(
    schema: &serde_json::Map<String, serde_json::Value>,
    text: &str,
) -> Result<(), ()> {
    if let Some(minimum) = schema.get("minLength").and_then(serde_json::Value::as_u64)
        && u64::try_from(text.chars().count()).map_err(|_| ())? < minimum
    {
        return Err(());
    }
    if let Some(pattern) = schema.get("pattern").and_then(serde_json::Value::as_str)
        && !match pattern {
            "^[a-f0-9]{64}$" => {
                text.len() == 64
                    && text.bytes().all(|character| {
                        character.is_ascii_hexdigit() && !character.is_ascii_uppercase()
                    })
            }
            "^[a-z0-9][a-z0-9_.-]*$" => {
                let mut characters = text.bytes();
                characters.next().is_some_and(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit()
                }) && characters.all(|character| {
                    character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || matches!(character, b'_' | b'.' | b'-')
                })
            }
            "^(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)(?:\\.(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?))*$" => {
                text.split('.').all(is_valid_hostname_label)
            }
            _ => return Err(()),
        }
    {
        return Err(());
    }
    Ok(())
}

fn is_valid_hostname_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || character == b'-')
        && !label.starts_with('-')
        && !label.ends_with('-')
}

fn validate_array_schema(
    schema: &serde_json::Map<String, serde_json::Value>,
    values: &[serde_json::Value],
) -> Result<(), ()> {
    if let Some(minimum) = schema.get("minItems").and_then(serde_json::Value::as_u64)
        && u64::try_from(values.len()).map_err(|_| ())? < minimum
    {
        return Err(());
    }
    if schema
        .get("uniqueItems")
        .and_then(serde_json::Value::as_bool)
        .is_some_and(|unique| unique)
        && values
            .iter()
            .enumerate()
            .any(|(index, value)| values.iter().take(index).any(|prior| prior == value))
    {
        return Err(());
    }
    if let Some(items) = schema.get("items") {
        for value in values {
            validate_json_schema(items, value)?;
        }
    }
    Ok(())
}

fn validate_object_schema(
    schema: &serde_json::Map<String, serde_json::Value>,
    values: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), ()> {
    if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array)
        && required
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|field| !values.contains_key(field))
    {
        return Err(());
    }
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object);
    let additional_properties = schema.get("additionalProperties");
    for (field, value) in values {
        match properties.and_then(|properties| properties.get(field)) {
            Some(field_schema) => validate_json_schema(field_schema, value)?,
            None => match additional_properties {
                Some(serde_json::Value::Object(schema)) => {
                    validate_json_schema(&serde_json::Value::Object(schema.clone()), value)?;
                }
                Some(serde_json::Value::Bool(true)) | None => {}
                _ => return Err(()),
            },
        }
    }
    Ok(())
}

fn validate_number_schema(
    schema: &serde_json::Map<String, serde_json::Value>,
    number: &serde_json::Number,
) -> Result<(), ()> {
    if let Some(minimum) = schema.get("minimum").and_then(serde_json::Value::as_f64)
        && number.as_f64().is_none_or(|value| value < minimum)
    {
        return Err(());
    }
    Ok(())
}

fn referenced_output_schema(path: &str) -> serde_json::Value {
    let bytes = match path {
        "contracts/json-schema/agent-workflow.v1.schema.json" => {
            include_str!("../../../contracts/json-schema/agent-workflow.v1.schema.json")
        }
        "contracts/json-schema/compiled-strategy.v1.schema.json" => {
            include_str!("../../../contracts/json-schema/compiled-strategy.v1.schema.json")
        }
        "contracts/json-schema/data-handling-decision.v1.schema.json" => {
            include_str!("../../../contracts/json-schema/data-handling-decision.v1.schema.json")
        }
        "contracts/json-schema/provider-manifest.v1.schema.json" => {
            include_str!("../../../contracts/json-schema/provider-manifest.v1.schema.json")
        }
        _ => panic!("unregistered MCP output schema reference: {path}"),
    };
    match serde_json::from_str(bytes) {
        Ok(value) => value,
        Err(error) => panic!("registered MCP output schema must parse: {error}"),
    }
}

fn tool_error(_message: String) -> CallToolResult {
    // Facade errors can contain user-controlled identifiers, endpoints or
    // provider diagnostics. Keep the MCP transcript deterministic and do not
    // reflect those values across the protocol boundary.
    CallToolResult::error(vec![ContentBlock::text(
        "operation_rejected: operation rejected by the shared Searchright facade",
    )])
}

fn json_invalid_params(error: serde_json::Error) -> McpError {
    invalid_params(error.to_string())
}

fn invalid_params(message: String) -> McpError {
    McpError::invalid_params(message, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_success_emits_matching_structured_content() -> Result<(), Box<dyn std::error::Error>> {
        let value = serde_json::json!({"valid": true, "contract": "search_strategy"});
        let result = json_success("validate_strategy", &value)?;

        assert_eq!(result.structured_content.as_ref(), Some(&value));
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
        assert!(serde_json::to_string(&result.content)?.contains("search_strategy"));
        Ok(())
    }

    #[test]
    fn tool_errors_are_stable_and_do_not_reflect_sensitive_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let secret = "https://user:password@example.test/search?api_key=secret";
        let first = tool_error(secret.to_owned());
        let second = tool_error("a different internal failure".to_owned());
        let first_json = serde_json::to_value(&first)?;

        assert_eq!(first, second);
        assert_eq!(first.is_error, Some(true));
        assert!(first.structured_content.is_none());
        assert!(first_json.to_string().contains("operation_rejected"));
        assert!(!first_json.to_string().contains("password"));
        assert!(!first_json.to_string().contains("api_key"));
        assert!(!first_json.to_string().contains("secret"));
        Ok(())
    }

    #[test]
    fn every_tool_advertises_governed_output_and_effect_metadata() {
        let server = SearchrightServer::default();
        let tools = server.tool_router.list_all();

        assert_eq!(tools.len(), 31);
        assert_eq!(output_schema_registry().schemas.len(), tools.len());
        for tool in tools {
            let Some(schema) = tool.output_schema else {
                panic!("every tool has an outputSchema")
            };
            assert!(matches!(
                schema.get("type"),
                Some(serde_json::Value::String(_))
            ));
            match schema.get("type").and_then(serde_json::Value::as_str) {
                Some("object") => assert!(
                    schema.get("properties").is_some()
                        || schema.get("allOf").is_some()
                        || schema.get("oneOf").is_some()
                        || schema.get("$ref").is_some(),
                    "{} has a trivial object output schema",
                    tool.name
                ),
                Some("array") => assert!(
                    schema.get("items").is_some(),
                    "{} has a trivial array output schema",
                    tool.name
                ),
                _ => panic!("{} has an unsupported output root", tool.name),
            }

            let Some(annotations) = tool.annotations else {
                panic!("every tool has effect annotations")
            };
            assert_eq!(annotations.read_only_hint, Some(true));
            assert_eq!(annotations.destructive_hint, Some(false));
            assert_eq!(annotations.idempotent_hint, Some(true));
            assert_eq!(annotations.open_world_hint, Some(false));
        }
    }

    #[test]
    fn text_results_retain_machine_readable_content() -> Result<(), McpError> {
        let result = text_success(
            "render_diagnostics",
            "plain_text",
            "stable output".to_owned(),
        )?;

        assert_eq!(result.is_error, Some(false));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("format")),
            Some(&serde_json::json!("plain_text"))
        );
        assert_eq!(result.content, vec![ContentBlock::text("stable output")]);
        Ok(())
    }

    #[test]
    fn output_registry_rejects_unknown_fields_and_wrong_value_types() {
        assert!(
            validate_tool_output(
                "validate_plan",
                &serde_json::json!({
                    "review_id": "review-1",
                    "findings": [],
                    "ready_for_strategy_design": true,
                    "unexpected": true,
                }),
            )
            .is_err()
        );
        assert!(
            validate_tool_output(
                "validate_plan",
                &serde_json::json!({
                    "review_id": "review-1",
                    "findings": [],
                    "ready_for_strategy_design": "yes",
                }),
            )
            .is_err()
        );
        assert!(json_success("unknown_tool", &serde_json::json!({"valid": true}),).is_err());
    }

    #[test]
    fn output_registry_preserves_catalogue_unions_and_referenced_schemas() {
        assert!(
            validate_tool_output(
                "generate_prisma",
                &serde_json::json!({"document": "graph TD", "format": "mermaid"}),
            )
            .is_ok()
        );
        assert!(
            validate_tool_output(
                "generate_prisma",
                &serde_json::json!({"document": "graph TD"}),
            )
            .is_err()
        );
        assert!(validate_tool_output(
            "compile_strategy",
            &serde_json::json!({
                "schema_version": "org.searchright.compiled-strategy.v1",
                "strategy_id": "strategy-1",
                "dialect": "pub_med",
                "query": "cancer",
                "warnings": [],
                "fidelity": "exact",
                "review_required": false,
                "loss_codes": [],
                "compilation_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "compiler_version": "0.1.0",
            }),
        )
        .is_ok());
    }

    #[test]
    fn live_client_success_matrix_covers_every_advertised_tool_and_prisma_branch()
    -> Result<(), String> {
        let cases = live_client_success_cases()?;
        let advertised = SearchrightServer::default().tool_router.list_all();
        let expected: std::collections::BTreeSet<_> =
            advertised.iter().map(|tool| tool.name.as_ref()).collect();
        let covered: std::collections::BTreeSet<_> =
            cases.iter().map(|case| case.tool_name).collect();

        assert_eq!(expected, covered);
        assert_eq!(cases.len(), advertised.len() + 1);
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.tool_name == "generate_prisma")
                .count(),
            2
        );
        assert!(
            cases
                .iter()
                .all(|case| case.arguments.keys().all(|key| !key.trim().is_empty()))
        );
        Ok(())
    }
}
