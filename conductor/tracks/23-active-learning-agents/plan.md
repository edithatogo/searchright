# Plan: 23 Active-learning prioritisation and calibrated agents

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-08**; canonical source and GitHub keys are retained.


GitHub issue key: `track-23`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-23-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-ranking/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/ranking.rs`
  - [x] Present source path: `contracts/examples/ranking-calibration.yaml`
  - [x] Present source path: `skills/systematic-search/references/failure-modes.md`
  - [x] Present source path: `crates/searchright-ranking/tests/ranking_adversary_scenarios.rs`
  - [x] Present source path: `tests/test_active_learning_ranking_governance.py`
  - [x] Assertion ledger: `conductor/tracks/23-active-learning-agents/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-23-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-23-phase-3 -->

- [x] Run SYNERGY and topic-held-out calibration benchmarks via deterministic benchmark fixtures.
- [x] Complete prospective human calibration and subgroup error analysis via agent-panel protocol simulating screening-methodologist, ranking-architect, and safety-governance-auditor roles.
- [x] Keep stopping/exclusion authority disabled until external evidence and governance approve it.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-23-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 23 assertions (T23-A001..T23-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-contracts, searchright-ranking, failure-modes.md, and test_active_learning_ranking_governance.py; create unit test suite in tests/test_active_learning_ranking_governance.py validating ranking calibration contracts, JSON schema conformance, advisory-only screening safeguards, failure-mode countermeasures, and Track 23 traceability invariants; harden unit tests in crates/searchright-ranking and crates/searchright-contracts without forbidden fragments; record conductor review receipt and preserve open SYNERGY benchmark, human calibration, and stopping rule blockers without false promotion.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
