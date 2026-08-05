#![forbid(unsafe_code)]
#![allow(
    clippy::print_stdout,
    reason = "the CLI must emit explicitly requested command output to stdout"
)]

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use evidence_search_core::{AuditLedger, QueryCompiler};
use searchright_agent::{AgentWorkflow, assess_plan_readiness};
use searchright_connectors::register_mvp_fixtures;
use searchright_contracts::{
    AuditEvent, BibliographicRecord, PrismaFlow, ReviewPlan, SearchDialect, SearchStrategy, Validate,
};
use searchright_dedup::{DedupConfig, Deduplicator};
use searchright_prisma::{build_prisma_s_ledger, render_mermaid, validate_flow};

#[derive(Debug, Parser)]
#[command(name = "searchright", version, about = "Contract-first systematic-search infrastructure")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a review plan and report readiness findings.
    ValidatePlan { input: PathBuf },
    /// Validate a source-specific search strategy.
    ValidateStrategy { input: PathBuf },
    /// Compile a portable strategy into source syntax.
    Compile {
        input: PathBuf,
        #[arg(long, value_enum)]
        dialect: DialectArg,
    },
    /// Deduplicate a JSON array of bibliographic records.
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
    /// List provider manifests available in the default no-network build.
    Providers,
    /// Print the conservative agent workflow policy.
    Workflow,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::ValidatePlan { input } => {
            let plan: ReviewPlan = read_document(&input)?;
            plan.validate().context("review plan is invalid")?;
            print_json(&serde_json::json!({
                "valid": true,
                "review_id": plan.review_id,
                "readiness_findings": assess_plan_readiness(&plan),
            }))?;
        }
        Command::ValidateStrategy { input } => {
            let strategy: SearchStrategy = read_document(&input)?;
            strategy.validate().context("search strategy is invalid")?;
            print_json(&serde_json::json!({"valid": true, "strategy_id": strategy.strategy_id}))?;
        }
        Command::Compile { input, dialect } => {
            let strategy: SearchStrategy = read_document(&input)?;
            print_json(&QueryCompiler::compile(&strategy, dialect.into())?)?;
        }
        Command::Deduplicate { input, title_threshold } => {
            if !(0.0..=1.0).contains(&title_threshold) {
                bail!("title-threshold must be between zero and one");
            }
            let records: Vec<BibliographicRecord> = read_document(&input)?;
            let result = Deduplicator::new(DedupConfig {
                title_similarity_threshold: title_threshold,
                ..DedupConfig::default()
            })?
            .cluster(&records)?;
            print_json(&result)?;
        }
        Command::Prisma { input, format } => {
            let flow: PrismaFlow = read_document(&input)?;
            validate_flow(&flow)?;
            match format {
                PrismaFormat::Json => print_json(&flow)?,
                PrismaFormat::Mermaid => println!("{}", render_mermaid(&flow)?),
                PrismaFormat::Ledger => print_json(&build_prisma_s_ledger(&flow)?)?,
            }
        }
        Command::VerifyAudit { input } => {
            let content = fs::read_to_string(&input)
                .with_context(|| format!("could not read {}", input.display()))?;
            let mut events = Vec::new();
            for (index, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                events.push(serde_json::from_str::<AuditEvent>(line).with_context(|| {
                    format!("invalid audit JSON at line {}", index.saturating_add(1))
                })?);
            }
            let verification = AuditLedger::from_events(events).verify()?;
            print_json(&serde_json::json!({
                "valid": true,
                "event_count": verification.event_count,
                "head_hash": verification.head_hash,
            }))?;
        }
        Command::Providers => {
            let mut registry = evidence_search_core::ProviderRegistry::new();
            register_mvp_fixtures(&mut registry)?;
            print_json(&registry.manifests())?;
        }
        Command::Workflow => print_json(&AgentWorkflow::systematic_search())?,
    }
    Ok(())
}

fn read_document<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path.display()))?;
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("yaml" | "yml") => serde_yaml::from_str(&content)
            .with_context(|| format!("invalid YAML in {}", path.display())),
        _ => serde_json::from_str(&content)
            .with_context(|| format!("invalid JSON in {}", path.display())),
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
