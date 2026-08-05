use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Count for one full-text exclusion reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExclusionCount {
    /// Stable reason identifier.
    pub reason_id: String,
    /// Human-readable label.
    pub label: String,
    /// Excluded reports or studies.
    pub count: u64,
}

/// PRISMA 2020 flow counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrismaFlow {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Records identified from databases.
    pub records_databases: u64,
    /// Records identified from registers.
    pub records_registers: u64,
    /// Records identified from other sources.
    pub records_other: u64,
    /// Duplicate records removed.
    pub duplicates_removed: u64,
    /// Records removed by automation before screening.
    pub automation_removed: u64,
    /// Records removed for other pre-screening reasons.
    pub other_removed: u64,
    /// Records screened at title/abstract.
    pub records_screened: u64,
    /// Records excluded at title/abstract.
    pub records_excluded: u64,
    /// Reports sought for retrieval.
    pub reports_sought: u64,
    /// Reports not retrieved.
    pub reports_not_retrieved: u64,
    /// Reports assessed for eligibility.
    pub reports_assessed: u64,
    /// Full-text exclusions by reason.
    #[serde(default)]
    pub full_text_exclusions: Vec<ExclusionCount>,
    /// Studies included.
    pub studies_included: u64,
    /// Reports of included studies.
    pub reports_included: u64,
}

/// PRISMA-S checklist item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrismaSItem {
    /// Item 1.
    DatabaseName,
    /// Item 2.
    MultiDatabaseSearching,
    /// Item 3.
    StudyRegistries,
    /// Item 4.
    OnlineResourcesAndBrowsing,
    /// Item 5.
    CitationSearching,
    /// Item 6.
    Contacts,
    /// Item 7.
    OtherMethods,
    /// Item 8.
    FullSearchStrategies,
    /// Item 9.
    LimitsAndRestrictions,
    /// Item 10.
    SearchFilters,
    /// Item 11.
    PriorWork,
    /// Item 12.
    Updates,
    /// Item 13.
    DatesOfSearches,
    /// Item 14.
    PeerReview,
    /// Item 15.
    TotalRecords,
    /// Item 16.
    Deduplication,
}

/// Completion status for a reporting item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrismaSItemStatus {
    /// Complete and evidence-linked.
    Complete,
    /// Partly complete.
    Partial,
    /// Required but missing.
    Missing,
    /// Not applicable with rationale.
    NotApplicable,
}

/// Evidence for one PRISMA-S item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrismaSLedger {
    /// Checklist item.
    pub item: PrismaSItem,
    /// Status.
    pub status: PrismaSItemStatus,
    /// Evidence links or audit event identifiers.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// Explanation or not-applicable rationale.
    pub note: String,
}
