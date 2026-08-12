//! Bounded citation-chasing and supplementary-discovery operations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use schemars::JsonSchema;
use searchright_contracts::{DiscoveryEdge, DiscoveryRun, Validate};
use serde::{Deserialize, Serialize};

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

/// Validate a discovery run and resolve candidates within its depth/record budgets.
pub fn bounded_candidates(run: &DiscoveryRun) -> Result<Vec<DiscoveredCandidate>, DiscoveryError> {
    run.validate()?;
    let seed_set = run.seed_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut edge_ids = BTreeSet::new();
    let mut adjacency = BTreeMap::<String, Vec<&DiscoveryEdge>>::new();
    for edge in &run.edges {
        if edge.method != run.method {
            return Err(DiscoveryError::MethodMismatch(edge.edge_id.clone()));
        }
        if !edge_ids.insert(edge.edge_id.as_str()) {
            return Err(DiscoveryError::DuplicateEdge(edge.edge_id.clone()));
        }
        adjacency
            .entry(edge.seed_id.clone())
            .or_default()
            .push(edge);
    }

    let mut queue = run
        .seed_ids
        .iter()
        .cloned()
        .map(|identifier| (identifier, 0_u8))
        .collect::<VecDeque<_>>();
    let mut seen_depth = seed_set
        .iter()
        .cloned()
        .map(|identifier| (identifier, 0_u8))
        .collect::<BTreeMap<_, _>>();
    let mut evidence = BTreeMap::<String, BTreeSet<String>>::new();

    while let Some((source, depth)) = queue.pop_front() {
        if depth >= run.maximum_depth {
            continue;
        }
        for edge in adjacency.get(&source).into_iter().flatten() {
            let next_depth = depth.saturating_add(1);
            let mut path_evidence = evidence.get(&source).cloned().unwrap_or_default();
            path_evidence.insert(edge.edge_id.clone());
            evidence
                .entry(edge.discovered_id.clone())
                .or_default()
                .extend(path_evidence);
            let should_visit = seen_depth
                .get(&edge.discovered_id)
                .is_none_or(|known| next_depth < *known);
            if should_visit {
                seen_depth.insert(edge.discovered_id.clone(), next_depth);
                queue.push_back((edge.discovered_id.clone(), next_depth));
            }
        }
    }

    let mut candidates = seen_depth
        .into_iter()
        .filter(|(identifier, _)| !seed_set.contains(identifier))
        .map(|(discovered_id, depth)| DiscoveredCandidate {
            edge_ids: evidence
                .remove(&discovered_id)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            discovered_id,
            depth,
            requires_human_release: run.requires_human_release,
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.depth
            .cmp(&right.depth)
            .then_with(|| left.discovered_id.cmp(&right.discovered_id))
    });
    if u64::try_from(candidates.len()).unwrap_or(u64::MAX) > run.maximum_records {
        candidates.truncate(usize::try_from(run.maximum_records).unwrap_or(usize::MAX));
    }
    Ok(candidates)
}

/// Supplementary-discovery failure.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Edge method differed from the run method.
    #[error("discovery edge `{0}` uses a different method from its run")]
    MethodMismatch(String),
    /// Edge identifier appeared more than once.
    #[error("discovery edge identifier `{0}` is duplicated")]
    DuplicateEdge(String),
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{DiscoveryEdge, DiscoveryMethod, DiscoveryRun};

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
}
