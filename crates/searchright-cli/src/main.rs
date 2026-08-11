//! Command-line interface for Searchright's contract-first review operations.
//!
//! The binary exposes explicit subcommands for validating, compiling, and
//! transforming review artefacts through the shared [`SearchrightEngine`].

#![forbid(unsafe_code)]
#![allow(
    clippy::match_same_arms,
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "legacy compatibility aliases deliberately delegate to the same facade operations; the CLI emits requested output to stdout and stable errors to stderr"
)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum, error::ErrorKind};
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

#[derive(Debug, Parser)]
#[command(
    name = "searchright",
    version,
    about = "Contract-first systematic-search infrastructure"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Preview or apply creation of a conservative local CLI configuration.
    Init {
        #[arg(long, default_value = ".searchright.json")]
        target: PathBuf,
        /// Apply the write. Without this flag the command is a dry run.
        #[arg(long)]
        apply: bool,
    },
    /// Review-plan operations.
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    /// Source and provider operations.
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },
    /// Search-strategy operations.
    Strategy {
        #[command(subcommand)]
        command: StrategyCommand,
    },
    /// Bounded execution-authority operations.
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Record import operations.
    Import {
        #[command(subcommand)]
        command: ImportCommand,
    },
    /// Human-governed screening support operations.
    Screen {
        #[command(subcommand)]
        command: ScreenCommand,
    },
    /// Reporting and provenance operations.
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Validate a review plan and report readiness findings.
    ValidatePlan { input: PathBuf },
    /// Validate a source-specific search strategy.
    ValidateStrategy { input: PathBuf },
    /// Validate neutral, non-canonical document extraction evidence.
    ValidateDocumentEvidence { input: PathBuf },
    /// Compile a portable strategy into source syntax.
    Compile {
        input: PathBuf,
        #[arg(long, value_enum)]
        dialect: DialectArg,
    },
    /// Deduplicate a JSON/YAML array of bibliographic records.
    Deduplicate {
        input: PathBuf,
        #[arg(long, default_value_t = 0.92)]
        title_threshold: f64,
    },
    /// Validate or render a PRISMA flow contract.
    Prisma {
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = PrismaFormat::Json)]
        format: PrismaFormat,
    },
    /// Verify a JSONL hash-chained audit ledger.
    VerifyAudit { input: PathBuf },
    /// Import bibliographic records and preserve source provenance.
    ImportRecords {
        input: PathBuf,
        #[arg(long, value_enum)]
        format: InterchangeArg,
        #[arg(long)]
        source_receipt_id: String,
    },
    /// Export canonical records with a conversion receipt.
    ExportRecords {
        input: PathBuf,
        #[arg(long)]
        review_id: String,
        #[arg(long, value_enum, default_value_t = InterchangeArg::SearchrightJson)]
        input_format: InterchangeArg,
        #[arg(long, value_enum)]
        output_format: InterchangeArg,
    },
    /// Validate and summarise a record-report-study graph.
    StudyGraph { input: PathBuf },
    /// Evaluate PRESS, seed-set recall and translation-loss gates.
    ValidateSearch { input: PathBuf },
    /// Compare parent and current result sets for a living review.
    LivingDiff { previous: PathBuf, current: PathBuf },
    /// Validate a set of living-update lineage contracts.
    ValidateLivingLineage { input: PathBuf },
    /// Build RO-Crate and W3C PROV-compatible exports.
    Provenance { input: PathBuf },
    /// Rank records transparently for prioritisation only.
    Rank {
        input: PathBuf,
        #[arg(long, required = true)]
        query_term: Vec<String>,
    },
    /// Inspect untrusted text for instruction-like or active-content markers.
    InspectContent {
        input: PathBuf,
        #[arg(long)]
        subject_id: String,
        #[arg(long, value_enum, default_value_t = ContentPolicyArg::DataOnly)]
        policy: ContentPolicyArg,
    },
    /// Render stable accessible diagnostics without ANSI-dependent output.
    RenderDiagnostics {
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = DiagnosticFormatArg::PlainText)]
        format: DiagnosticFormatArg,
    },
    /// Evaluate a data-handling request against an institutional policy.
    EvaluateGovernance { policy: PathBuf, request: PathBuf },
    /// Authorise an HTTPS endpoint against an execution envelope.
    AuthoriseEndpoint { input: PathBuf, endpoint: String },
    /// Validate a protocol amendment.
    ValidateAmendment { input: PathBuf },
    /// Validate a methodological standards pack.
    ValidateStandardPack { input: PathBuf },
    /// Validate an assessment against a standards pack.
    ValidateStandardAssessment { input: PathBuf },
    /// Validate ranking calibration and its no-auto-exclusion contract.
    ValidateRankingCalibration { input: PathBuf },
    /// Validate a supplementary-discovery run.
    ValidateDiscoveryRun { input: PathBuf },
    /// Verify an evidence-bearing lifecycle trace against the finite assurance model.
    VerifyWorkflowTrace { input: PathBuf },
    /// Resolve bounded supplementary-discovery candidates for human release.
    DiscoveryCandidates { input: PathBuf },
    /// Verify a WASI provider-component manifest against exact component bytes.
    VerifyProviderComponent {
        manifest: PathBuf,
        component: PathBuf,
    },
    /// Build a redacted bring-your-own-access request plan.
    PlanLicensedRequest {
        profile: PathBuf,
        strategy: PathBuf,
        endpoint: String,
    },
    /// Validate a benchmark report and its explicit claim boundary.
    ValidateBenchmarkReport { input: PathBuf },
    /// List provider manifests available in the default no-network build.
    Providers,
    /// Print the conservative agent workflow policy.
    Workflow,
}

#[derive(Debug, Subcommand)]
enum PlanCommand {
    /// Validate a review plan and report readiness findings.
    Validate { input: PathBuf },
}

#[derive(Debug, Subcommand)]
enum SourceCommand {
    /// List fixture-backed providers available without network access.
    List,
}

#[derive(Debug, Subcommand)]
enum StrategyCommand {
    /// Validate a source-specific search strategy.
    Validate { input: PathBuf },
    /// Compile a portable strategy into source syntax.
    Compile {
        input: PathBuf,
        #[arg(long, value_enum)]
        dialect: DialectArg,
    },
}

#[derive(Debug, Subcommand)]
enum RunCommand {
    /// Check endpoint authority without executing a request.
    AuthoriseEndpoint { input: PathBuf, endpoint: String },
}

#[derive(Debug, Subcommand)]
enum ImportCommand {
    /// Import bibliographic records and preserve source provenance.
    Records {
        input: PathBuf,
        #[arg(long, value_enum)]
        format: InterchangeArg,
        #[arg(long)]
        source_receipt_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum ScreenCommand {
    /// Rank records for prioritisation without making exclusion decisions.
    Rank {
        input: PathBuf,
        #[arg(long, required = true)]
        query_term: Vec<String>,
    },
    /// Validate and summarise explicit record-report-study linkage.
    StudyGraph { input: PathBuf },
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    /// Validate or render a PRISMA flow contract.
    Prisma {
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = PrismaFormat::Json)]
        format: PrismaFormat,
    },
    /// Build RO-Crate and W3C PROV-compatible exports.
    Provenance { input: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DialectArg {
    Pubmed,
    OvidMedline,
    Embase,
    EuropePmc,
    CinahlEbsco,
    PsycinfoOvid,
    Scopus,
    WebOfScience,
    Crossref,
    Openalex,
    ClinicaltrialsGov,
    GenericBoolean,
}

impl From<DialectArg> for SearchDialect {
    fn from(value: DialectArg) -> Self {
        match value {
            DialectArg::Pubmed => Self::PubMed,
            DialectArg::OvidMedline => Self::OvidMedline,
            DialectArg::Embase => Self::Embase,
            DialectArg::EuropePmc => Self::EuropePmc,
            DialectArg::CinahlEbsco => Self::CinahlEbsco,
            DialectArg::PsycinfoOvid => Self::PsycInfoOvid,
            DialectArg::Scopus => Self::Scopus,
            DialectArg::WebOfScience => Self::WebOfScience,
            DialectArg::Crossref => Self::Crossref,
            DialectArg::Openalex => Self::OpenAlex,
            DialectArg::ClinicaltrialsGov => Self::ClinicalTrialsGov,
            DialectArg::GenericBoolean => Self::GenericBoolean,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum PrismaFormat {
    #[default]
    Json,
    Mermaid,
    Ledger,
}

impl From<PrismaFormat> for PrismaOutput {
    fn from(value: PrismaFormat) -> Self {
        match value {
            PrismaFormat::Json => Self::Json,
            PrismaFormat::Mermaid => Self::Mermaid,
            PrismaFormat::Ledger => Self::PrismaSLedger,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum InterchangeArg {
    #[default]
    SearchrightJson,
    JsonLines,
    CslJson,
    Ris,
    Nbib,
    Csv,
}

impl From<InterchangeArg> for InterchangeFormat {
    fn from(value: InterchangeArg) -> Self {
        match value {
            InterchangeArg::SearchrightJson => Self::SearchrightJson,
            InterchangeArg::JsonLines => Self::JsonLines,
            InterchangeArg::CslJson => Self::CslJson,
            InterchangeArg::Ris => Self::Ris,
            InterchangeArg::Nbib => Self::Nbib,
            InterchangeArg::Csv => Self::Csv,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ContentPolicyArg {
    #[default]
    DataOnly,
    SanitiseThenDataOnly,
    HumanInspectionRequired,
}

impl From<ContentPolicyArg> for UntrustedContentPolicy {
    fn from(value: ContentPolicyArg) -> Self {
        match value {
            ContentPolicyArg::DataOnly => Self::DataOnly,
            ContentPolicyArg::SanitiseThenDataOnly => Self::SanitiseThenDataOnly,
            ContentPolicyArg::HumanInspectionRequired => Self::HumanInspectionRequired,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum DiagnosticFormatArg {
    #[default]
    PlainText,
    Json,
    JsonLines,
}

impl From<DiagnosticFormatArg> for searchright::diagnostics::DiagnosticOutput {
    fn from(value: DiagnosticFormatArg) -> Self {
        match value {
            DiagnosticFormatArg::PlainText => Self::PlainText,
            DiagnosticFormatArg::Json => Self::Json,
            DiagnosticFormatArg::JsonLines => Self::JsonLines,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProvenanceInput {
    plan: ReviewPlan,
    #[serde(default)]
    receipts: Vec<SourceReceipt>,
    #[serde(default)]
    events: Vec<AuditEvent>,
}

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => match execute(cli) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => emit_error(&error, 3),
        },
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            print!("{error}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            let message = error.to_string();
            emit_error(&anyhow::anyhow!(message.trim().to_owned()), 2)
        }
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { target, apply } => initialise(&target, apply)?,
        Command::Plan {
            command: PlanCommand::Validate { input },
        } => {
            let plan: ReviewPlan = read_document(&input)?;
            print_json(&SearchrightEngine::validate_plan(&plan)?)?;
        }
        Command::Source {
            command: SourceCommand::List,
        } => print_json(&SearchrightEngine::default_provider_manifests()?)?,
        Command::Strategy {
            command: StrategyCommand::Validate { input },
        } => {
            let strategy: SearchStrategy = read_document(&input)?;
            SearchrightEngine::validate_strategy(&strategy)
                .context("search strategy is invalid")?;
            print_json(&serde_json::json!({
                "valid": true,
                "strategy_id": strategy.strategy_id,
            }))?;
        }
        Command::Strategy {
            command: StrategyCommand::Compile { input, dialect },
        } => {
            let strategy: SearchStrategy = read_document(&input)?;
            print_json(&SearchrightEngine::compile_strategy(
                &strategy,
                dialect.into(),
            )?)?;
        }
        Command::Run {
            command: RunCommand::AuthoriseEndpoint { input, endpoint },
        } => {
            let envelope: ExecutionEnvelope = read_document(&input)?;
            SearchrightEngine::authorise_endpoint(&envelope, &endpoint)?;
            print_json(&serde_json::json!({"authorised": true, "endpoint": endpoint}))?;
        }
        Command::Import {
            command:
                ImportCommand::Records {
                    input,
                    format,
                    source_receipt_id,
                },
        } => {
            let document = fs::read_to_string(&input)
                .with_context(|| format!("could not read {}", input.display()))?;
            print_json(&SearchrightEngine::import_records(
                &document,
                format.into(),
                &source_receipt_id,
            )?)?;
        }
        Command::Screen {
            command: ScreenCommand::Rank { input, query_term },
        } => {
            let records: Vec<BibliographicRecord> = read_document(&input)?;
            print_json(&SearchrightEngine::rank_records(&records, &query_term)?)?;
        }
        Command::Screen {
            command: ScreenCommand::StudyGraph { input },
        } => {
            let graph: StudyGraph = read_document(&input)?;
            print_json(&SearchrightEngine::assess_study_graph(&graph)?)?;
        }
        Command::Report {
            command: ReportCommand::Prisma { input, format },
        } => {
            let flow: PrismaFlow = read_document(&input)?;
            match SearchrightEngine::prisma(&flow, format.into())? {
                PrismaArtifact::Mermaid(document) => println!("{document}"),
                artifact => print_json(&artifact)?,
            }
        }
        Command::Report {
            command: ReportCommand::Provenance { input },
        } => {
            let provenance: ProvenanceInput = read_document(&input)?;
            print_json(&SearchrightEngine::provenance(
                &provenance.plan,
                &provenance.receipts,
                &provenance.events,
            )?)?;
        }
        Command::ValidatePlan { input } => {
            let plan: ReviewPlan = read_document(&input)?;
            print_json(&SearchrightEngine::validate_plan(&plan)?)?;
        }
        Command::ValidateStrategy { input } => {
            let strategy: SearchStrategy = read_document(&input)?;
            SearchrightEngine::validate_strategy(&strategy)
                .context("search strategy is invalid")?;
            print_json(&serde_json::json!({
                "valid": true,
                "strategy_id": strategy.strategy_id,
            }))?;
        }
        Command::ValidateDocumentEvidence { input } => {
            let evidence: DocumentEvidence = read_document(&input)?;
            SearchrightEngine::validate_document_evidence(&evidence)
                .context("document evidence is invalid")?;
            print_json(&serde_json::json!({
                "valid": true,
                "document_id": evidence.document_id,
                "canonical_write_permitted": evidence.canonical_write_permitted,
            }))?;
        }
        Command::Compile { input, dialect } => {
            let strategy: SearchStrategy = read_document(&input)?;
            print_json(&SearchrightEngine::compile_strategy(
                &strategy,
                dialect.into(),
            )?)?;
        }
        Command::Deduplicate {
            input,
            title_threshold,
        } => {
            if !(0.0..=1.0).contains(&title_threshold) {
                bail!("title-threshold must be between zero and one");
            }
            let records: Vec<BibliographicRecord> = read_document(&input)?;
            print_json(&SearchrightEngine::deduplicate(
                &records,
                DedupConfig {
                    title_similarity_threshold: title_threshold,
                    ..DedupConfig::default()
                },
            )?)?;
        }
        Command::Prisma { input, format } => {
            let flow: PrismaFlow = read_document(&input)?;
            match SearchrightEngine::prisma(&flow, format.into())? {
                PrismaArtifact::Mermaid(document) => println!("{document}"),
                artifact => print_json(&artifact)?,
            }
        }
        Command::VerifyAudit { input } => {
            print_json(&SearchrightEngine::verify_audit(read_jsonl(&input)?)?)?;
        }
        Command::ImportRecords {
            input,
            format,
            source_receipt_id,
        } => {
            let document = fs::read_to_string(&input)
                .with_context(|| format!("could not read {}", input.display()))?;
            print_json(&SearchrightEngine::import_records(
                &document,
                format.into(),
                &source_receipt_id,
            )?)?;
        }
        Command::ExportRecords {
            input,
            review_id,
            input_format,
            output_format,
        } => {
            let records: Vec<BibliographicRecord> = read_document(&input)?;
            print_json(&SearchrightEngine::export_records(
                &review_id,
                &records,
                input_format.into(),
                output_format.into(),
            )?)?;
        }
        Command::StudyGraph { input } => {
            let graph: StudyGraph = read_document(&input)?;
            print_json(&SearchrightEngine::assess_study_graph(&graph)?)?;
        }
        Command::ValidateSearch { input } => {
            let report: SearchValidationReport = read_document(&input)?;
            print_json(&SearchrightEngine::assess_search_validation(&report)?)?;
        }
        Command::LivingDiff { previous, current } => {
            let previous: Vec<BibliographicRecord> = read_document(&previous)?;
            let current: Vec<BibliographicRecord> = read_document(&current)?;
            print_json(&SearchrightEngine::diff_living_records(
                &previous, &current,
            )?)?;
        }
        Command::ValidateLivingLineage { input } => {
            let runs: Vec<LivingUpdateRun> = read_document(&input)?;
            SearchrightEngine::validate_living_lineage(&runs)?;
            print_json(&serde_json::json!({"valid": true, "run_count": runs.len()}))?;
        }
        Command::Provenance { input } => {
            let provenance: ProvenanceInput = read_document(&input)?;
            print_json(&SearchrightEngine::provenance(
                &provenance.plan,
                &provenance.receipts,
                &provenance.events,
            )?)?;
        }
        Command::Rank { input, query_term } => {
            let records: Vec<BibliographicRecord> = read_document(&input)?;
            print_json(&SearchrightEngine::rank_records(&records, &query_term)?)?;
        }
        Command::InspectContent {
            input,
            subject_id,
            policy,
        } => {
            let content = fs::read_to_string(&input)
                .with_context(|| format!("could not read {}", input.display()))?;
            print_json(&SearchrightEngine::inspect_content(
                &subject_id,
                &content,
                policy.into(),
            ))?;
        }
        Command::RenderDiagnostics { input, format } => {
            let diagnostics: Vec<Diagnostic> = read_document(&input)?;
            let document = SearchrightEngine::render_diagnostics(&diagnostics, format.into())?;
            print!("{document}");
        }
        Command::EvaluateGovernance { policy, request } => {
            let policy: InstitutionalPolicy = read_document(&policy)?;
            let request: DataHandlingRequest = read_document(&request)?;
            print_json(&SearchrightEngine::evaluate_governance(&policy, &request)?)?;
        }
        Command::AuthoriseEndpoint { input, endpoint } => {
            let envelope: ExecutionEnvelope = read_document(&input)?;
            SearchrightEngine::authorise_endpoint(&envelope, &endpoint)?;
            print_json(&serde_json::json!({"authorised": true, "endpoint": endpoint}))?;
        }
        Command::ValidateAmendment { input } => {
            let value: ProtocolAmendment = read_document(&input)?;
            SearchrightEngine::validate_amendment(&value)?;
            valid_receipt("protocol_amendment")?;
        }
        Command::ValidateStandardPack { input } => {
            let value: StandardPack = read_document(&input)?;
            SearchrightEngine::validate_standard_pack(&value)?;
            valid_receipt("standard_pack")?;
        }
        Command::ValidateStandardAssessment { input } => {
            let value: StandardAssessment = read_document(&input)?;
            SearchrightEngine::validate_standard_assessment(&value)?;
            valid_receipt("standard_assessment")?;
        }
        Command::ValidateRankingCalibration { input } => {
            let value: RankingCalibration = read_document(&input)?;
            SearchrightEngine::validate_ranking_calibration(&value)?;
            valid_receipt("ranking_calibration")?;
        }
        Command::ValidateDiscoveryRun { input } => {
            let value: DiscoveryRun = read_document(&input)?;
            SearchrightEngine::validate_discovery_run(&value)?;
            valid_receipt("discovery_run")?;
        }
        Command::VerifyWorkflowTrace { input } => {
            let trace: WorkflowTrace = read_document(&input)?;
            print_json(&SearchrightEngine::verify_workflow_trace(&trace)?)?;
        }
        Command::DiscoveryCandidates { input } => {
            let run: DiscoveryRun = read_document(&input)?;
            print_json(&SearchrightEngine::discovery_candidates(&run)?)?;
        }
        Command::VerifyProviderComponent {
            manifest,
            component,
        } => {
            let manifest: ProviderComponentManifest = read_document(&manifest)?;
            let bytes = fs::read(&component)
                .with_context(|| format!("could not read {}", component.display()))?;
            SearchrightEngine::verify_provider_component(&manifest, &bytes)?;
            print_json(&serde_json::json!({
                "valid": true,
                "component_id": manifest.component_id,
                "bytes": bytes.len(),
            }))?;
        }
        Command::PlanLicensedRequest {
            profile,
            strategy,
            endpoint,
        } => {
            let profile: LicensedAdapterProfile = read_document(&profile)?;
            let strategy: CompiledStrategy = read_document(&strategy)?;
            print_json(&SearchrightEngine::plan_licensed_request(
                &profile, &strategy, &endpoint,
            )?)?;
        }
        Command::ValidateBenchmarkReport { input } => {
            let report: BenchmarkReport = read_document(&input)?;
            SearchrightEngine::validate_benchmark_report(&report)?;
            valid_receipt("benchmark_report")?;
        }
        Command::Providers => {
            print_json(&SearchrightEngine::default_provider_manifests()?)?;
        }
        Command::Workflow => print_json(&SearchrightEngine::workflow())?,
    }
    Ok(())
}

fn initialise(target: &Path, apply: bool) -> Result<()> {
    let document = serde_json::json!({
        "schema_version": "org.searchright.cli-config.v1",
        "network_enabled": false,
        "write_authority": "explicit_apply",
    });
    if apply {
        if target.exists() {
            bail!(
                "refusing to overwrite existing configuration at {}",
                target.display()
            );
        }
        let encoded = format!("{}\n", serde_json::to_string_pretty(&document)?);
        fs::write(target, encoded)
            .with_context(|| format!("could not write configuration to {}", target.display()))?;
    }
    print_json(&serde_json::json!({
        "schema_version": "org.searchright.cli-result.v1",
        "operation": "init",
        "mode": if apply { "apply" } else { "dry_run" },
        "target": target,
        "changed": apply,
        "configuration": document,
    }))
}

fn emit_error(error: &anyhow::Error, exit_code: u8) -> ExitCode {
    let envelope = serde_json::json!({
        "schema_version": "org.searchright.cli-error.v1",
        "code": if exit_code == 2 { "cli.usage" } else { "cli.operation_failed" },
        "message": error.to_string(),
        "corrective_action": if exit_code == 2 {
            "Run searchright --help and correct the command arguments."
        } else {
            "Review the named input or authority policy, then retry."
        },
        "partial_output_safe": false,
    });
    eprintln!("{envelope}");
    ExitCode::from(exit_code)
}

fn read_document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("yaml" | "yml") => serde_yaml::from_str(&content)
            .with_context(|| format!("invalid YAML in {}", path.display())),
        _ => serde_json::from_str(&content)
            .with_context(|| format!("invalid JSON in {}", path.display())),
    }
}

fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    let mut values = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        values.push(
            serde_json::from_str(line).with_context(|| {
                format!("invalid JSONL value at line {}", index.saturating_add(1))
            })?,
        );
    }
    Ok(values)
}

fn valid_receipt(contract: &str) -> Result<()> {
    print_json(&serde_json::json!({"valid": true, "contract": contract}))
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn grouped_command_hierarchy_is_stable() {
        let command = Cli::command();
        let names = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect::<Vec<_>>();
        for expected in [
            "init", "plan", "source", "strategy", "run", "import", "screen", "report",
        ] {
            assert!(names.contains(&expected), "missing {expected} command");
        }
    }

    #[test]
    fn init_defaults_to_dry_run() {
        let cli = Cli::try_parse_from(["searchright", "init", "--target", "not-created.json"])
            .unwrap_or_else(|error| panic!("command should parse: {error}"));
        assert!(matches!(cli.command, Command::Init { apply: false, .. }));
    }

    #[test]
    fn init_refuses_to_overwrite() {
        let result = initialise(Path::new("Cargo.toml"), true);
        let Err(error) = result else {
            panic!("existing files must not be overwritten");
        };
        assert!(error.to_string().contains("refusing to overwrite"));
    }

    #[test]
    fn grouped_run_only_checks_authority() {
        let cli = Cli::try_parse_from([
            "searchright",
            "run",
            "authorise-endpoint",
            "envelope.json",
            "https://example.org",
        ])
        .unwrap_or_else(|error| panic!("command should parse: {error}"));
        assert!(matches!(
            cli.command,
            Command::Run {
                command: RunCommand::AuthoriseEndpoint { .. }
            }
        ));
    }
}
