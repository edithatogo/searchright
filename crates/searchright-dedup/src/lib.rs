//! Deterministic duplicate candidate generation and clustering.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use searchright_contracts::BibliographicRecord;
use serde::{Deserialize, Serialize};

/// Deduplication thresholds and matching policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DedupConfig {
    /// Jaccard title threshold after deterministic normalisation.
    pub title_similarity_threshold: f64,
    /// Maximum publication-year difference for fuzzy title matching.
    pub year_tolerance: i32,
    /// Require compatible first-author surnames for fuzzy matching when available.
    pub require_first_author_match: bool,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            title_similarity_threshold: 0.92,
            year_tolerance: 1,
            require_first_author_match: true,
        }
    }
}

/// Why two records were considered duplicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MatchEvidence {
    /// First record identifier.
    pub left_record_id: String,
    /// Second record identifier.
    pub right_record_id: String,
    /// Stable reason code.
    pub reason: String,
    /// Match score between zero and one.
    pub score: f64,
    /// Fields supporting or weakening the match.
    #[serde(default)]
    pub details: BTreeMap<String, String>,
    /// Fuzzy matches require review before destructive removal.
    pub review_required: bool,
}

/// One connected component of duplicate records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateCluster {
    /// Deterministic cluster identifier.
    pub cluster_id: String,
    /// Record selected as representative without deleting the others.
    pub representative_record_id: String,
    /// All records in the cluster.
    pub record_ids: Vec<String>,
    /// Pairwise evidence that joined the cluster.
    pub evidence: Vec<MatchEvidence>,
}

/// Complete deduplication output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DedupResult {
    /// Duplicate clusters with at least two records.
    pub clusters: Vec<DuplicateCluster>,
    /// Representatives plus singleton records.
    pub retained_record_ids: Vec<String>,
    /// Number of pairwise comparisons performed.
    pub comparisons: u64,
    /// Records that would be removed if every cluster were applied.
    pub proposed_duplicate_count: u64,
}

/// Deterministic pairwise deduplicator.
#[derive(Debug, Clone)]
pub struct Deduplicator {
    config: DedupConfig,
}

#[allow(
    clippy::indexing_slicing,
    reason = "pairwise indices are generated exclusively from the bounded input record slice"
)]
impl Deduplicator {
    /// Create a deduplicator with explicit, validated policy.
    pub fn new(config: DedupConfig) -> Result<Self, DedupError> {
        if !config.title_similarity_threshold.is_finite()
            || !(0.0..=1.0).contains(&config.title_similarity_threshold)
        {
            return Err(DedupError::InvalidTitleThreshold);
        }
        if config.year_tolerance < 0 {
            return Err(DedupError::InvalidYearTolerance);
        }
        Ok(Self { config })
    }

    /// Cluster duplicate candidates while retaining all source records and evidence.
    #[allow(
        clippy::indexing_slicing,
        reason = "pairwise and component indices are generated within the validated records slice"
    )]
    pub fn cluster(&self, records: &[BibliographicRecord]) -> Result<DedupResult, DedupError> {
        validate_records(records)?;
        let mut union_find = UnionFind::new(records.len());
        let mut evidence = Vec::new();
        let mut comparisons = 0_u64;

        for left in 0..records.len() {
            for right in (left + 1)..records.len() {
                comparisons = comparisons
                    .checked_add(1)
                    .ok_or(DedupError::CountOverflow("comparisons"))?;
                if let Some(match_evidence) = self.compare(&records[left], &records[right]) {
                    union_find.union(left, right);
                    evidence.push(match_evidence);
                }
            }
        }

        let mut components: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for index in 0..records.len() {
            let root = union_find.find(index);
            components.entry(root).or_default().push(index);
        }

        let mut clusters = Vec::new();
        let mut retained = BTreeSet::new();
        let mut proposed_duplicate_count = 0_u64;
        for indices in components.values() {
            if indices.len() == 1 {
                retained.insert(records[indices[0]].record_id.clone());
                continue;
            }
            let representative_index = choose_representative(indices, records);
            let representative = records[representative_index].record_id.clone();
            retained.insert(representative.clone());
            proposed_duplicate_count = proposed_duplicate_count
                .checked_add(usize_to_u64(indices.len().saturating_sub(1)))
                .ok_or(DedupError::CountOverflow("proposed_duplicate_count"))?;
            let mut record_ids: Vec<String> = indices
                .iter()
                .map(|index| records[*index].record_id.clone())
                .collect();
            record_ids.sort();
            let record_set: BTreeSet<&str> = record_ids.iter().map(String::as_str).collect();
            let mut cluster_evidence: Vec<MatchEvidence> = evidence
                .iter()
                .filter(|item| {
                    record_set.contains(item.left_record_id.as_str())
                        && record_set.contains(item.right_record_id.as_str())
                })
                .cloned()
                .collect();
            cluster_evidence.sort_by(|left, right| {
                left.left_record_id
                    .cmp(&right.left_record_id)
                    .then_with(|| left.right_record_id.cmp(&right.right_record_id))
            });
            let cluster_material = record_ids.join("\n");
            let cluster_digest = blake3::hash(cluster_material.as_bytes())
                .to_hex()
                .to_string();
            let cluster_suffix: String = cluster_digest.chars().take(16).collect();
            clusters.push(DuplicateCluster {
                cluster_id: format!("dup-{cluster_suffix}"),
                representative_record_id: representative,
                record_ids,
                evidence: cluster_evidence,
            });
        }
        clusters.sort_by(|left, right| left.cluster_id.cmp(&right.cluster_id));

        Ok(DedupResult {
            clusters,
            retained_record_ids: retained.into_iter().collect(),
            comparisons,
            proposed_duplicate_count,
        })
    }

    fn compare(
        &self,
        left: &BibliographicRecord,
        right: &BibliographicRecord,
    ) -> Option<MatchEvidence> {
        if let Some(reason) = exact_identifier_reason(left, right) {
            return Some(MatchEvidence {
                left_record_id: left.record_id.clone(),
                right_record_id: right.record_id.clone(),
                reason: reason.to_owned(),
                score: 1.0,
                details: BTreeMap::new(),
                review_required: false,
            });
        }

        let title_score = jaccard_similarity(
            &normalise_tokens(&left.title),
            &normalise_tokens(&right.title),
        );
        if title_score < self.config.title_similarity_threshold {
            return None;
        }
        if !years_compatible(
            left.publication_year,
            right.publication_year,
            self.config.year_tolerance,
        ) {
            return None;
        }
        let author_match = first_authors_compatible(left, right);
        if self.config.require_first_author_match && !author_match {
            return None;
        }

        let mut details = BTreeMap::new();
        details.insert("title_similarity".to_owned(), format!("{title_score:.4}"));
        details.insert(
            "first_author_compatible".to_owned(),
            author_match.to_string(),
        );
        details.insert(
            "publication_years".to_owned(),
            format!("{:?}/{:?}", left.publication_year, right.publication_year),
        );
        Some(MatchEvidence {
            left_record_id: left.record_id.clone(),
            right_record_id: right.record_id.clone(),
            reason: "fuzzy_title_author_year".to_owned(),
            score: title_score,
            details,
            review_required: true,
        })
    }
}

fn exact_identifier_reason(
    left: &BibliographicRecord,
    right: &BibliographicRecord,
) -> Option<&'static str> {
    if equal_normalised_valid(
        left.identifiers.doi.as_deref(),
        right.identifiers.doi.as_deref(),
        normalise_doi,
        valid_doi,
    ) {
        return Some("exact_doi");
    }
    if equal_normalised_valid(
        left.identifiers.pmid.as_deref(),
        right.identifiers.pmid.as_deref(),
        normalise_trimmed,
        valid_pmid,
    ) {
        return Some("exact_pmid");
    }
    if equal_trimmed(
        left.identifiers.trial_registration.as_deref(),
        right.identifiers.trial_registration.as_deref(),
    ) {
        return Some("exact_trial_registration");
    }
    None
}

fn equal_normalised_valid(
    left: Option<&str>,
    right: Option<&str>,
    normaliser: fn(&str) -> String,
    validator: fn(&str) -> bool,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left = normaliser(left);
            validator(&left) && left == normaliser(right)
        }
        _ => false,
    }
}

fn normalise_trimmed(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn valid_doi(value: &str) -> bool {
    value.starts_with("10.") && value.contains('/') && !value.chars().any(char::is_whitespace)
}

fn valid_pmid(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

fn equal_trimmed(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left = left.trim().to_ascii_lowercase();
            !left.is_empty() && left == right.trim().to_ascii_lowercase()
        }
        _ => false,
    }
}

/// Normalise DOI resolver prefixes and case.
#[must_use]
pub fn normalise_doi(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("https://dx.doi.org/")
        .trim_start_matches("http://dx.doi.org/")
        .trim_start_matches("doi.org/")
        .trim_start_matches("doi:")
        .trim()
        .to_owned()
}

fn normalise_tokens(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

#[allow(
    clippy::float_cmp,
    reason = "the union value is an exactly represented non-negative integer count projected to f64"
)]
fn jaccard_similarity(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = usize_to_f64(left.intersection(right).count());
    let union = usize_to_f64(left.union(right).count());
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn years_compatible(left: Option<i32>, right: Option<i32>, tolerance: i32) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            (i64::from(left) - i64::from(right)).abs() <= i64::from(tolerance)
        }
        _ => true,
    }
}

fn first_authors_compatible(left: &BibliographicRecord, right: &BibliographicRecord) -> bool {
    match (left.authors.first(), right.authors.first()) {
        (Some(left), Some(right)) => author_key(left) == author_key(right),
        _ => true,
    }
}

fn author_key(value: &str) -> String {
    let trimmed = value.trim();
    if let Some((surname, _)) = trimmed.split_once(',') {
        return surname.trim().to_ascii_lowercase();
    }
    trimmed
        .split_whitespace()
        .next_back()
        .map_or_else(String::new, str::to_ascii_lowercase)
}

#[allow(
    clippy::indexing_slicing,
    reason = "indices originate from non-empty connected components over the records slice"
)]
fn choose_representative(indices: &[usize], records: &[BibliographicRecord]) -> usize {
    let mut best = indices[0];
    for index in indices.iter().copied().skip(1) {
        let score = metadata_score(&records[index]);
        let best_score = metadata_score(&records[best]);
        if score > best_score
            || (score == best_score && records[index].record_id < records[best].record_id)
        {
            best = index;
        }
    }
    best
}

fn metadata_score(record: &BibliographicRecord) -> u16 {
    let mut score = 0_u16;
    score += u16::from(record.identifiers.doi.is_some()) * 4;
    score += u16::from(record.identifiers.pmid.is_some()) * 4;
    score += u16::from(record.abstract_text.is_some()) * 3;
    score += u16::from(!record.authors.is_empty()) * 2;
    score += u16::from(record.publication_year.is_some());
    score += u16::from(record.container_title.is_some());
    score
}

/// Deduplication configuration or input error.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DedupError {
    /// Title threshold was NaN, infinite or outside zero to one.
    #[error("title similarity threshold must be finite and between zero and one")]
    InvalidTitleThreshold,
    /// Publication-year tolerance was negative.
    #[error("publication-year tolerance must be zero or greater")]
    InvalidYearTolerance,
    /// A record identifier was empty.
    #[error("record identifier must not be empty")]
    EmptyRecordId,
    /// Two input records used the same Searchright identifier.
    #[error("duplicate input record identifier `{0}`")]
    DuplicateRecordId(String),
    /// A count exceeded the persisted representation.
    #[error("deduplication count overflow while calculating `{0}`")]
    CountOverflow(&'static str),
}

fn validate_records(records: &[BibliographicRecord]) -> Result<(), DedupError> {
    let mut identifiers = BTreeSet::new();
    for record in records {
        if record.record_id.trim().is_empty() {
            return Err(DedupError::EmptyRecordId);
        }
        if !identifiers.insert(record.record_id.as_str()) {
            return Err(DedupError::DuplicateRecordId(record.record_id.clone()));
        }
    }
    Ok(())
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

#[derive(Debug)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

#[allow(
    clippy::indexing_slicing,
    reason = "union-find indices are constructed from and bounded by the parent vector length"
)]
impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, item: usize) -> usize {
        if self.parent[item] != item {
            let root = self.find(self.parent[item]);
            self.parent[item] = root;
        }
        self.parent[item]
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return;
        }
        match self.rank[left_root].cmp(&self.rank[right_root]) {
            std::cmp::Ordering::Less => self.parent[left_root] = right_root,
            std::cmp::Ordering::Greater => self.parent[right_root] = left_root,
            std::cmp::Ordering::Equal => {
                self.parent[right_root] = left_root;
                self.rank[left_root] = self.rank[left_root].saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{RecordIdentifiers, RecordKind};
    use serde_json::Value;

    use super::*;

    fn record(id: &str, doi: Option<&str>, title: &str) -> BibliographicRecord {
        BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: id.to_owned(),
            source_receipt_id: "receipt".to_owned(),
            native_id: id.to_owned(),
            kind: RecordKind::JournalArticle,
            identifiers: RecordIdentifiers {
                doi: doi.map(str::to_owned),
                ..RecordIdentifiers::default()
            },
            title: title.to_owned(),
            abstract_text: None,
            authors: vec!["Mordaunt, Dylan".to_owned()],
            container_title: None,
            publication_year: Some(2026),
            publication_date: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            urls: Vec::new(),
            provider_metadata: Value::Null,
        }
    }

    #[test]
    fn exact_doi_clusters_records() {
        let records = vec![
            record("a", Some("https://doi.org/10.1/ABC"), "First title"),
            record("b", Some("doi:10.1/abc"), "Different title"),
            record("c", None, "Unrelated"),
        ];
        let deduplicator = Deduplicator::new(DedupConfig::default());
        assert!(deduplicator.is_ok());
        if let Ok(deduplicator) = deduplicator {
            let result = deduplicator.cluster(&records);
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert_eq!(result.clusters.len(), 1);
                assert_eq!(result.proposed_duplicate_count, 1);
                assert_eq!(result.retained_record_ids.len(), 2);
            }
        }
    }

    proptest::proptest! {
        #[test]
        fn doi_normalisation_is_idempotent(input in "[A-Za-z0-9./:_-]{1,80}") {
            let once = normalise_doi(&input);
            let twice = normalise_doi(&once);
            proptest::prop_assert_eq!(once, twice);
        }
    }
}
