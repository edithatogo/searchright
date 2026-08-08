//! Bounded citation-chasing and supplementary-discovery operations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use searchright_contracts::{DiscoveryEdge, DiscoveryRun, Validate};
use serde::{Deserialize, Serialize};

/// One candidate released from a bounded discovery graph for human review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        adjacency.entry(edge.seed_id.clone()).or_default().push(edge);
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
            evidence
                .entry(edge.discovered_id.clone())
                .or_default()
                .insert(edge.edge_id.clone());
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
        candidates.truncate(
            usize::try_from(run.maximum_records).unwrap_or(usize::MAX),
        );
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
            assert_eq!(candidates.first().map(|item| item.discovered_id.as_str()), Some("candidate-1"));
            assert!(candidates.iter().all(|item| item.requires_human_release));
        }
    }
}
