//! PRISMA arithmetic, flow rendering and PRISMA-S reporting evidence.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use searchright_contracts::{
    PRISMA_FLOW_SCHEMA_VERSION, PrismaFlow, PrismaSItem, PrismaSItemStatus, PrismaSLedger,
    SearchRun,
};

/// Evidence available for PRISMA-S reporting.
#[derive(Debug, Clone, Default)]
pub struct SearchReportingEvidence {
    /// Database/platform names were recorded.
    pub database_names: Vec<String>,
    /// Multi-database platform use was described or explicitly absent.
    pub multi_database_note: Option<String>,
    /// Registries searched.
    pub registries: Vec<String>,
    /// Browsing/online resource description.
    pub online_resources_note: Option<String>,
    /// Citation-search method.
    pub citation_search_note: Option<String>,
    /// Contact method.
    pub contacts_note: Option<String>,
    /// Other methods.
    pub other_methods_note: Option<String>,
    /// Full strategy artefact identifiers.
    pub full_strategy_evidence: Vec<String>,
    /// Limits/restrictions description.
    pub limits_note: Option<String>,
    /// Search filters and versions.
    pub filters_note: Option<String>,
    /// Reused prior work.
    pub prior_work_note: Option<String>,
    /// Update process.
    pub updates_note: Option<String>,
    /// Search dates evidence.
    pub search_date_evidence: Vec<String>,
    /// PRESS/peer-review evidence.
    pub peer_review_evidence: Vec<String>,
    /// Per-source result count evidence.
    pub total_record_evidence: Vec<String>,
    /// Deduplication process and count evidence.
    pub deduplication_evidence: Vec<String>,
}

/// Validate flow identifiers and arithmetic.
pub fn validate_flow(flow: &PrismaFlow) -> Result<(), PrismaError> {
    if flow.schema_version != PRISMA_FLOW_SCHEMA_VERSION {
        return Err(PrismaError::SchemaVersion {
            expected: PRISMA_FLOW_SCHEMA_VERSION,
            actual: flow.schema_version.clone(),
        });
    }
    require_text(&flow.review_id, "review_id")?;
    let identified = identified_total(flow)?;
    let removed = removed_total(flow)?;
    let expected_screened = identified
        .checked_sub(removed)
        .ok_or(PrismaError::RemovedExceedsIdentified)?;
    if expected_screened != flow.records_screened {
        return Err(PrismaError::CountMismatch {
            field: "records_screened",
            expected: expected_screened,
            actual: flow.records_screened,
        });
    }
    let expected_sought = flow
        .records_screened
        .checked_sub(flow.records_excluded)
        .ok_or(PrismaError::ExcludedExceedsScreened)?;
    if expected_sought != flow.reports_sought {
        return Err(PrismaError::CountMismatch {
            field: "reports_sought",
            expected: expected_sought,
            actual: flow.reports_sought,
        });
    }
    let expected_assessed = flow
        .reports_sought
        .checked_sub(flow.reports_not_retrieved)
        .ok_or(PrismaError::NotRetrievedExceedsSought)?;
    if expected_assessed != flow.reports_assessed {
        return Err(PrismaError::CountMismatch {
            field: "reports_assessed",
            expected: expected_assessed,
            actual: flow.reports_assessed,
        });
    }

    let mut reason_ids = BTreeSet::new();
    for reason in &flow.full_text_exclusions {
        require_text(&reason.reason_id, "full_text_exclusions.reason_id")?;
        require_text(&reason.label, "full_text_exclusions.label")?;
        if !reason_ids.insert(reason.reason_id.as_str()) {
            return Err(PrismaError::DuplicateExclusionReason(
                reason.reason_id.clone(),
            ));
        }
    }
    let full_text_excluded = full_text_exclusion_total(flow)?;
    let expected_assessed_from_outcomes = full_text_excluded
        .checked_add(flow.reports_included)
        .ok_or(PrismaError::CountOverflow {
            field: "reports_assessed_outcomes",
        })?;
    if expected_assessed_from_outcomes != flow.reports_assessed {
        return Err(PrismaError::CountMismatch {
            field: "reports_assessed_outcomes",
            expected: flow.reports_assessed,
            actual: expected_assessed_from_outcomes,
        });
    }
    if flow.studies_included > flow.reports_included {
        return Err(PrismaError::StudiesExceedReports);
    }
    Ok(())
}

/// Render a PRISMA-style flow diagram as Mermaid source.
pub fn mermaid_flow(flow: &PrismaFlow) -> Result<String, PrismaError> {
    validate_flow(flow)?;
    let identified = identified_total(flow)?;
    let exclusion_lines = if flow.full_text_exclusions.is_empty() {
        "No full-text exclusions".to_owned()
    } else {
        flow.full_text_exclusions
            .iter()
            .map(|reason| format!("{} (n = {})", escape_mermaid(&reason.label), reason.count))
            .collect::<Vec<_>>()
            .join("<br/>")
    };
    Ok(format!(
        r#"flowchart TB
  DB["Records identified from databases<br/>(n = {db})"]
  REG["Records identified from registers<br/>(n = {registers})"]
  OTHER["Records identified from other sources<br/>(n = {other})"]
  MERGE["Total records identified<br/>(n = {identified})"]
  REMOVE["Records removed before screening<br/>Duplicates: {duplicates}<br/>Automation: {automation}<br/>Other: {other_removed}"]
  SCREEN["Records screened<br/>(n = {screened})"]
  EXCLUDE["Records excluded<br/>(n = {records_excluded})"]
  SOUGHT["Reports sought for retrieval<br/>(n = {sought})"]
  NOTFOUND["Reports not retrieved<br/>(n = {not_retrieved})"]
  ASSESSED["Reports assessed for eligibility<br/>(n = {assessed})"]
  FTEX["Reports excluded<br/>{exclusion_lines}"]
  INCLUDED["Studies included in review<br/>(n = {studies})<br/>Reports of included studies: {reports}"]
  DB --> MERGE
  REG --> MERGE
  OTHER --> MERGE
  MERGE --> REMOVE --> SCREEN
  SCREEN --> EXCLUDE
  SCREEN --> SOUGHT
  SOUGHT --> NOTFOUND
  SOUGHT --> ASSESSED
  ASSESSED --> FTEX
  ASSESSED --> INCLUDED
"#,
        db = flow.records_databases,
        registers = flow.records_registers,
        other = flow.records_other,
        identified = identified,
        duplicates = flow.duplicates_removed,
        automation = flow.automation_removed,
        other_removed = flow.other_removed,
        screened = flow.records_screened,
        records_excluded = flow.records_excluded,
        sought = flow.reports_sought,
        not_retrieved = flow.reports_not_retrieved,
        assessed = flow.reports_assessed,
        studies = flow.studies_included,
        reports = flow.reports_included,
    ))
}

/// Render Mermaid using the public interface name shared by CLI and MCP.
pub fn render_mermaid(flow: &PrismaFlow) -> Result<String, PrismaError> {
    mermaid_flow(flow)
}

/// Build a conservative PRISMA-S ledger from flow evidence alone.
///
/// Only total-record and deduplication items can be evidenced by a flow contract;
/// all other items remain explicitly missing until run/strategy evidence is supplied.
pub fn build_prisma_s_ledger(flow: &PrismaFlow) -> Result<Vec<PrismaSLedger>, PrismaError> {
    validate_flow(flow)?;
    let total_identified = identified_total(flow)?;
    let removed = removed_total(flow)?;
    Ok(prisma_s_ledger(&SearchReportingEvidence {
        total_record_evidence: vec![format!(
            "PRISMA flow reports {total_identified} records identified"
        )],
        deduplication_evidence: vec![format!(
            "PRISMA flow reports {} duplicates and {removed} total pre-screen removals",
            flow.duplicates_removed
        )],
        ..SearchReportingEvidence::default()
    }))
}

/// Build the 16-item PRISMA-S ledger from evidence, marking missing items explicitly.
#[must_use]
pub fn prisma_s_ledger(evidence: &SearchReportingEvidence) -> Vec<PrismaSLedger> {
    vec![
        ledger(
            PrismaSItem::DatabaseName,
            !evidence.database_names.is_empty(),
            &evidence.database_names,
            "Name each database and platform.",
        ),
        optional_ledger(
            PrismaSItem::MultiDatabaseSearching,
            evidence.multi_database_note.as_deref(),
            "State whether databases were searched simultaneously on a platform.",
        ),
        optional_collection_ledger(
            PrismaSItem::StudyRegistries,
            &evidence.registries,
            "List registries or explain why none were used.",
        ),
        optional_ledger(
            PrismaSItem::OnlineResourcesAndBrowsing,
            evidence.online_resources_note.as_deref(),
            "Describe purposeful browsing or state not applicable.",
        ),
        optional_ledger(
            PrismaSItem::CitationSearching,
            evidence.citation_search_note.as_deref(),
            "Describe backward/forward citation searching or state not applicable.",
        ),
        optional_ledger(
            PrismaSItem::Contacts,
            evidence.contacts_note.as_deref(),
            "Describe contacts or state not applicable.",
        ),
        optional_ledger(
            PrismaSItem::OtherMethods,
            evidence.other_methods_note.as_deref(),
            "Describe other methods or state not applicable.",
        ),
        ledger(
            PrismaSItem::FullSearchStrategies,
            !evidence.full_strategy_evidence.is_empty(),
            &evidence.full_strategy_evidence,
            "Provide full strategies for every source.",
        ),
        optional_ledger(
            PrismaSItem::LimitsAndRestrictions,
            evidence.limits_note.as_deref(),
            "Report and justify limits/restrictions, including an explicit none.",
        ),
        optional_ledger(
            PrismaSItem::SearchFilters,
            evidence.filters_note.as_deref(),
            "Name and cite filters or state none.",
        ),
        optional_ledger(
            PrismaSItem::PriorWork,
            evidence.prior_work_note.as_deref(),
            "Describe reused prior strategies or state none.",
        ),
        optional_ledger(
            PrismaSItem::Updates,
            evidence.updates_note.as_deref(),
            "Describe updated searches or state not yet applicable.",
        ),
        ledger(
            PrismaSItem::DatesOfSearches,
            !evidence.search_date_evidence.is_empty(),
            &evidence.search_date_evidence,
            "Record the date each source was searched.",
        ),
        ledger(
            PrismaSItem::PeerReview,
            !evidence.peer_review_evidence.is_empty(),
            &evidence.peer_review_evidence,
            "Report search peer review or explicitly state none.",
        ),
        ledger(
            PrismaSItem::TotalRecords,
            !evidence.total_record_evidence.is_empty(),
            &evidence.total_record_evidence,
            "Report records from each source and overall.",
        ),
        ledger(
            PrismaSItem::Deduplication,
            !evidence.deduplication_evidence.is_empty(),
            &evidence.deduplication_evidence,
            "Describe deduplication process, software and counts.",
        ),
    ]
}

/// Derive common reporting evidence from a completed search run.
#[must_use]
pub fn evidence_from_run(run: &SearchRun) -> SearchReportingEvidence {
    SearchReportingEvidence {
        database_names: run
            .receipts
            .iter()
            .map(|receipt| receipt.source_label.clone())
            .collect(),
        search_date_evidence: run
            .receipts
            .iter()
            .map(|receipt| format!("{}:{}", receipt.receipt_id, receipt.executed_at))
            .collect(),
        total_record_evidence: run
            .receipts
            .iter()
            .map(|receipt| format!("{}:{}", receipt.receipt_id, receipt.records_retrieved))
            .collect(),
        ..SearchReportingEvidence::default()
    }
}

/// PRISMA arithmetic/reporting error.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PrismaError {
    /// The flow used a contract version unsupported by this crate.
    #[error("unsupported PRISMA schema version `{actual}`; expected `{expected}`")]
    SchemaVersion {
        expected: &'static str,
        actual: String,
    },
    /// A required identifier was empty.
    #[error("required PRISMA field `{field}` is empty")]
    EmptyField { field: &'static str },
    /// Addition overflowed the count representation.
    #[error("PRISMA count overflow while calculating `{field}`")]
    CountOverflow { field: &'static str },
    /// Pre-screen removals exceed identified records.
    #[error("records removed before screening exceed identified records")]
    RemovedExceedsIdentified,
    /// Title/abstract exclusions exceed screened records.
    #[error("record exclusions exceed screened records")]
    ExcludedExceedsScreened,
    /// Reports not retrieved exceed reports sought.
    #[error("reports not retrieved exceed reports sought")]
    NotRetrievedExceedsSought,
    /// Cross-field count mismatch.
    #[error("{field} count mismatch: expected {expected}, found {actual}")]
    CountMismatch {
        field: &'static str,
        expected: u64,
        actual: u64,
    },
    /// The same full-text reason identifier appeared twice.
    #[error("duplicate full-text exclusion reason `{0}`")]
    DuplicateExclusionReason(String),
    /// More studies than reports were recorded.
    #[error("included studies cannot exceed reports of included studies")]
    StudiesExceedReports,
}

fn identified_total(flow: &PrismaFlow) -> Result<u64, PrismaError> {
    checked_sum(
        [
            flow.records_databases,
            flow.records_registers,
            flow.records_other,
        ],
        "records_identified",
    )
}

fn removed_total(flow: &PrismaFlow) -> Result<u64, PrismaError> {
    checked_sum(
        [
            flow.duplicates_removed,
            flow.automation_removed,
            flow.other_removed,
        ],
        "records_removed",
    )
}

fn full_text_exclusion_total(flow: &PrismaFlow) -> Result<u64, PrismaError> {
    flow.full_text_exclusions
        .iter()
        .try_fold(0_u64, |total, reason| {
            total
                .checked_add(reason.count)
                .ok_or(PrismaError::CountOverflow {
                    field: "full_text_exclusions",
                })
        })
}

fn checked_sum<const N: usize>(values: [u64; N], field: &'static str) -> Result<u64, PrismaError> {
    values.into_iter().try_fold(0_u64, |total, value| {
        total
            .checked_add(value)
            .ok_or(PrismaError::CountOverflow { field })
    })
}

fn require_text(value: &str, field: &'static str) -> Result<(), PrismaError> {
    if value.trim().is_empty() {
        Err(PrismaError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn ledger(
    item: PrismaSItem,
    complete: bool,
    evidence: &[String],
    missing_note: &str,
) -> PrismaSLedger {
    PrismaSLedger {
        item,
        status: if complete {
            PrismaSItemStatus::Complete
        } else {
            PrismaSItemStatus::Missing
        },
        evidence: evidence.to_vec(),
        note: if complete {
            "Evidence linked.".to_owned()
        } else {
            missing_note.to_owned()
        },
    }
}

fn optional_collection_ledger(item: PrismaSItem, values: &[String], note: &str) -> PrismaSLedger {
    if values.is_empty() {
        PrismaSLedger {
            item,
            status: PrismaSItemStatus::Missing,
            evidence: Vec::new(),
            note: note.to_owned(),
        }
    } else {
        PrismaSLedger {
            item,
            status: PrismaSItemStatus::Complete,
            evidence: values.to_vec(),
            note: "Evidence linked.".to_owned(),
        }
    }
}

fn optional_ledger(item: PrismaSItem, value: Option<&str>, note: &str) -> PrismaSLedger {
    match value {
        Some(value)
            if value.trim().eq_ignore_ascii_case("not applicable")
                || value.trim().eq_ignore_ascii_case("none") =>
        {
            PrismaSLedger {
                item,
                status: PrismaSItemStatus::NotApplicable,
                evidence: vec![value.to_owned()],
                note: "Explicitly reported as not applicable/none.".to_owned(),
            }
        }
        Some(value) if !value.trim().is_empty() => PrismaSLedger {
            item,
            status: PrismaSItemStatus::Complete,
            evidence: vec![value.to_owned()],
            note: "Evidence linked.".to_owned(),
        },
        _ => PrismaSLedger {
            item,
            status: PrismaSItemStatus::Missing,
            evidence: Vec::new(),
            note: note.to_owned(),
        },
    }
}

fn escape_mermaid(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "'")
        .replace('\r', " ")
        .replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use searchright_contracts::ExclusionCount;

    use super::*;

    fn valid_flow() -> PrismaFlow {
        PrismaFlow {
            schema_version: "org.searchright.prisma-flow.v1".to_owned(),
            review_id: "r1".to_owned(),
            records_databases: 100,
            records_registers: 10,
            records_other: 0,
            duplicates_removed: 10,
            automation_removed: 0,
            other_removed: 0,
            records_screened: 100,
            records_excluded: 80,
            reports_sought: 20,
            reports_not_retrieved: 2,
            reports_assessed: 18,
            full_text_exclusions: vec![ExclusionCount {
                reason_id: "wrong".to_owned(),
                label: "Wrong population".to_owned(),
                count: 13,
            }],
            studies_included: 4,
            reports_included: 5,
        }
    }

    #[test]
    fn arithmetic_and_mermaid_validate() {
        assert!(validate_flow(&valid_flow()).is_ok());
        let diagram = mermaid_flow(&valid_flow());
        assert!(diagram.is_ok());
        if let Ok(diagram) = diagram {
            assert!(diagram.contains("flowchart TB"));
        }
    }

    #[test]
    fn mismatch_is_rejected() {
        let mut flow = valid_flow();
        flow.records_screened = 99;
        assert!(matches!(
            validate_flow(&flow),
            Err(PrismaError::CountMismatch {
                field: "records_screened",
                ..
            })
        ));
    }

    #[test]
    fn count_overflow_is_rejected() {
        let mut flow = valid_flow();
        flow.records_databases = u64::MAX;
        flow.records_registers = 1;
        assert!(matches!(
            validate_flow(&flow),
            Err(PrismaError::CountOverflow {
                field: "records_identified"
            })
        ));
    }
}
