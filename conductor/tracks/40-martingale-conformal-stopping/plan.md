# Plan: 40 Anytime-valid martingale stopping and conformal risk control

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-40`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-40-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `conductor/tracks/40-martingale-conformal-stopping/spec.md`
  - [x] Present source path: `crates/searchright-contracts/src/screening.rs`
  - [x] Present source path: `crates/searchright-ranking/src/lib.rs`
  - [x] Present source path: `crates/searchright-bench/src/lib.rs`
  - [x] Assertion ledger: `conductor/tracks/40-martingale-conformal-stopping/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-40-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/check_traceability.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-40-phase-3 -->

- [ ] Define StoppingConfig, MartingaleStep, and StoppingCertificate contracts in crates/searchright-contracts/src/screening.rs.
- [ ] Implement anytime-valid hypergeometric betting martingale and Ville's inequality e-value wealth trajectory evaluator in crates/searchright-ranking/src/lib.rs.
- [ ] Implement conformal risk-controlling threshold calibration bounding false-negative screening loss in crates/searchright-ranking/src/lib.rs.
- [ ] Integrate martingale stopping certificates and strict holdout-leakage guards into sealed SYNERGY/CLEF TAR benchmark evaluation in crates/searchright-bench/src/lib.rs.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-40-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
