//! Bounded citation-chasing and supplementary-discovery operations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use schemars::JsonSchema;
use searchright_contracts::{
    DiscoveryCoverageAssessment, DiscoveryEdge, DiscoveryMethod, DiscoveryRun,
    DiscoverySourceMethod, MAX_DISCOVERY_IDENTIFIER_BYTES, ManualDiscoveryLog, Validate,
};
use serde::{Deserialize, Serialize};

const SOURCE_METHOD_FIXTURE: &str =
    include_str!("../../../contracts/fixtures/discovery-source-methods.json");
const MAX_CITATION_FIXTURE_BYTES: usize = 1024 * 1024;
const MAX_CITATION_ROWS: usize = 10_000;
const MAX_CITATION_IDENTIFIER_BYTES: usize = MAX_DISCOVERY_IDENTIFIER_BYTES;
const MAX_DISCOVERY_WORK: usize = 1_000_000;
const MAX_DISCOVERY_EVIDENCE: usize = 100_000;

/// One candidate released from a bounded discovery graph for human review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveredCandidate {
    /// Candidate identifier.
    pub discovered_id: String,
    /// Minimum graph depth from any declared seed.
    pub depth: u8,
    /// Evidence edges supporting discovery.
    pub edge_ids: Vec<String>,
    /// Human release is required before screening ingestion.
    pub requires_human_release: bool,
}

/// Load the versioned, source-specific supplementary-discovery method fixture.
pub fn source_method_catalog() -> Result<Vec<DiscoverySourceMethod>, DiscoveryError> {
    let methods: Vec<DiscoverySourceMethod> = serde_json::from_str(SOURCE_METHOD_FIXTURE)?;
    let mut source_ids = BTreeSet::new();
    for method in &methods {
        method.validate()?;
        if !source_ids.insert(method.source_id.as_str()) {
            return Err(DiscoveryError::DuplicateSource(method.source_id.clone()));
        }
    }
    Ok(methods)
}

/// Validate manual-method logs and a complete source coverage/risk matrix.
pub fn validate_method_reporting(
    methods: &[DiscoverySourceMethod],
    logs: &[ManualDiscoveryLog],
    coverage: &[DiscoveryCoverageAssessment],
) -> Result<(), DiscoveryError> {
    let canonical = source_method_catalog()?;
    let canonical_by_source = canonical
        .iter()
        .map(|method| (method.source_id.as_str(), method))
        .collect::<BTreeMap<_, _>>();
    let mut method_by_source = BTreeMap::new();
    for method in methods {
        method.validate()?;
        if method_by_source
            .insert(method.source_id.as_str(), method)
            .is_some()
        {
            return Err(DiscoveryError::DuplicateSource(method.source_id.clone()));
        }
    }
    if method_by_source != canonical_by_source {
        return Err(DiscoveryError::CatalogDrift);
    }

    let mut logged_sources = BTreeSet::new();
    let mut log_ids = BTreeSet::new();
    for log in logs {
        log.validate()?;
        if !log_ids.insert(log.log_id.as_str()) {
            return Err(DiscoveryError::DuplicateLog(log.log_id.clone()));
        }
        let Some(method) = method_by_source.get(log.source_id.as_str()) else {
            return Err(DiscoveryError::UnknownSource(log.source_id.clone()));
        };
        if log.method != method.method {
            return Err(DiscoveryError::LogMethodMismatch(log.log_id.clone()));
        }
        logged_sources.insert(log.source_id.as_str());
    }

    let mut assessed_sources = BTreeSet::new();
    for assessment in coverage {
        assessment.validate()?;
        if !method_by_source.contains_key(assessment.source_id.as_str()) {
            return Err(DiscoveryError::UnknownSource(assessment.source_id.clone()));
        }
        if !assessed_sources.insert(assessment.source_id.as_str()) {
            return Err(DiscoveryError::DuplicateCoverage(
                assessment.source_id.clone(),
            ));
        }
        if assessment.executed && !logged_sources.contains(assessment.source_id.as_str()) {
            return Err(DiscoveryError::MissingExecutionLog(
                assessment.source_id.clone(),
            ));
        }
        if !assessment.executed && logged_sources.contains(assessment.source_id.as_str()) {
            return Err(DiscoveryError::UnexpectedExecutionLog(
                assessment.source_id.clone(),
            ));
        }
    }
    let declared_sources = method_by_source.keys().copied().collect::<BTreeSet<_>>();
    if assessed_sources != declared_sources {
        return Err(DiscoveryError::IncompleteCoverage);
    }
    Ok(())
}

/// Convert a bounded `OpenCitations` fixture response into forward-citation edges.
pub fn parse_opencitations_forward_fixture(
    seed_id: &str,
    receipt_id: &str,
    payload: &[u8],
) -> Result<Vec<DiscoveryEdge>, DiscoveryError> {
    require_identifier(seed_id, "seed identifier")?;
    require_identifier(receipt_id, "receipt identifier")?;
    if payload.len() > MAX_CITATION_FIXTURE_BYTES {
        return Err(DiscoveryError::FixtureLimit(
            "OpenCitations fixture exceeds the byte budget",
        ));
    }
    let rows: Vec<OpenCitationsRow> = serde_json::from_slice(payload)?;
    if rows.is_empty() {
        return Err(DiscoveryError::EmptyCitationFixture);
    }
    if rows.len() > MAX_CITATION_ROWS {
        return Err(DiscoveryError::FixtureLimit(
            "OpenCitations fixture exceeds the row budget",
        ));
    }
    let mut cited_ids = BTreeSet::new();
    for row in rows {
        for citation in row.citing.split_whitespace() {
            let citation = citation.trim();
            if citation.len() > MAX_CITATION_IDENTIFIER_BYTES {
                return Err(DiscoveryError::FixtureLimit(
                    "OpenCitations identifier exceeds the byte budget",
                ));
            }
            require_identifier(citation, "citing identifier")?;
            if !citation.is_empty() && citation != seed_id {
                cited_ids.insert(citation.to_owned());
                if cited_ids.len() > MAX_CITATION_ROWS {
                    return Err(DiscoveryError::FixtureLimit(
                        "OpenCitations fixture exceeds the identifier budget",
                    ));
                }
            }
        }
    }
    if cited_ids.is_empty() {
        return Err(DiscoveryError::EmptyCitationFixture);
    }
    Ok(cited_ids
        .into_iter()
        .map(|discovered_id| DiscoveryEdge {
            edge_id: stable_edge_id(seed_id, receipt_id, &discovered_id),
            seed_id: seed_id.to_owned(),
            discovered_id,
            method: DiscoveryMethod::ForwardCitation,
            provider_id: "opencitations-fixture".to_owned(),
            receipt_id: receipt_id.to_owned(),
        })
        .collect())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenCitationsRow {
    citing: String,
}

fn stable_edge_id(seed_id: &str, receipt_id: &str, discovered_id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in [seed_id, receipt_id, discovered_id] {
        hasher.update(value.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(value.as_bytes());
    }
    format!("opencitations-forward-{}", hasher.finalize().to_hex())
}

fn require_identifier(value: &str, label: &'static str) -> Result<(), DiscoveryError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.len() > MAX_CITATION_IDENTIFIER_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(DiscoveryError::InvalidIdentifier(label));
    }
    Ok(())
}

/// Resolve candidates and evidence on seed-to-candidate walks within the depth budget.
///
/// Candidates are truncated deterministically by minimum depth and identifier.
/// Evidence is never truncated: exhaustion of the shared work or evidence budget
/// rejects the complete run, without returning partial candidates.
pub fn bounded_candidates(run: &DiscoveryRun) -> Result<Vec<DiscoveredCandidate>, DiscoveryError> {
    bounded_candidates_with_budgets(run, MAX_DISCOVERY_WORK, MAX_DISCOVERY_EVIDENCE)
}

fn consume_budget(remaining: &mut usize, label: &'static str) -> Result<(), DiscoveryError> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or(DiscoveryError::ResourceLimit(label))?;
    Ok(())
}

fn bounded_candidates_with_budgets(
    run: &DiscoveryRun,
    mut work_remaining: usize,
    mut evidence_remaining: usize,
) -> Result<Vec<DiscoveredCandidate>, DiscoveryError> {
    run.validate()?;
    let seed_set = run
        .seed_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut edge_ids = BTreeSet::new();
    let mut adjacency = BTreeMap::<&str, Vec<&DiscoveryEdge>>::new();
    let mut incoming = BTreeMap::<&str, Vec<&DiscoveryEdge>>::new();
    for edge in &run.edges {
        consume_budget(&mut work_remaining, "discovery traversal work budget")?;
        if edge.method != run.method {
            return Err(DiscoveryError::MethodMismatch(edge.edge_id.clone()));
        }
        if !edge_ids.insert(edge.edge_id.as_str()) {
            return Err(DiscoveryError::DuplicateEdge(edge.edge_id.clone()));
        }
        adjacency
            .entry(edge.seed_id.as_str())
            .or_default()
            .push(edge);
        incoming
            .entry(edge.discovered_id.as_str())
            .or_default()
            .push(edge);
    }

    let mut queue = seed_set
        .iter()
        .copied()
        .map(|identifier| (identifier, 0_u8))
        .collect::<VecDeque<_>>();
    let mut seen_depth = seed_set
        .iter()
        .copied()
        .map(|identifier| (identifier, 0_u8))
        .collect::<BTreeMap<_, _>>();

    while let Some((source, depth)) = queue.pop_front() {
        consume_budget(&mut work_remaining, "discovery traversal work budget")?;
        if depth >= run.maximum_depth {
            continue;
        }
        for edge in adjacency.get(&source).into_iter().flatten() {
            consume_budget(&mut work_remaining, "discovery traversal work budget")?;
            if !seen_depth.contains_key(edge.discovered_id.as_str()) {
                let next_depth = depth + 1;
                seen_depth.insert(edge.discovered_id.as_str(), next_depth);
                queue.push_back((edge.discovered_id.as_str(), next_depth));
            }
        }
    }

    let mut selected = seen_depth
        .iter()
        .map(|(&identifier, &depth)| (identifier, depth))
        .filter(|(identifier, _)| !seed_set.contains(identifier))
        .collect::<Vec<_>>();
    selected.sort_by_key(|&(identifier, depth)| (depth, identifier));
    selected.truncate(usize::try_from(run.maximum_records).unwrap_or(usize::MAX));

    let mut candidates = Vec::with_capacity(selected.len());
    for (discovered_id, depth) in selected {
        let mut reverse_seen = BTreeSet::from([discovered_id]);
        let mut reverse_queue = VecDeque::from([(discovered_id, 0_u8)]);
        let mut supporting_edges = Vec::new();
        while let Some((target, reverse_depth)) = reverse_queue.pop_front() {
            consume_budget(&mut work_remaining, "discovery traversal work budget")?;
            for edge in incoming.get(target).into_iter().flatten() {
                consume_budget(&mut work_remaining, "discovery traversal work budget")?;
                // Concatenating shortest prefix/suffix walks through this edge
                // proves precisely that it supports an in-budget discovery walk.
                if seen_depth
                    .get(edge.seed_id.as_str())
                    .is_some_and(|seed_depth| seed_depth + 1 + reverse_depth <= run.maximum_depth)
                {
                    consume_budget(
                        &mut evidence_remaining,
                        "discovery evidence membership budget",
                    )?;
                    supporting_edges.push(edge.edge_id.clone());
                }
                let next_depth = reverse_depth + 1;
                if next_depth < run.maximum_depth && reverse_seen.insert(edge.seed_id.as_str()) {
                    reverse_queue.push_back((edge.seed_id.as_str(), next_depth));
                }
            }
        }
        supporting_edges.sort();
        candidates.push(DiscoveredCandidate {
            discovered_id: discovered_id.to_owned(),
            depth,
            edge_ids: supporting_edges,
            requires_human_release: run.requires_human_release,
        });
    }
    Ok(candidates)
}

/// Supplementary-discovery failure.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// Graph processing exceeded its shared work or output-evidence ceiling.
    #[error("{0} exhausted; no partial discovery result is returned")]
    ResourceLimit(&'static str),
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Edge method differed from the run method.
    #[error("discovery edge `{0}` uses a different method from its run")]
    MethodMismatch(String),
    /// Edge identifier appeared more than once.
    #[error("discovery edge identifier `{0}` is duplicated")]
    DuplicateEdge(String),
    /// A source identifier appeared more than once in the method catalogue.
    #[error("discovery source identifier `{0}` is duplicated")]
    DuplicateSource(String),
    /// A manual log identifier appeared more than once.
    #[error("manual discovery log identifier `{0}` is duplicated")]
    DuplicateLog(String),
    /// A coverage assessment appeared more than once for a source.
    #[error("coverage assessment for discovery source `{0}` is duplicated")]
    DuplicateCoverage(String),
    /// A log or coverage row refers to an undeclared source.
    #[error("discovery source `{0}` is not declared")]
    UnknownSource(String),
    /// A manual log used a different method than its declared source.
    #[error("manual discovery log `{0}` uses a different method from its source")]
    LogMethodMismatch(String),
    /// An executed source lacks a reproducible method log.
    #[error("executed discovery source `{0}` requires a method log")]
    MissingExecutionLog(String),
    /// A log exists for a source reported as unexecuted.
    #[error("unexecuted discovery source `{0}` must not have a method log")]
    UnexpectedExecutionLog(String),
    /// Caller-supplied source methods differ from the canonical catalogue.
    #[error("discovery source catalogue differs from the canonical fixture")]
    CatalogDrift,
    /// The coverage matrix does not assess every declared source exactly once.
    #[error("discovery coverage must assess every declared source exactly once")]
    IncompleteCoverage,
    /// A required fixture identifier was unsafe or exceeded its budget.
    #[error("{0} is blank, unbounded, padded, or contains control characters")]
    InvalidIdentifier(&'static str),
    /// A fixture exceeded its deterministic resource budget.
    #[error("{0}")]
    FixtureLimit(&'static str),
    /// A citation fixture contained no usable citation evidence.
    #[error("citation fixture contains no usable citation evidence")]
    EmptyCitationFixture,
    /// Fixture decoding failed.
    #[error("discovery fixture is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{
        ContactOutcome, DiscoveryCoverageAssessment, DiscoveryCoverageRisk, DiscoveryEdge,
        DiscoveryMethod, DiscoveryRun, ManualDiscoveryLog,
    };

    use super::*;

    fn run(edges: Vec<DiscoveryEdge>) -> DiscoveryRun {
        DiscoveryRun {
            schema_version: "org.searchright.discovery-run.v1".to_owned(),
            review_id: "review-1".to_owned(),
            run_id: "citation-run-1".to_owned(),
            method: DiscoveryMethod::ForwardCitation,
            seed_ids: vec!["seed".to_owned()],
            edges,
            maximum_depth: 2,
            maximum_records: 10,
            requires_human_release: true,
        }
    }

    fn edge(edge_id: &str, seed_id: &str, discovered_id: &str) -> DiscoveryEdge {
        DiscoveryEdge {
            edge_id: edge_id.to_owned(),
            seed_id: seed_id.to_owned(),
            discovered_id: discovered_id.to_owned(),
            method: DiscoveryMethod::ForwardCitation,
            provider_id: "fixture".to_owned(),
            receipt_id: format!("receipt-{edge_id}"),
        }
    }

    #[test]
    fn graph_traversal_respects_depth_and_human_release() {
        let run = DiscoveryRun {
            schema_version: "org.searchright.discovery-run.v1".to_owned(),
            review_id: "review-1".to_owned(),
            run_id: "citation-run-1".to_owned(),
            method: DiscoveryMethod::ForwardCitation,
            seed_ids: vec!["seed".to_owned()],
            edges: vec![
                DiscoveryEdge {
                    edge_id: "edge-1".to_owned(),
                    seed_id: "seed".to_owned(),
                    discovered_id: "candidate-1".to_owned(),
                    method: DiscoveryMethod::ForwardCitation,
                    provider_id: "fixture".to_owned(),
                    receipt_id: "receipt-1".to_owned(),
                },
                DiscoveryEdge {
                    edge_id: "edge-2".to_owned(),
                    seed_id: "candidate-1".to_owned(),
                    discovered_id: "candidate-2".to_owned(),
                    method: DiscoveryMethod::ForwardCitation,
                    provider_id: "fixture".to_owned(),
                    receipt_id: "receipt-2".to_owned(),
                },
            ],
            maximum_depth: 1,
            maximum_records: 10,
            requires_human_release: true,
        };
        let candidates = bounded_candidates(&run);
        assert!(candidates.is_ok());
        if let Ok(candidates) = candidates {
            assert_eq!(candidates.len(), 1);
            assert_eq!(
                candidates.first().map(|item| item.discovered_id.as_str()),
                Some("candidate-1")
            );
            assert!(candidates.iter().all(|item| item.requires_human_release));
        }
    }

    #[test]
    fn output_is_deterministic_and_record_bounded() {
        let edges = vec![
            edge("edge-z", "seed", "candidate-z"),
            edge("edge-a", "seed", "candidate-a"),
            edge("edge-b", "seed", "candidate-b"),
        ];
        let mut forward = run(edges.clone());
        forward.maximum_records = 2;
        let mut reverse = run(edges.into_iter().rev().collect());
        reverse.maximum_records = 2;

        let forward_candidates = bounded_candidates(&forward);
        let reverse_candidates = bounded_candidates(&reverse);

        let (Ok(forward_candidates), Ok(reverse_candidates)) =
            (forward_candidates, reverse_candidates)
        else {
            panic!("fixture discovery runs should be valid");
        };
        assert_eq!(forward_candidates, reverse_candidates);
        assert_eq!(
            forward_candidates
                .into_iter()
                .map(|item| item.discovered_id)
                .collect::<Vec<_>>(),
            vec!["candidate-a".to_owned(), "candidate-b".to_owned()]
        );
    }

    #[test]
    fn rejects_duplicate_edges_and_method_mismatch() {
        let duplicate = run(vec![
            edge("edge-1", "seed", "candidate-a"),
            edge("edge-1", "seed", "candidate-b"),
        ]);
        assert!(matches!(
            bounded_candidates(&duplicate),
            Err(DiscoveryError::DuplicateEdge(edge_id)) if edge_id == "edge-1"
        ));

        let mut wrong_method = edge("edge-2", "seed", "candidate-a");
        wrong_method.method = DiscoveryMethod::BackwardCitation;
        assert!(matches!(
            bounded_candidates(&run(vec![wrong_method])),
            Err(DiscoveryError::MethodMismatch(edge_id)) if edge_id == "edge-2"
        ));
    }

    #[test]
    fn aggregates_distinct_receipted_paths() {
        let candidates = bounded_candidates(&run(vec![
            edge("edge-1", "seed", "candidate-a"),
            edge("edge-2", "seed", "candidate-a"),
        ]));

        let Ok(candidates) = candidates else {
            panic!("fixture discovery run should be valid");
        };
        assert_eq!(
            candidates
                .first()
                .map(|candidate| candidate.edge_ids.clone()),
            Some(vec!["edge-1".to_owned(), "edge-2".to_owned()])
        );
    }

    #[test]
    fn retains_complete_evidence_for_multihop_candidates() {
        let candidates = bounded_candidates(&run(vec![
            edge("edge-1", "seed", "candidate-a"),
            edge("edge-2", "candidate-a", "candidate-b"),
        ]));

        let Ok(candidates) = candidates else {
            panic!("fixture discovery run should be valid");
        };
        assert_eq!(
            candidates
                .iter()
                .find(|candidate| candidate.discovered_id == "candidate-b")
                .map(|candidate| candidate.edge_ids.clone()),
            Some(vec!["edge-1".to_owned(), "edge-2".to_owned()])
        );
    }

    #[test]
    fn bounded_walk_evidence_respects_depth_and_input_order() {
        let mut fixture = run(vec![
            edge("sa", "seed", "a"),
            edge("sb", "seed", "b"),
            edge("ba", "b", "a"),
            edge("ax", "a", "x"),
        ]);
        for (depth, expected) in [(2, vec!["ax", "sa"]), (3, vec!["ax", "ba", "sa", "sb"])] {
            fixture.maximum_depth = depth;
            let Ok(forward) = bounded_candidates(&fixture) else {
                panic!("bounded graph should be valid");
            };
            assert_eq!(
                forward
                    .iter()
                    .find(|candidate| candidate.discovered_id == "x")
                    .map(|candidate| candidate
                        .edge_ids
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()),
                Some(expected)
            );
            fixture.edges.reverse();
            assert_eq!(bounded_candidates(&fixture).ok(), Some(forward));
        }
    }

    #[test]
    fn cycle_evidence_requires_enough_walk_depth() {
        let mut fixture = run(vec![
            edge("sa", "seed", "a"),
            edge("ab", "a", "b"),
            edge("ba", "b", "a"),
        ]);
        for (depth, expected) in [(2, vec!["sa"]), (3, vec!["ab", "ba", "sa"])] {
            fixture.maximum_depth = depth;
            let Ok(candidates) = bounded_candidates(&fixture) else {
                panic!("cycle fixture should be valid");
            };
            assert_eq!(
                candidates
                    .iter()
                    .find(|candidate| candidate.discovered_id == "a")
                    .map(|candidate| (
                        candidate.depth,
                        candidate
                            .edge_ids
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                    )),
                Some((1, expected))
            );
        }
    }

    #[test]
    fn work_and_evidence_budgets_reject_instead_of_truncating() {
        let fixture = run(vec![edge("sa", "seed", "a")]);
        // One indexed edge, three forward operations, three reverse operations.
        assert!(bounded_candidates_with_budgets(&fixture, 7, 1).is_ok());
        assert!(matches!(
            bounded_candidates_with_budgets(&fixture, 6, 1),
            Err(DiscoveryError::ResourceLimit(
                "discovery traversal work budget"
            ))
        ));
        assert!(matches!(
            bounded_candidates_with_budgets(&fixture, 7, 0),
            Err(DiscoveryError::ResourceLimit(
                "discovery evidence membership budget"
            ))
        ));
    }

    #[test]
    fn record_limit_avoids_unselected_fanout_evidence() {
        let mut edges = Vec::new();
        for index in 0..100 {
            let source = format!("a{index:03}");
            edges.push(edge(&format!("s{index}"), "seed", &source));
            edges.push(edge(&format!("h{index}"), &source, "hub"));
            edges.push(edge(&format!("z{index}"), "hub", &format!("z{index:03}")));
        }
        let mut fixture = run(edges);
        fixture.maximum_depth = 3;
        fixture.maximum_records = 1;
        let Ok(candidates) = bounded_candidates_with_budgets(&fixture, 1_000, 1) else {
            panic!("one retained record should require only one evidence membership");
        };
        let [candidate] = candidates.as_slice() else {
            panic!("record limit must retain exactly one candidate");
        };
        assert_eq!(candidate.discovered_id, "a000");
        assert_eq!(candidate.edge_ids, vec!["s0"]);
    }

    // Deliberately independent, tiny exhaustive walk enumerator: no distance
    // identity and no provenance merging are shared with the implementation.
    fn walk_oracle(fixture: &DiscoveryRun) -> Vec<DiscoveredCandidate> {
        let seeds = fixture
            .seed_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut queue = seeds
            .iter()
            .map(|&seed| (seed, 0_u8, Vec::<String>::new()))
            .collect::<VecDeque<_>>();
        let mut evidence = BTreeMap::<&str, (u8, BTreeSet<String>)>::new();
        while let Some((node, depth, path)) = queue.pop_front() {
            if !seeds.contains(node) {
                let entry = evidence
                    .entry(node)
                    .or_insert_with(|| (depth, BTreeSet::new()));
                entry.0 = entry.0.min(depth);
                entry.1.extend(path.iter().cloned());
            }
            if depth < fixture.maximum_depth {
                for edge in fixture.edges.iter().filter(|edge| edge.seed_id == node) {
                    let mut next_path = path.clone();
                    next_path.push(edge.edge_id.clone());
                    queue.push_back((edge.discovered_id.as_str(), depth + 1, next_path));
                }
            }
        }
        let mut candidates = evidence
            .into_iter()
            .map(|(identifier, (depth, edges))| DiscoveredCandidate {
                discovered_id: identifier.to_owned(),
                depth,
                edge_ids: edges.into_iter().collect(),
                requires_human_release: true,
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| (a.depth, &a.discovered_id).cmp(&(b.depth, &b.discovered_id)));
        candidates.truncate(usize::try_from(fixture.maximum_records).unwrap_or(usize::MAX));
        candidates
    }

    #[test]
    fn exhaustive_small_graphs_match_walk_oracle() {
        let nodes = ["seed", "a", "b"];
        let possible = nodes
            .iter()
            .flat_map(|source| {
                nodes
                    .iter()
                    .filter(move |target| source != *target)
                    .map(move |target| edge(&format!("{source}-{target}"), source, target))
            })
            .collect::<Vec<_>>();
        for mask in 0..(1_usize << possible.len()) {
            for maximum_depth in 1..=4 {
                for seeds in [
                    vec!["seed".to_owned()],
                    vec!["seed".to_owned(), "b".to_owned()],
                ] {
                    let mut fixture = run(possible
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| mask & (1 << index) != 0)
                        .map(|(_, edge)| edge.clone())
                        .collect());
                    fixture.maximum_depth = maximum_depth;
                    fixture.seed_ids = seeds;
                    let expected = walk_oracle(&fixture);
                    assert_eq!(
                        bounded_candidates(&fixture).ok(),
                        Some(expected.clone()),
                        "mask={mask}, depth={maximum_depth}"
                    );
                    fixture.edges.reverse();
                    fixture.seed_ids.reverse();
                    assert_eq!(bounded_candidates(&fixture).ok(), Some(expected));
                }
            }
        }
    }

    #[test]
    fn citing_tokens_reject_controls_without_changing_length_errors() {
        for value in ["doi:10.1/a\0b", "doi:10.1/a\u{7f}b"] {
            let payload = serde_json::json!([{ "citing": value }]).to_string();
            assert!(matches!(
                parse_opencitations_forward_fixture("seed", "receipt", payload.as_bytes()),
                Err(DiscoveryError::InvalidIdentifier("citing identifier"))
            ));
        }
        let payload =
            serde_json::json!([{ "citing": "x".repeat(MAX_CITATION_IDENTIFIER_BYTES + 1) }])
                .to_string();
        assert!(matches!(
            parse_opencitations_forward_fixture("seed", "receipt", payload.as_bytes()),
            Err(DiscoveryError::FixtureLimit(_))
        ));
        assert!(
            parse_opencitations_forward_fixture(
                "seed",
                "receipt",
                "[{\"citing\":\"doi:10.1/α doi:10.1/β\"}]".as_bytes()
            )
            .is_ok()
        );
    }

    #[test]
    fn source_catalog_covers_every_named_discovery_vector() {
        let catalog = source_method_catalog();
        assert!(catalog.is_ok());
        let Ok(catalog) = catalog else {
            return;
        };
        let ids = catalog
            .iter()
            .map(|item| item.source_id.as_str())
            .collect::<BTreeSet<_>>();
        for required in [
            "clinicaltrials-gov",
            "who-ictrp",
            "anzctr",
            "osf",
            "zenodo",
            "figshare",
            "dataverse",
            "institutional-repositories",
            "conference-search",
            "thesis-search",
            "policy-search",
            "organisational-websites",
            "opencitations",
            "backward-reference-checking",
            "contact-log",
            "handsearch-log",
        ] {
            assert!(ids.contains(required));
        }
        assert!(catalog.iter().all(|method| !method.limitations.is_empty()));
    }

    #[test]
    fn opencitations_fixture_is_deduplicated_and_bounded_by_the_run() {
        let edges = parse_opencitations_forward_fixture(
            "doi:10.1000/seed",
            "receipt-opencitations-fixture",
            include_bytes!("../../../contracts/fixtures/opencitations-forward.json"),
        );
        assert!(edges.is_ok());
        let Ok(edges) = edges else {
            return;
        };
        assert_eq!(edges.len(), 3);
        let mut run = run(edges);
        run.seed_ids = vec!["doi:10.1000/seed".to_owned()];
        run.maximum_records = 2;
        let candidates = bounded_candidates(&run);
        assert!(candidates.is_ok());
        assert!(candidates.is_ok_and(|items| items.len() == 2));
    }

    #[test]
    fn reporting_requires_complete_risk_coverage_and_execution_logs() {
        let catalog = source_method_catalog();
        assert!(catalog.is_ok());
        let Ok(catalog) = catalog else {
            return;
        };
        let coverage = catalog
            .iter()
            .map(|method| DiscoveryCoverageAssessment {
                source_id: method.source_id.clone(),
                executed: false,
                risk: DiscoveryCoverageRisk::Unknown,
                rationale: vec!["topic-specific execution remains pending".to_owned()],
            })
            .collect::<Vec<_>>();
        assert!(validate_method_reporting(&catalog, &[], &coverage).is_ok());

        let mut executed = coverage;
        if let Some(first) = executed.first_mut() {
            first.executed = true;
            first.risk = DiscoveryCoverageRisk::Moderate;
        }
        assert!(matches!(
            validate_method_reporting(&catalog, &[], &executed),
            Err(DiscoveryError::MissingExecutionLog(_))
        ));
    }

    #[test]
    fn manual_logs_preserve_exact_method_text_without_contact_identity() {
        let log = ManualDiscoveryLog {
            log_id: "contact-1".to_owned(),
            source_id: "contact-log".to_owned(),
            method: DiscoveryMethod::Contact,
            conducted_on: "2026-08-29".to_owned(),
            exact_method_text: "Template C1: request unpublished outcome data".to_owned(),
            operator_role: "review information specialist".to_owned(),
            scope_details: vec!["Contact template C1; one organisation role".to_owned()],
            total_results: None,
            results_inspected: 0,
            discovered_ids: Vec::new(),
            contact_outcome: Some(ContactOutcome::NoResponse),
            last_follow_up_on: Some("2026-08-29".to_owned()),
            limitations: vec!["No response is not evidence that no study exists".to_owned()],
        };
        assert!(log.validate().is_ok());
        let encoded = serde_json::to_string(&log);
        assert!(encoded.is_ok());
        assert!(!encoded.is_ok_and(|value| value.contains('@')));
    }

    #[test]
    fn reporting_rejects_mismatched_and_unexpected_execution_logs() {
        let Ok(catalog) = source_method_catalog() else {
            panic!("canonical fixture must decode");
        };
        let mut coverage = catalog
            .iter()
            .map(|method| DiscoveryCoverageAssessment {
                source_id: method.source_id.clone(),
                executed: false,
                risk: DiscoveryCoverageRisk::Unknown,
                rationale: vec!["Not executed".to_owned()],
            })
            .collect::<Vec<_>>();
        let log = ManualDiscoveryLog {
            log_id: "handsearch-1".to_owned(),
            source_id: "handsearch-log".to_owned(),
            method: DiscoveryMethod::Handsearch,
            conducted_on: "2026-08-30".to_owned(),
            exact_method_text: "Journal fixture volume 1 issue 1 pages 1-2".to_owned(),
            operator_role: "review operator".to_owned(),
            scope_details: vec!["Fixture volume 1".to_owned()],
            total_results: Some(1),
            results_inspected: 1,
            discovered_ids: vec!["candidate-1".to_owned()],
            contact_outcome: None,
            last_follow_up_on: None,
            limitations: vec!["Fixture only".to_owned()],
        };
        assert!(matches!(
            validate_method_reporting(&catalog, std::slice::from_ref(&log), &coverage),
            Err(DiscoveryError::UnexpectedExecutionLog(_))
        ));
        for item in &mut coverage {
            if item.source_id == "handsearch-log" {
                item.executed = true;
            }
        }
        assert!(validate_method_reporting(&catalog, std::slice::from_ref(&log), &coverage).is_ok());
        let mut wrong = log.clone();
        wrong.method = DiscoveryMethod::BackwardCitation;
        assert!(matches!(
            validate_method_reporting(&catalog, &[wrong], &coverage),
            Err(DiscoveryError::LogMethodMismatch(_))
        ));
        assert!(matches!(
            validate_method_reporting(&catalog, &[log.clone(), log], &coverage),
            Err(DiscoveryError::DuplicateLog(_))
        ));
    }

    #[test]
    fn citation_fixture_limits_fail_closed() {
        let oversized = vec![b' '; MAX_CITATION_FIXTURE_BYTES + 1];
        assert!(matches!(
            parse_opencitations_forward_fixture("seed", "receipt", &oversized),
            Err(DiscoveryError::FixtureLimit(_))
        ));

        let long_identifier = format!(
            r#"[{{"citing":"{}"}}]"#,
            "x".repeat(MAX_CITATION_IDENTIFIER_BYTES + 1)
        );
        assert!(matches!(
            parse_opencitations_forward_fixture("seed", "receipt", long_identifier.as_bytes()),
            Err(DiscoveryError::FixtureLimit(_))
        ));
    }

    #[test]
    fn reporting_rejects_catalog_subsets() {
        let Ok(catalog) = source_method_catalog() else {
            panic!("canonical catalogue should decode");
        };
        assert!(matches!(
            validate_method_reporting(&[], &[], &[]),
            Err(DiscoveryError::CatalogDrift)
        ));
        let Some((_, subset)) = catalog.split_last() else {
            panic!("canonical catalogue should not be empty");
        };
        assert!(matches!(
            validate_method_reporting(subset, &[], &[]),
            Err(DiscoveryError::CatalogDrift)
        ));
    }

    #[test]
    fn citation_fixture_is_strict_and_edge_ids_are_context_bound() {
        for malformed in [
            b"[]".as_slice(),
            b"[{}]",
            b"[{\"wrong\":\"doi:x\"}]",
            b"[{\"citing\":\"\"}]",
        ] {
            assert!(parse_opencitations_forward_fixture("seed", "receipt", malformed).is_err());
        }
        let payload = br#"[{"citing":"doi:10.1/result"}]"#;
        let Ok(first) = parse_opencitations_forward_fixture("seed-a", "receipt", payload) else {
            panic!("first citation fixture should be valid");
        };
        let Ok(second) = parse_opencitations_forward_fixture("seed-b", "receipt", payload) else {
            panic!("second citation fixture should be valid");
        };
        assert_ne!(
            first.first().map(|edge| edge.edge_id.as_str()),
            second.first().map(|edge| edge.edge_id.as_str())
        );
        let Ok(repeated) = parse_opencitations_forward_fixture("seed-a", "receipt", payload) else {
            panic!("repeated citation fixture should be valid");
        };
        assert_eq!(first, repeated);

        for invalid in [" seed", "seed ", "seed\0value"] {
            assert!(matches!(
                parse_opencitations_forward_fixture(invalid, "receipt", payload),
                Err(DiscoveryError::InvalidIdentifier("seed identifier"))
            ));
        }
        assert!(matches!(
            parse_opencitations_forward_fixture(
                &"x".repeat(MAX_CITATION_IDENTIFIER_BYTES + 1),
                "receipt",
                payload
            ),
            Err(DiscoveryError::InvalidIdentifier("seed identifier"))
        ));
    }
}
