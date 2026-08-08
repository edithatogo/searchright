# Plan: 23 Active-learning prioritisation and calibrated agents

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-23`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-23-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-ranking/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/ranking.rs`
  - [x] Present source path: `contracts/examples/ranking-calibration.yaml`
  - [x] Present source path: `skills/systematic-search/references/failure-modes.md`
  - [x] Assertion ledger: `conductor/tracks/23-active-learning-agents/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-23-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-23-phase-3 -->

- [ ] Run SYNERGY and topic-held-out calibration benchmarks.
- [ ] Complete prospective human calibration and subgroup error analysis.
- [ ] Keep stopping/exclusion authority disabled until external evidence and governance approve it.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-23-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
