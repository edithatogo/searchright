use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    ContractError, DISCOVERY_RUN_SCHEMA_VERSION, Validate, require_schema_version, require_text,
};

/// Discovery method used beyond the primary database search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Backward reference checking.
    BackwardCitation,
    /// Forward citation searching.
    ForwardCitation,
    /// Similar-article discovery.
    SimilarArticles,
    /// Trial-registry searching.
    TrialRegistry,
    /// Repository or preprint searching.
    Repository,
    /// Grey literature searching.
    GreyLiterature,
    /// Handsearching.
    Handsearch,
    /// Contact with investigators or organisations.
    Contact,
    /// Another declared method.
    Other(String),
}

/// Source family used by a supplementary-discovery method.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySourceKind {
    /// A prospective trial register.
    TrialRegister,
    /// A general-purpose or institutional research repository.
    Repository,
    /// A conference or proceedings source.
    Conference,
    /// A thesis or dissertation source.
    Thesis,
    /// A policy-document source.
    Policy,
    /// An organisational website.
    OrganisationalWebsite,
    /// A citation graph or citation index.
    CitationIndex,
    /// Direct contact with an investigator or organisation.
    Contact,
    /// A manually searched journal, proceedings series or collection.
    Handsearch,
}

/// How a source-specific discovery method is executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryAccessMode {
    /// Deterministic fixture-backed adapter; live execution remains separately authorised.
    FixtureAdapter,
    /// A documented manual method executed by a human.
    DocumentedManual,
}

/// Privacy-safe outcome of an investigator or organisation contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContactOutcome {
    /// No response was received by the recorded follow-up date.
    NoResponse,
    /// The recipient declined or could not provide information.
    Declined,
    /// A response was received but identified no candidate records.
    RespondedNoCandidates,
    /// A response identified one or more candidate records.
    RespondedWithCandidates,
}

/// One lawful, bounded source-specific discovery method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoverySourceMethod {
    /// Stable source identifier.
    pub source_id: String,
    /// Human-readable source name.
    pub source_name: String,
    /// Source family.
    pub source_kind: DiscoverySourceKind,
    /// Discovery method represented by this source.
    pub method: DiscoveryMethod,
    /// Fixture adapter or documented manual execution.
    pub access_mode: DiscoveryAccessMode,
    /// Reproducible, source-specific procedure or query template.
    pub procedure: String,
    /// Explicit coverage and access limitations.
    pub limitations: Vec<String>,
    /// Whether live execution needs a separate explicit opt-in.
    pub live_opt_in_required: bool,
}

/// Reproducible log for one manual, contact, or handsearch action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ManualDiscoveryLog {
    /// Stable log identifier.
    pub log_id: String,
    /// Source identifier from the method catalogue.
    pub source_id: String,
    /// Discovery method performed.
    pub method: DiscoveryMethod,
    /// Calendar date in `YYYY-MM-DD` form.
    pub conducted_on: String,
    /// Exact query, browse path, contact template, or handsearch range.
    pub exact_method_text: String,
    /// Role of the person executing the method; personal identity is excluded.
    pub operator_role: String,
    /// Structured source scope, such as years, jurisdictions, pages or installations.
    pub scope_details: Vec<String>,
    /// Total result count reported by the source, when applicable.
    pub total_results: Option<u64>,
    /// Number of results inspected.
    pub results_inspected: u64,
    /// Candidate identifiers released for human review.
    pub discovered_ids: Vec<String>,
    /// Structured contact outcome; required only for contact methods.
    pub contact_outcome: Option<ContactOutcome>,
    /// Last follow-up date for a contact, if one occurred.
    pub last_follow_up_on: Option<String>,
    /// Limitations observed during execution.
    pub limitations: Vec<String>,
}

/// Coverage risk assigned to one declared discovery source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryCoverageRisk {
    /// No material source-specific gap was identified in the bounded method.
    Low,
    /// A known limitation may omit relevant material.
    Moderate,
    /// Access or method limitations create a substantial omission risk.
    High,
    /// Risk cannot yet be assessed.
    Unknown,
}

/// Coverage and risk assessment for one source-specific method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryCoverageAssessment {
    /// Source identifier from the method catalogue.
    pub source_id: String,
    /// Whether the method was executed for this review.
    pub executed: bool,
    /// Assessed coverage risk.
    pub risk: DiscoveryCoverageRisk,
    /// Reasons supporting the risk assessment.
    pub rationale: Vec<String>,
}

impl Validate for DiscoverySourceMethod {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.source_id, "discovery_source.source_id")?;
        require_text(&self.source_name, "discovery_source.source_name")?;
        require_text(&self.procedure, "discovery_source.procedure")?;
        require_nonblank_unique(&self.limitations, "discovery_source.limitations")?;
        if matches!(self.access_mode, DiscoveryAccessMode::FixtureAdapter)
            && !self.live_opt_in_required
        {
            return Err(ContractError::Invariant(
                "fixture adapters must require a separate live opt-in".to_owned(),
            ));
        }
        let expected = match self.source_kind {
            DiscoverySourceKind::TrialRegister => {
                matches!(self.method, DiscoveryMethod::TrialRegistry)
            }
            DiscoverySourceKind::Repository => matches!(self.method, DiscoveryMethod::Repository),
            DiscoverySourceKind::Conference
            | DiscoverySourceKind::Thesis
            | DiscoverySourceKind::Policy
            | DiscoverySourceKind::OrganisationalWebsite => {
                matches!(self.method, DiscoveryMethod::GreyLiterature)
            }
            DiscoverySourceKind::CitationIndex => matches!(
                self.method,
                DiscoveryMethod::BackwardCitation | DiscoveryMethod::ForwardCitation
            ),
            DiscoverySourceKind::Contact => matches!(self.method, DiscoveryMethod::Contact),
            DiscoverySourceKind::Handsearch => matches!(self.method, DiscoveryMethod::Handsearch),
        };
        if !expected {
            return Err(ContractError::Invariant(
                "discovery source kind and method are inconsistent".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for ManualDiscoveryLog {
    fn validate(&self) -> Result<(), ContractError> {
        require_discovery_identifier(&self.log_id, "manual_discovery.log_id")?;
        require_discovery_identifier(&self.source_id, "manual_discovery.source_id")?;
        for identifier in &self.discovered_ids {
            require_discovery_identifier(identifier, "manual_discovery.discovered_ids[]")?;
        }
        require_text(
            &self.exact_method_text,
            "manual_discovery.exact_method_text",
        )?;
        require_text(&self.operator_role, "manual_discovery.operator_role")?;
        validate_date(&self.conducted_on)?;
        require_nonblank_unique(&self.scope_details, "manual_discovery.scope_details")?;
        require_nonblank_unique_if_present(
            &self.discovered_ids,
            "manual_discovery.discovered_ids",
        )?;
        require_nonblank_unique(&self.limitations, "manual_discovery.limitations")?;
        for value in std::iter::once(self.exact_method_text.as_str())
            .chain([self.log_id.as_str(), self.source_id.as_str()])
            .chain(self.discovered_ids.iter().map(String::as_str))
            .chain(std::iter::once(self.operator_role.as_str()))
            .chain(self.scope_details.iter().map(String::as_str))
            .chain(self.limitations.iter().map(String::as_str))
        {
            reject_sensitive_text(value)?;
        }
        if matches!(self.method, DiscoveryMethod::Contact) {
            let Some(outcome) = &self.contact_outcome else {
                return Err(ContractError::Invariant(
                    "contact logs require an outcome".to_owned(),
                ));
            };
            if self.total_results.is_some() {
                return Err(ContractError::Invariant(
                    "contact logs must not use source result counts".to_owned(),
                ));
            }
            let has_candidates = !self.discovered_ids.is_empty();
            if has_candidates != matches!(outcome, ContactOutcome::RespondedWithCandidates) {
                return Err(ContractError::Invariant(
                    "contact outcome must agree with discovered candidates".to_owned(),
                ));
            }
            if let Some(follow_up) = &self.last_follow_up_on {
                validate_date(follow_up)?;
                if follow_up < &self.conducted_on {
                    return Err(ContractError::Invariant(
                        "contact follow-up cannot precede contact".to_owned(),
                    ));
                }
            }
        } else {
            if self.contact_outcome.is_some() || self.last_follow_up_on.is_some() {
                return Err(ContractError::Invariant(
                    "only contact logs may record contact outcomes or follow-up".to_owned(),
                ));
            }
            let Some(total) = self.total_results else {
                return Err(ContractError::Invariant(
                    "non-contact logs require a total result count".to_owned(),
                ));
            };
            if self.results_inspected > total {
                return Err(ContractError::Invariant(
                    "results inspected cannot exceed total results".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

fn reject_sensitive_text(value: &str) -> Result<(), ContractError> {
    let lower = value.to_ascii_lowercase();
    let sensitive = [
        "@",
        "authorization:",
        "bearer ",
        "cookie:",
        "api_key",
        "apikey",
        "password",
        "token=",
    ];
    if value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        || sensitive.iter().any(|fragment| lower.contains(fragment))
    {
        return Err(ContractError::Invariant(
            "manual discovery text must exclude identities, credentials and control characters"
                .to_owned(),
        ));
    }
    Ok(())
}

impl Validate for DiscoveryCoverageAssessment {
    fn validate(&self) -> Result<(), ContractError> {
        require_discovery_identifier(&self.source_id, "discovery_coverage.source_id")?;
        reject_sensitive_text(&self.source_id)?;
        require_nonblank_unique(&self.rationale, "discovery_coverage.rationale")?;
        for rationale in &self.rationale {
            reject_sensitive_text(rationale)?;
        }
        if !self.executed && matches!(self.risk, DiscoveryCoverageRisk::Low) {
            return Err(ContractError::Invariant(
                "an unexecuted discovery source cannot have low coverage risk".to_owned(),
            ));
        }
        Ok(())
    }
}

fn require_nonblank_unique(values: &[String], field: &'static str) -> Result<(), ContractError> {
    if values.is_empty() {
        return Err(ContractError::EmptyCollection(field));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !unique.insert(value) {
            return Err(ContractError::Invariant(format!(
                "{field} entries must be unique"
            )));
        }
    }
    Ok(())
}

fn require_nonblank_unique_if_present(
    values: &[String],
    field: &'static str,
) -> Result<(), ContractError> {
    let mut unique = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !unique.insert(value) {
            return Err(ContractError::Invariant(format!(
                "{field} entries must be unique"
            )));
        }
    }
    Ok(())
}

fn validate_date(value: &str) -> Result<(), ContractError> {
    let bytes = value.as_bytes();
    let valid_shape = bytes.len() == 10
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit());
    let year = value.get(0..4).and_then(|part| part.parse::<u16>().ok());
    let month = value.get(5..7).and_then(|part| part.parse::<u8>().ok());
    let day = value.get(8..10).and_then(|part| part.parse::<u8>().ok());
    let valid_calendar_date = match (year, month, day) {
        (Some(year), Some(month), Some(day)) => {
            let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let maximum_day = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if leap_year => 29,
                2 => 28,
                _ => 0,
            };
            day > 0 && day <= maximum_day
        }
        _ => false,
    };
    if !valid_shape || !valid_calendar_date {
        return Err(ContractError::Invariant(
            "manual discovery date must use YYYY-MM-DD".to_owned(),
        ));
    }
    Ok(())
}

/// Evidence-bearing discovery edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryEdge {
    /// Stable edge identifier.
    pub edge_id: String,
    /// Seed record/report identifier.
    pub seed_id: String,
    /// Discovered identifier.
    pub discovered_id: String,
    /// Discovery method.
    pub method: DiscoveryMethod,
    /// Source/provider used.
    pub provider_id: String,
    /// Evidence receipt identifier.
    pub receipt_id: String,
}

/// One bounded supplementary-discovery run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryRun {
    /// Contract identifier.
    pub schema_version: String,
    /// Review identifier.
    pub review_id: String,
    /// Stable run identifier.
    pub run_id: String,
    /// Discovery method.
    pub method: DiscoveryMethod,
    /// Seed identifiers.
    pub seed_ids: Vec<String>,
    /// Discovered edges.
    #[serde(default)]
    pub edges: Vec<DiscoveryEdge>,
    /// Maximum depth used for graph traversal.
    pub maximum_depth: u8,
    /// Maximum records allowed.
    pub maximum_records: u64,
    /// Whether human review is required before adding records to screening.
    pub requires_human_release: bool,
}

/// Hard ceiling for graph traversal depth accepted by the neutral contract.
pub const MAX_DISCOVERY_DEPTH: u8 = 8;
/// Hard ceiling for records released from one discovery run.
pub const MAX_DISCOVERY_RECORDS: u64 = 100_000;
/// Hard ceiling for evidence edges accepted before traversal.
pub const MAX_DISCOVERY_EDGES: usize = 100_000;
/// Hard ceiling for seeds accepted before constructing traversal state.
pub const MAX_DISCOVERY_SEEDS: usize = 10_000;
/// Maximum UTF-8 bytes in each run or evidence identifier.
pub const MAX_DISCOVERY_IDENTIFIER_BYTES: usize = 512;
/// Maximum aggregate identifier bytes in one run, including repeated values.
pub const MAX_DISCOVERY_IDENTIFIER_TOTAL_BYTES: usize = 16 * 1024 * 1024;

fn require_discovery_identifier(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.len() > MAX_DISCOVERY_IDENTIFIER_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(ContractError::Invariant(format!(
            "{field} must be bounded, unpadded and free of control characters"
        )));
    }
    require_text(value, field)
}

fn count_identifier_bytes(total: &mut usize, value: &str) -> Result<(), ContractError> {
    *total = total.saturating_add(value.len());
    if *total > MAX_DISCOVERY_IDENTIFIER_TOTAL_BYTES {
        return Err(ContractError::Invariant(
            "discovery run exceeds the aggregate identifier byte ceiling".to_owned(),
        ));
    }
    Ok(())
}

impl Validate for DiscoveryEdge {
    fn validate(&self) -> Result<(), ContractError> {
        if let DiscoveryMethod::Other(label) = &self.method {
            require_discovery_identifier(label, "discovery.method")?;
        }
        require_discovery_identifier(&self.edge_id, "discovery.edge_id")?;
        require_discovery_identifier(&self.seed_id, "discovery.seed_id")?;
        require_discovery_identifier(&self.discovered_id, "discovery.discovered_id")?;
        require_discovery_identifier(&self.provider_id, "discovery.provider_id")?;
        require_discovery_identifier(&self.receipt_id, "discovery.receipt_id")?;
        if self.seed_id == self.discovered_id {
            return Err(ContractError::Invariant(
                "discovery edge must not point a seed to itself".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for DiscoveryRun {
    fn validate(&self) -> Result<(), ContractError> {
        if let DiscoveryMethod::Other(label) = &self.method {
            require_discovery_identifier(label, "discovery.method")?;
        }
        require_schema_version(
            &self.schema_version,
            DISCOVERY_RUN_SCHEMA_VERSION,
            "discovery.schema_version",
        )?;
        require_discovery_identifier(&self.review_id, "discovery.review_id")?;
        require_discovery_identifier(&self.run_id, "discovery.run_id")?;
        if self.seed_ids.is_empty() {
            return Err(ContractError::EmptyCollection("discovery.seed_ids"));
        }
        if self.seed_ids.len() > MAX_DISCOVERY_SEEDS || self.edges.len() > MAX_DISCOVERY_EDGES {
            return Err(ContractError::Invariant(
                "discovery run exceeds the hard resource ceiling".to_owned(),
            ));
        }
        let mut identifier_bytes = 0;
        count_identifier_bytes(&mut identifier_bytes, &self.review_id)?;
        count_identifier_bytes(&mut identifier_bytes, &self.run_id)?;
        let mut seed_ids = BTreeSet::new();
        for seed_id in &self.seed_ids {
            require_discovery_identifier(seed_id, "discovery.seed_ids[]")?;
            count_identifier_bytes(&mut identifier_bytes, seed_id)?;
            if !seed_ids.insert(seed_id) {
                return Err(ContractError::Invariant(
                    "discovery seed identifiers must be unique".to_owned(),
                ));
            }
        }
        if self.maximum_depth == 0 || self.maximum_records == 0 {
            return Err(ContractError::Invariant(
                "discovery budgets must be greater than zero".to_owned(),
            ));
        }
        if self.maximum_depth > MAX_DISCOVERY_DEPTH
            || self.maximum_records > MAX_DISCOVERY_RECORDS
            || self.edges.len() > MAX_DISCOVERY_EDGES
        {
            return Err(ContractError::Invariant(
                "discovery run exceeds the hard resource ceiling".to_owned(),
            ));
        }
        if !self.requires_human_release {
            return Err(ContractError::Invariant(
                "supplementary discovery requires human release before screening ingestion"
                    .to_owned(),
            ));
        }
        for edge in &self.edges {
            for identifier in [
                &edge.edge_id,
                &edge.seed_id,
                &edge.discovered_id,
                &edge.provider_id,
                &edge.receipt_id,
            ] {
                count_identifier_bytes(&mut identifier_bytes, identifier)?;
            }
            edge.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> DiscoveryRun {
        DiscoveryRun {
            schema_version: DISCOVERY_RUN_SCHEMA_VERSION.to_owned(),
            review_id: "review-1".to_owned(),
            run_id: "discovery-1".to_owned(),
            method: DiscoveryMethod::GreyLiterature,
            seed_ids: vec!["seed-1".to_owned()],
            edges: Vec::new(),
            maximum_depth: 1,
            maximum_records: 10,
            requires_human_release: true,
        }
    }

    #[test]
    fn discovery_requires_human_release() {
        let mut run = run();
        run.requires_human_release = false;

        assert!(
            matches!(run.validate(), Err(ContractError::Invariant(message)) if message.contains("human release"))
        );
    }

    #[test]
    fn discovery_rejects_duplicate_and_blank_seeds() {
        let mut duplicate = run();
        duplicate.seed_ids.push("seed-1".to_owned());
        assert!(
            matches!(duplicate.validate(), Err(ContractError::Invariant(message)) if message.contains("unique"))
        );

        let mut blank = run();
        blank.seed_ids = vec![" ".to_owned()];
        assert!(blank.validate().is_err());
    }

    #[test]
    fn manual_discovery_dates_are_calendar_valid() {
        let base = ManualDiscoveryLog {
            log_id: "log-1".to_owned(),
            source_id: "handsearch-log".to_owned(),
            method: DiscoveryMethod::Handsearch,
            conducted_on: "2024-02-29".to_owned(),
            exact_method_text: "Volume 1, issues 1-4, pages 1-200".to_owned(),
            operator_role: "information specialist".to_owned(),
            scope_details: vec!["Volume 1, issues 1-4".to_owned()],
            total_results: Some(200),
            results_inspected: 200,
            discovered_ids: Vec::new(),
            contact_outcome: None,
            last_follow_up_on: None,
            limitations: vec!["Supplement pages were unavailable".to_owned()],
        };
        assert!(base.validate().is_ok());

        for invalid in ["2023-02-29", "2026-13-01", "2026-04-31", "29-08-2026"] {
            let mut log = base.clone();
            log.conducted_on = invalid.to_owned();
            assert!(log.validate().is_err());
        }
    }

    #[test]
    fn coverage_and_adapter_authority_fail_closed() {
        let assessment = DiscoveryCoverageAssessment {
            source_id: "who-ictrp".to_owned(),
            executed: false,
            risk: DiscoveryCoverageRisk::Low,
            rationale: vec!["not searched".to_owned()],
        };
        assert!(assessment.validate().is_err());

        let method = DiscoverySourceMethod {
            source_id: "opencitations".to_owned(),
            source_name: "OpenCitations".to_owned(),
            source_kind: DiscoverySourceKind::CitationIndex,
            method: DiscoveryMethod::ForwardCitation,
            access_mode: DiscoveryAccessMode::FixtureAdapter,
            procedure: "fixture only".to_owned(),
            limitations: vec!["incomplete graph".to_owned()],
            live_opt_in_required: false,
        };
        assert!(method.validate().is_err());
    }

    #[test]
    fn discovery_run_hard_resource_ceilings_fail_closed() {
        let mut oversized = run();
        oversized.maximum_depth = MAX_DISCOVERY_DEPTH + 1;
        assert!(oversized.validate().is_err());
        oversized = run();
        oversized.maximum_records = MAX_DISCOVERY_RECORDS + 1;
        assert!(oversized.validate().is_err());
    }

    #[test]
    fn discovery_identifier_and_seed_budgets_are_enforced() {
        let mut bounded = run();
        bounded.seed_ids = vec!["α".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES / 2)];
        assert!(bounded.validate().is_ok());
        let Some(seed) = bounded.seed_ids.first_mut() else {
            panic!("fixture requires a seed");
        };
        seed.push('x');
        assert!(bounded.validate().is_err());
        for invalid in [" seed", "seed ", "seed\0id", "seed\u{7f}id"] {
            bounded = run();
            bounded.seed_ids = vec![invalid.to_owned()];
            assert!(bounded.validate().is_err());
        }
        bounded = run();
        bounded.seed_ids = (0..MAX_DISCOVERY_SEEDS)
            .map(|i| format!("seed-{i}"))
            .collect();
        assert!(bounded.validate().is_ok());
        bounded.seed_ids.push("extra".to_owned());
        assert!(
            matches!(bounded.validate(), Err(ContractError::Invariant(message)) if message.contains("hard resource ceiling"))
        );
        bounded = run();
        bounded.method = DiscoveryMethod::Other("x".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES + 1));
        assert!(bounded.validate().is_err());
    }

    #[test]
    fn discovery_edge_identifiers_and_total_bytes_are_bounded() {
        let mut evidence = DiscoveryEdge {
            edge_id: "e".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES),
            seed_id: "s".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES),
            discovered_id: "d".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES),
            method: DiscoveryMethod::GreyLiterature,
            provider_id: "p".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES),
            receipt_id: "r".repeat(MAX_DISCOVERY_IDENTIFIER_BYTES),
        };
        assert!(evidence.validate().is_ok());
        evidence.receipt_id.push('x');
        assert!(evidence.validate().is_err());
        evidence.receipt_id.pop();
        let mut bounded = run();
        bounded.edges =
            vec![
                evidence;
                MAX_DISCOVERY_IDENTIFIER_TOTAL_BYTES / (5 * MAX_DISCOVERY_IDENTIFIER_BYTES) + 1
            ];
        assert!(
            matches!(bounded.validate(), Err(ContractError::Invariant(message)) if message.contains("aggregate identifier byte ceiling"))
        );
        let mut total = MAX_DISCOVERY_IDENTIFIER_TOTAL_BYTES - 1;
        assert!(count_identifier_bytes(&mut total, "x").is_ok());
        assert!(count_identifier_bytes(&mut total, "x").is_err());
    }

    #[test]
    fn contact_logs_require_privacy_safe_consistent_outcomes() {
        let mut log = ManualDiscoveryLog {
            log_id: "contact-1".to_owned(),
            source_id: "contact-log".to_owned(),
            method: DiscoveryMethod::Contact,
            conducted_on: "2026-08-29".to_owned(),
            exact_method_text: "Approved template C1 requesting unpublished results".to_owned(),
            operator_role: "information specialist".to_owned(),
            scope_details: vec!["One investigator role for one seed report".to_owned()],
            total_results: None,
            results_inspected: 0,
            discovered_ids: Vec::new(),
            contact_outcome: Some(ContactOutcome::NoResponse),
            last_follow_up_on: None,
            limitations: vec!["Non-response leaves coverage unknown".to_owned()],
        };
        assert!(log.validate().is_ok());
        log.exact_method_text = "Contact person@example.org".to_owned();
        assert!(log.validate().is_err());
        log.exact_method_text = "Approved template C1 requesting unpublished results".to_owned();
        log.discovered_ids.push("candidate-1".to_owned());
        assert!(log.validate().is_err());
        log.contact_outcome = Some(ContactOutcome::RespondedWithCandidates);
        assert!(log.validate().is_ok());
        for field in ["log", "source", "candidate"] {
            let mut unsafe_log = log.clone();
            match field {
                "log" => unsafe_log.log_id = "person@example.org".to_owned(),
                "source" => unsafe_log.source_id = "person@example.org".to_owned(),
                _ => unsafe_log.discovered_ids = vec!["person@example.org".to_owned()],
            }
            assert!(unsafe_log.validate().is_err());
        }
        let assessment = DiscoveryCoverageAssessment {
            source_id: "contact-log".to_owned(),
            executed: true,
            risk: DiscoveryCoverageRisk::Unknown,
            rationale: vec!["Contact person@example.org".to_owned()],
        };
        assert!(assessment.validate().is_err());
    }
}
