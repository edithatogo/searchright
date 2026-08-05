#![forbid(unsafe_code)]

use evidence_search_core::{AuditLedger, QueryCompiler};
use rmcp::{
    ErrorData as McpError, ServiceExt,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
    transport::stdio,
};
use searchright_agent::{AgentWorkflow, assess_plan_readiness};
use searchright_contracts::{
    AuditEvent, BibliographicRecord, PrismaFlow, ReviewPlan, SearchDialect, SearchStrategy, Validate,
};
use searchright_dedup::{DedupConfig, Deduplicator};
use searchright_prisma::{build_prisma_s_ledger, render_mermaid, validate_flow};
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

#[derive(Clone, Default)]
struct SearchrightServer;

#[tool_router(server_handler)]
impl SearchrightServer {
    #[tool(description = "Validate a versioned review plan and return methodological readiness findings")]
    fn validate_plan(
        &self,
        Parameters(input): Parameters<DocumentInput>,
    ) -> Result<CallToolResult, McpError> {
        let plan: ReviewPlan = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        if let Err(error) = plan.validate() {
            return Ok(tool_error(error.to_string()));
        }
        json_success(&serde_json::json!({
            "valid": true,
            "review_id": plan.review_id,
            "readiness_findings": assess_plan_readiness(&plan),
        }))
    }

    #[tool(description = "Compile a portable Searchright query AST into a declared database dialect")]
    fn compile_strategy(
        &self,
        Parameters(input): Parameters<CompileInput>,
    ) -> Result<CallToolResult, McpError> {
        let strategy: SearchStrategy = parse_document(&input.document, input.format.as_deref())
            .map_err(invalid_params)?;
        let dialect = parse_dialect(&input.dialect).map_err(invalid_params)?;
        match QueryCompiler::compile(&strategy, dialect) {
            Ok(compiled) => json_success(&compiled),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Generate reviewable duplicate clusters without deleting source records")]
    fn deduplicate_records(
        &self,
        Parameters(input): Parameters<DedupInput>,
    ) -> Result<CallToolResult, McpError> {
        let records: Vec<BibliographicRecord> =
            serde_json::from_str(&input.records_json).map_err(|error| invalid_params(error.to_string()))?;
        let threshold = input.title_threshold.unwrap_or(0.92);
        if !(0.0..=1.0).contains(&threshold) {
            return Err(invalid_params("title_threshold must be between zero and one".to_owned()));
        }
        let deduplicator = Deduplicator::new(DedupConfig {
            title_similarity_threshold: threshold,
            ..DedupConfig::default()
        })
        .map_err(|error| invalid_params(error.to_string()))?;
        match deduplicator.cluster(&records) {
            Ok(result) => json_success(&result),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Validate PRISMA flow arithmetic and render JSON, Mermaid or the PRISMA-S ledger")]
    fn generate_prisma(
        &self,
        Parameters(input): Parameters<PrismaInput>,
    ) -> Result<CallToolResult, McpError> {
        let flow: PrismaFlow =
            serde_json::from_str(&input.flow_json).map_err(|error| invalid_params(error.to_string()))?;
        if let Err(error) = validate_flow(&flow) {
            return Ok(tool_error(error.to_string()));
        }
        match input.output.as_str() {
            "json" => json_success(&flow),
            "mermaid" => match render_mermaid(&flow) {
                Ok(mermaid) => Ok(CallToolResult::success(vec![ContentBlock::text(mermaid)])),
                Err(error) => Ok(tool_error(error.to_string())),
            },
            "prisma_s_ledger" => match build_prisma_s_ledger(&flow) {
                Ok(ledger) => json_success(&ledger),
                Err(error) => Ok(tool_error(error.to_string())),
            },
            _ => Err(invalid_params(
                "output must be json, mermaid or prisma_s_ledger".to_owned(),
            )),
        }
    }

    #[tool(description = "Verify a hash-chained Searchright audit JSONL stream")]
    fn verify_audit(
        &self,
        Parameters(input): Parameters<AuditInput>,
    ) -> Result<CallToolResult, McpError> {
        let mut events = Vec::new();
        for (index, line) in input.audit_jsonl.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<AuditEvent>(line) {
                Ok(event) => events.push(event),
                Err(error) => {
                    return Err(invalid_params(format!(
                        "invalid audit event at line {}: {error}",
                        index.saturating_add(1)
                    )));
                }
            }
        }
        match AuditLedger::from_events(events).verify() {
            Ok(verification) => json_success(&serde_json::json!({
                "valid": true,
                "event_count": verification.event_count,
                "head_hash": verification.head_hash,
            })),
            Err(error) => Ok(tool_error(error.to_string())),
        }
    }

    #[tool(description = "Return the conservative planning, execution and screening workflow with authority gates")]
    fn workflow(&self) -> Result<CallToolResult, McpError> {
        json_success(&AgentWorkflow::systematic_search())
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

fn json_success(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    serde_json::to_string_pretty(value)
        .map(|json| CallToolResult::success(vec![ContentBlock::text(json)]))
        .map_err(|error| McpError::internal_error(error.to_string(), None))
}

fn tool_error(message: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

fn invalid_params(message: String) -> McpError {
    McpError::invalid_params(message, None)
}
