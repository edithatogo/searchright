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
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};
use searchright::contracts::{
    AuditEvent, BenchmarkReport, BibliographicRecord, CompiledStrategy, DataHandlingRequest,
    Diagnostic, DiscoveryRun, DocumentEvidence, ExecutionEnvelope, InstitutionalPolicy,
    InterchangeFormat, LicensedAdapterProfile, LivingUpdateRun, PressReview, PrismaFlow,
    ProtocolAmendment, ProviderComponentManifest, RankingCalibration, ReviewPlan, SearchDialect,
    SearchRequest, SearchStrategy, SearchValidationReport, SourceReceipt, StandardAssessment,
    StandardPack, StudyGraph, UntrustedContentPolicy, WorkflowTrace,
};
use searchright::dedup::DedupConfig;
use searchright::{
    LocalReviewOperation, PrismaArtifact, PrismaOutput, SearchExecutionOperation, SearchrightEngine,
};
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Zsh,
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
    /// Validate and optionally persist a human-confirmed review-plan draft.
    PlanReview {
        input: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirmation: Option<PathBuf>,
    },
    /// Validate and optionally persist human-confirmed PRESS evidence.
    PressReviewStrategy {
        input: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirmation: Option<PathBuf>,
    },
    /// Preview or apply deterministic fixture execution under an envelope.
    ExecuteSearch {
        request: PathBuf,
        envelope: PathBuf,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        source_label: String,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        commit_id: Option<String>,
        #[arg(long)]
        confirmed_by: Option<String>,
    },
    /// Persist one complete screening decision under the supplied role policy.
    RecordScreeningDecision {
        policy: PathBuf,
        decision: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
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
    /// Generate a shell completion script on standard output.
    Completions {
        #[arg(value_enum)]
        shell: CompletionShell,
    },
    /// Generate the searchright(1) manual page on standard output.
    Manpage,
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
    /// Validate and optionally persist a human-confirmed review-plan draft.
    Review {
        input: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirmation: Option<PathBuf>,
    },
    /// Validate a review plan and report readiness findings.
    Validate { input: PathBuf },
    /// Validate a protocol amendment.
    ValidateAmendment { input: PathBuf },
    /// Evaluate a data-handling request against institutional policy.
    EvaluateGovernance { policy: PathBuf, request: PathBuf },
    /// Print the conservative agent workflow policy.
    Workflow,
}

#[derive(Debug, Subcommand)]
enum SourceCommand {
    /// List fixture-backed providers available without network access.
    List,
    /// Check endpoint authority without executing a request.
    AuthoriseEndpoint { input: PathBuf, endpoint: String },
    /// Validate a supplementary-discovery run.
    ValidateDiscoveryRun { input: PathBuf },
    /// Resolve discovery candidates for human release.
    DiscoveryCandidates { input: PathBuf },
    /// Verify a WASI provider-component manifest against exact bytes.
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
}

#[derive(Debug, Subcommand)]
enum StrategyCommand {
    /// Validate and optionally persist human-confirmed PRESS evidence.
    PressReview {
        input: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirmation: Option<PathBuf>,
    },
    /// Validate a source-specific search strategy.
    Validate { input: PathBuf },
    /// Compile a portable strategy into source syntax.
    Compile {
        input: PathBuf,
        #[arg(long, value_enum)]
        dialect: DialectArg,
    },
    /// Evaluate PRESS, seed-set recall and translation-loss gates.
    ValidateSearch { input: PathBuf },
    /// Validate a methodological standards pack.
    ValidateStandardPack { input: PathBuf },
    /// Validate an assessment against a standards pack.
    ValidateStandardAssessment { input: PathBuf },
    /// Validate a benchmark report and its explicit claim boundary.
    ValidateBenchmarkReport { input: PathBuf },
}

#[derive(Debug, Subcommand)]
enum RunCommand {
    /// Preview or apply deterministic fixture execution under an envelope.
    Execute {
        request: PathBuf,
        envelope: PathBuf,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        source_label: String,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        commit_id: Option<String>,
        #[arg(long)]
        confirmed_by: Option<String>,
    },
    /// Check endpoint authority without executing a request.
    AuthoriseEndpoint { input: PathBuf, endpoint: String },
    /// Verify a JSONL hash-chained audit ledger.
    VerifyAudit { input: PathBuf },
    /// Verify an evidence-bearing lifecycle trace.
    VerifyWorkflowTrace { input: PathBuf },
    /// Inspect untrusted text without executing embedded instructions.
    InspectContent {
        input: PathBuf,
        #[arg(long)]
        subject_id: String,
        #[arg(long, value_enum, default_value_t = ContentPolicyArg::DataOnly)]
        policy: ContentPolicyArg,
    },
    /// Validate neutral, non-canonical document extraction evidence.
    ValidateDocumentEvidence { input: PathBuf },
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
    /// Deduplicate records without deleting source records.
    Deduplicate {
        input: PathBuf,
        #[arg(long, default_value_t = 0.92)]
        title_threshold: f64,
    },
}

#[derive(Debug, Subcommand)]
enum ScreenCommand {
    /// Persist one complete screening decision under the supplied role policy.
    RecordDecision {
        policy: PathBuf,
        decision: PathBuf,
        #[arg(long, default_value = ".searchright/review-store")]
        store: PathBuf,
    },
    /// Rank records for prioritisation without making exclusion decisions.
    Rank {
        input: PathBuf,
        #[arg(long, required = true)]
        query_term: Vec<String>,
    },
    /// Validate and summarise explicit record-report-study linkage.
    StudyGraph { input: PathBuf },
    /// Validate ranking calibration and its no-auto-exclusion contract.
    ValidateRankingCalibration { input: PathBuf },
    /// Compare parent and current result sets for a living review.
    LivingDiff { previous: PathBuf, current: PathBuf },
    /// Validate living-update lineage contracts.
    ValidateLivingLineage { input: PathBuf },
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
    /// Render stable accessible diagnostics without ANSI-dependent output.
    RenderDiagnostics {
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = DiagnosticFormatArg::PlainText)]
        format: DiagnosticFormatArg,
    },
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
        Ok(cli) => {
            let command = canonical_command(cli.command);
            let stage = command_stage(&command);
            match execute(command) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => emit_operation_error(stage, &error),
            }
        }
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            print!("{error}");
            ExitCode::SUCCESS
        }
        Err(_error) => emit_error("command arguments did not match the CLI contract", 2),
    }
}

const fn command_stage(command: &Command) -> &'static str {
    match command {
        Command::Init { .. } => "init",
        Command::PlanReview { .. } => "plan-review",
        Command::PressReviewStrategy { .. } => "press-review-strategy",
        Command::ExecuteSearch { .. } => "execute-search",
        Command::RecordScreeningDecision { .. } => "record-screening-decision",
        Command::ValidatePlan { .. } => "validate-plan",
        Command::ValidateStrategy { .. } => "validate-strategy",
        Command::ValidateDocumentEvidence { .. } => "validate-document-evidence",
        Command::Compile { .. } => "compile",
        Command::Deduplicate { .. } => "deduplicate",
        Command::Prisma { .. } => "prisma",
        Command::VerifyAudit { .. } => "verify-audit",
        Command::ImportRecords { .. } => "import-records",
        Command::ExportRecords { .. } => "export-records",
        Command::StudyGraph { .. } => "study-graph",
        Command::ValidateSearch { .. } => "validate-search",
        Command::LivingDiff { .. } => "living-diff",
        Command::ValidateLivingLineage { .. } => "validate-living-lineage",
        Command::Provenance { .. } => "provenance",
        Command::Rank { .. } => "rank",
        Command::InspectContent { .. } => "inspect-content",
        Command::RenderDiagnostics { .. } => "render-diagnostics",
        Command::EvaluateGovernance { .. } => "evaluate-governance",
        Command::AuthoriseEndpoint { .. } => "authorise-endpoint",
        Command::ValidateAmendment { .. } => "validate-amendment",
        Command::ValidateStandardPack { .. } => "validate-standard-pack",
        Command::ValidateStandardAssessment { .. } => "validate-standard-assessment",
        Command::ValidateRankingCalibration { .. } => "validate-ranking-calibration",
        Command::VerifyWorkflowTrace { .. } => "verify-workflow-trace",
        Command::ValidateDiscoveryRun { .. } => "validate-discovery-run",
        Command::DiscoveryCandidates { .. } => "discovery-candidates",
        Command::VerifyProviderComponent { .. } => "verify-provider-component",
        Command::PlanLicensedRequest { .. } => "plan-licensed-request",
        Command::ValidateBenchmarkReport { .. } => "validate-benchmark-report",
        Command::Providers => "providers",
        Command::Workflow => "workflow",
        Command::Completions { .. } => "completions",
        Command::Manpage => "manpage",
        Command::Plan { .. }
        | Command::Source { .. }
        | Command::Strategy { .. }
        | Command::Run { .. }
        | Command::Import { .. }
        | Command::Screen { .. }
        | Command::Report { .. } => "command-dispatch",
    }
}

fn canonical_command(command: Command) -> Command {
    match command {
        Command::Plan { command } => match command {
            PlanCommand::Review {
                input,
                store,
                apply,
                confirmation,
            } => Command::PlanReview {
                input,
                store,
                apply,
                confirmation,
            },
            PlanCommand::Validate { input } => Command::ValidatePlan { input },
            PlanCommand::ValidateAmendment { input } => Command::ValidateAmendment { input },
            PlanCommand::EvaluateGovernance { policy, request } => {
                Command::EvaluateGovernance { policy, request }
            }
            PlanCommand::Workflow => Command::Workflow,
        },
        Command::Source { command } => match command {
            SourceCommand::List => Command::Providers,
            SourceCommand::AuthoriseEndpoint { input, endpoint } => {
                Command::AuthoriseEndpoint { input, endpoint }
            }
            SourceCommand::ValidateDiscoveryRun { input } => {
                Command::ValidateDiscoveryRun { input }
            }
            SourceCommand::DiscoveryCandidates { input } => Command::DiscoveryCandidates { input },
            SourceCommand::VerifyProviderComponent {
                manifest,
                component,
            } => Command::VerifyProviderComponent {
                manifest,
                component,
            },
            SourceCommand::PlanLicensedRequest {
                profile,
                strategy,
                endpoint,
            } => Command::PlanLicensedRequest {
                profile,
                strategy,
                endpoint,
            },
        },
        Command::Strategy { command } => match command {
            StrategyCommand::PressReview {
                input,
                store,
                apply,
                confirmation,
            } => Command::PressReviewStrategy {
                input,
                store,
                apply,
                confirmation,
            },
            StrategyCommand::Validate { input } => Command::ValidateStrategy { input },
            StrategyCommand::Compile { input, dialect } => Command::Compile { input, dialect },
            StrategyCommand::ValidateSearch { input } => Command::ValidateSearch { input },
            StrategyCommand::ValidateStandardPack { input } => {
                Command::ValidateStandardPack { input }
            }
            StrategyCommand::ValidateStandardAssessment { input } => {
                Command::ValidateStandardAssessment { input }
            }
            StrategyCommand::ValidateBenchmarkReport { input } => {
                Command::ValidateBenchmarkReport { input }
            }
        },
        Command::Run { command } => match command {
            RunCommand::Execute {
                request,
                envelope,
                provider_id,
                source_label,
                store,
                apply,
                commit_id,
                confirmed_by,
            } => Command::ExecuteSearch {
                request,
                envelope,
                provider_id,
                source_label,
                store,
                apply,
                commit_id,
                confirmed_by,
            },
            RunCommand::AuthoriseEndpoint { input, endpoint } => {
                Command::AuthoriseEndpoint { input, endpoint }
            }
            RunCommand::VerifyAudit { input } => Command::VerifyAudit { input },
            RunCommand::VerifyWorkflowTrace { input } => Command::VerifyWorkflowTrace { input },
            RunCommand::InspectContent {
                input,
                subject_id,
                policy,
            } => Command::InspectContent {
                input,
                subject_id,
                policy,
            },
            RunCommand::ValidateDocumentEvidence { input } => {
                Command::ValidateDocumentEvidence { input }
            }
        },
        Command::Import { command } => match command {
            ImportCommand::Records {
                input,
                format,
                source_receipt_id,
            } => Command::ImportRecords {
                input,
                format,
                source_receipt_id,
            },
            ImportCommand::ExportRecords {
                input,
                review_id,
                input_format,
                output_format,
            } => Command::ExportRecords {
                input,
                review_id,
                input_format,
                output_format,
            },
            ImportCommand::Deduplicate {
                input,
                title_threshold,
            } => Command::Deduplicate {
                input,
                title_threshold,
            },
        },
        Command::Screen { command } => match command {
            ScreenCommand::RecordDecision {
                policy,
                decision,
                store,
            } => Command::RecordScreeningDecision {
                policy,
                decision,
                store,
            },
            ScreenCommand::Rank { input, query_term } => Command::Rank { input, query_term },
            ScreenCommand::StudyGraph { input } => Command::StudyGraph { input },
            ScreenCommand::ValidateRankingCalibration { input } => {
                Command::ValidateRankingCalibration { input }
            }
            ScreenCommand::LivingDiff { previous, current } => {
                Command::LivingDiff { previous, current }
            }
            ScreenCommand::ValidateLivingLineage { input } => {
                Command::ValidateLivingLineage { input }
            }
        },
        Command::Report { command } => match command {
            ReportCommand::Prisma { input, format } => Command::Prisma { input, format },
            ReportCommand::Provenance { input } => Command::Provenance { input },
            ReportCommand::RenderDiagnostics { input, format } => {
                Command::RenderDiagnostics { input, format }
            }
        },
        command => command,
    }
}

fn execute(command: Command) -> Result<()> {
    match command {
        Command::Init { target, apply } => initialise(&target, apply)?,
        Command::PlanReview {
            input,
            store,
            apply,
            confirmation,
        } => {
            let plan: ReviewPlan = read_document(&input)?;
            if apply || confirmation.is_some() {
                bail!(
                    "CLI apply is disabled: consequential writes require a trusted host-injected authority verifier"
                );
            }
            let _ = store;
            print_json(&SearchrightEngine::plan_review(
                &plan,
                LocalReviewOperation::Preview,
            )?)?;
        }
        Command::PressReviewStrategy {
            input,
            store,
            apply,
            confirmation,
        } => {
            let review: PressReview = read_document(&input)?;
            if apply || confirmation.is_some() {
                bail!(
                    "CLI apply is disabled: consequential writes require a trusted host-injected authority verifier"
                );
            }
            let _ = store;
            print_json(&SearchrightEngine::press_review_strategy(
                &review,
                LocalReviewOperation::Preview,
            )?)?;
        }
        Command::ExecuteSearch {
            request,
            envelope,
            provider_id,
            source_label,
            store,
            apply,
            commit_id,
            confirmed_by,
        } => {
            let request: SearchRequest = read_document(&request)?;
            let envelope: ExecutionEnvelope = read_document(&envelope)?;
            let operation = if apply {
                bail!(
                    "CLI apply is disabled: consequential writes require a trusted host-injected authority verifier"
                );
            } else {
                if commit_id.is_some() || confirmed_by.is_some() {
                    bail!("preview must not carry --commit-id or --confirmed-by");
                }
                SearchExecutionOperation::Preview
            };
            let _ = store;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()?;
            print_json(&runtime.block_on(SearchrightEngine::execute_search(
                &provider_id,
                &source_label,
                request,
                &envelope,
                operation,
            ))?)?;
        }
        Command::RecordScreeningDecision {
            policy,
            decision,
            store,
        } => {
            let _ = (policy, decision, store);
            bail!(
                "CLI screening writes are disabled: consequential writes require a trusted host-injected authority verifier"
            );
        }
        Command::Plan { .. }
        | Command::Source { .. }
        | Command::Strategy { .. }
        | Command::Run { .. }
        | Command::Import { .. }
        | Command::Screen { .. }
        | Command::Report { .. } => unreachable!("grouped commands must be canonicalized"),
        Command::Completions { shell } => {
            io::stdout()
                .write_all(&completion_document(shell))
                .context("could not write the shell completion script")?;
        }
        Command::Manpage => {
            io::stdout()
                .write_all(&manpage_document()?)
                .context("could not write the searchright manual page")?;
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
            let endpoint = SearchrightEngine::authorise_endpoint(&envelope, &endpoint)?;
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
        let encoded = format!("{}\n", serde_json::to_string_pretty(&document)?);
        let open_result = OpenOptions::new().write(true).create_new(true).open(target);
        let context = if open_result
            .as_ref()
            .is_err_and(|error| error.kind() == io::ErrorKind::AlreadyExists)
        {
            "target already exists; choose a new path or remove it explicitly"
        } else {
            "configuration target could not be created; verify its parent and permissions"
        };
        let mut file = open_result.context(context)?;
        file.write_all(encoded.as_bytes())
            .context("could not write configuration")?;
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

fn emit_error(message: &str, exit_code: u8) -> ExitCode {
    let envelope = serde_json::json!({
        "schema_version": "org.searchright.cli-error.v1",
        "code": if exit_code == 2 { "cli.usage" } else { "cli.operation_failed" },
        "message": message,
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

fn emit_operation_error(stage: &str, error: &anyhow::Error) -> ExitCode {
    let is_io = error
        .chain()
        .any(<dyn std::error::Error + 'static>::is::<io::Error>);
    let json_category = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<serde_json::Error>())
        .map(serde_json::Error::classify);
    let is_syntax = matches!(
        json_category,
        Some(serde_json::error::Category::Syntax | serde_json::error::Category::Eof)
    ) || error
        .chain()
        .any(<dyn std::error::Error + 'static>::is::<serde_yaml::Error>);
    let is_document_contract = matches!(json_category, Some(serde_json::error::Category::Data));
    let (category, message, corrective_action) = if is_io {
        (
            "filesystem",
            "the operation could not safely read or create its filesystem input",
            "Verify the input path, parent directory, permissions, and no-overwrite state, then retry.",
        )
    } else if is_syntax {
        (
            "document_syntax",
            "the input document does not match the required JSON or YAML syntax",
            "Correct the input document syntax for the named stage, then retry.",
        )
    } else if is_document_contract {
        (
            "document_contract",
            "the input document is missing or misstates required contract fields",
            "Correct the document fields for the named contract stage, then retry.",
        )
    } else {
        (
            "contract_or_authority",
            "the operation failed a contract validation or authority policy check",
            "Review the input contract and authority policy for the named stage, then retry.",
        )
    };
    let envelope = serde_json::json!({
        "schema_version": "org.searchright.cli-error.v1",
        "code": format!("cli.{category}"),
        "stage": stage,
        "category": category,
        "message": message,
        "corrective_action": corrective_action,
        "partial_output_safe": false,
    });
    eprintln!("{envelope}");
    ExitCode::from(3)
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

fn completion_document(shell: CompletionShell) -> Vec<u8> {
    match shell {
        CompletionShell::Bash => include_bytes!("../assets/completions/searchright.bash").to_vec(),
        CompletionShell::Elvish => {
            include_bytes!("../assets/completions/searchright.elvish").to_vec()
        }
        CompletionShell::Fish => include_bytes!("../assets/completions/searchright.fish").to_vec(),
        CompletionShell::PowerShell => {
            include_bytes!("../assets/completions/searchright.powershell").to_vec()
        }
        CompletionShell::Zsh => include_bytes!("../assets/completions/searchright.zsh").to_vec(),
    }
}

fn manpage_document() -> Result<Vec<u8>> {
    let mut document = Vec::new();
    clap_mangen::Man::new(Cli::command())
        .render(&mut document)
        .context("could not render the searchright manual page")?;
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_command_hierarchy_is_stable() {
        let command = Cli::command();
        let names = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect::<Vec<_>>();
        for expected in [
            "init",
            "plan",
            "source",
            "strategy",
            "run",
            "import",
            "screen",
            "report",
            "completions",
            "manpage",
        ] {
            assert!(names.contains(&expected), "missing {expected} command");
        }
    }

    #[test]
    fn grouped_commands_cover_every_fixture_backed_operation() {
        let command = Cli::command();
        for (group, expected) in [
            (
                "plan",
                &[
                    "validate",
                    "validate-amendment",
                    "evaluate-governance",
                    "workflow",
                ][..],
            ),
            (
                "source",
                &[
                    "list",
                    "authorise-endpoint",
                    "validate-discovery-run",
                    "discovery-candidates",
                    "verify-provider-component",
                    "plan-licensed-request",
                ][..],
            ),
            (
                "strategy",
                &[
                    "validate",
                    "compile",
                    "validate-search",
                    "validate-standard-pack",
                    "validate-standard-assessment",
                    "validate-benchmark-report",
                ][..],
            ),
            (
                "run",
                &[
                    "authorise-endpoint",
                    "verify-audit",
                    "verify-workflow-trace",
                    "inspect-content",
                    "validate-document-evidence",
                ][..],
            ),
            ("import", &["records", "export-records", "deduplicate"][..]),
            (
                "screen",
                &[
                    "rank",
                    "study-graph",
                    "validate-ranking-calibration",
                    "living-diff",
                    "validate-living-lineage",
                ][..],
            ),
            (
                "report",
                &["prisma", "provenance", "render-diagnostics"][..],
            ),
        ] {
            let Some(parent) = command.find_subcommand(group) else {
                panic!("missing {group} group");
            };
            let names = parent
                .get_subcommands()
                .map(clap::Command::get_name)
                .collect::<Vec<_>>();
            for child in expected {
                assert!(names.contains(child), "missing {group} {child} command");
            }
        }
    }

    #[test]
    fn every_grouped_command_canonicalises_to_its_legacy_dispatch() -> Result<()> {
        let cases: &[(&[&str], &[&str])] = &[
            (
                &["plan", "validate", "input.json"],
                &["validate-plan", "input.json"],
            ),
            (
                &["plan", "validate-amendment", "input.json"],
                &["validate-amendment", "input.json"],
            ),
            (
                &["plan", "evaluate-governance", "policy.json", "request.json"],
                &["evaluate-governance", "policy.json", "request.json"],
            ),
            (&["plan", "workflow"], &["workflow"]),
            (&["source", "list"], &["providers"]),
            (
                &[
                    "source",
                    "authorise-endpoint",
                    "input.json",
                    "https://example.test",
                ],
                &["authorise-endpoint", "input.json", "https://example.test"],
            ),
            (
                &["source", "validate-discovery-run", "input.json"],
                &["validate-discovery-run", "input.json"],
            ),
            (
                &["source", "discovery-candidates", "input.json"],
                &["discovery-candidates", "input.json"],
            ),
            (
                &[
                    "source",
                    "verify-provider-component",
                    "manifest.json",
                    "component.wasm",
                ],
                &[
                    "verify-provider-component",
                    "manifest.json",
                    "component.wasm",
                ],
            ),
            (
                &[
                    "source",
                    "plan-licensed-request",
                    "profile.json",
                    "strategy.json",
                    "https://example.test",
                ],
                &[
                    "plan-licensed-request",
                    "profile.json",
                    "strategy.json",
                    "https://example.test",
                ],
            ),
            (
                &["strategy", "validate", "input.json"],
                &["validate-strategy", "input.json"],
            ),
            (
                &["strategy", "compile", "input.json", "--dialect", "pubmed"],
                &["compile", "input.json", "--dialect", "pubmed"],
            ),
            (
                &["strategy", "validate-search", "input.json"],
                &["validate-search", "input.json"],
            ),
            (
                &["strategy", "validate-standard-pack", "input.json"],
                &["validate-standard-pack", "input.json"],
            ),
            (
                &["strategy", "validate-standard-assessment", "input.json"],
                &["validate-standard-assessment", "input.json"],
            ),
            (
                &["strategy", "validate-benchmark-report", "input.json"],
                &["validate-benchmark-report", "input.json"],
            ),
            (
                &[
                    "run",
                    "authorise-endpoint",
                    "input.json",
                    "https://example.test",
                ],
                &["authorise-endpoint", "input.json", "https://example.test"],
            ),
            (
                &["run", "verify-audit", "input.jsonl"],
                &["verify-audit", "input.jsonl"],
            ),
            (
                &["run", "verify-workflow-trace", "input.json"],
                &["verify-workflow-trace", "input.json"],
            ),
            (
                &[
                    "run",
                    "inspect-content",
                    "input.txt",
                    "--subject-id",
                    "subject",
                ],
                &["inspect-content", "input.txt", "--subject-id", "subject"],
            ),
            (
                &["run", "validate-document-evidence", "input.json"],
                &["validate-document-evidence", "input.json"],
            ),
            (
                &[
                    "import",
                    "records",
                    "input.ris",
                    "--format",
                    "ris",
                    "--source-receipt-id",
                    "receipt",
                ],
                &[
                    "import-records",
                    "input.ris",
                    "--format",
                    "ris",
                    "--source-receipt-id",
                    "receipt",
                ],
            ),
            (
                &[
                    "import",
                    "export-records",
                    "input.json",
                    "--review-id",
                    "review",
                    "--output-format",
                    "ris",
                ],
                &[
                    "export-records",
                    "input.json",
                    "--review-id",
                    "review",
                    "--output-format",
                    "ris",
                ],
            ),
            (
                &["import", "deduplicate", "input.json"],
                &["deduplicate", "input.json"],
            ),
            (
                &["screen", "rank", "input.json", "--query-term", "term"],
                &["rank", "input.json", "--query-term", "term"],
            ),
            (
                &["screen", "study-graph", "input.json"],
                &["study-graph", "input.json"],
            ),
            (
                &["screen", "validate-ranking-calibration", "input.json"],
                &["validate-ranking-calibration", "input.json"],
            ),
            (
                &["screen", "living-diff", "previous.json", "current.json"],
                &["living-diff", "previous.json", "current.json"],
            ),
            (
                &["screen", "validate-living-lineage", "input.json"],
                &["validate-living-lineage", "input.json"],
            ),
            (
                &["report", "prisma", "input.json"],
                &["prisma", "input.json"],
            ),
            (
                &["report", "provenance", "input.json"],
                &["provenance", "input.json"],
            ),
            (
                &["report", "render-diagnostics", "input.json"],
                &["render-diagnostics", "input.json"],
            ),
        ];
        for (grouped, legacy) in cases {
            let grouped =
                Cli::try_parse_from(std::iter::once("searchright").chain(grouped.iter().copied()))?;
            let legacy =
                Cli::try_parse_from(std::iter::once("searchright").chain(legacy.iter().copied()))?;
            assert_eq!(
                format!("{:?}", canonical_command(grouped.command)),
                format!("{:?}", legacy.command),
            );
        }
        Ok(())
    }

    #[test]
    fn completion_and_manpage_documents_are_generated_without_writes() -> Result<()> {
        let completions = String::from_utf8(completion_document(CompletionShell::Bash))?;
        assert!(completions.contains("_searchright"));
        assert!(completions.contains("validate-plan"));

        let manpage = String::from_utf8(manpage_document()?)?;
        assert!(manpage.contains(".TH searchright 1"));
        assert!(manpage.contains("searchright\\-completions(1)"));
        Ok(())
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
        assert!(error.to_string().contains("target already exists"));
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
