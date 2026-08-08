# Plan: 17 Benchmarks, search validation and human calibration

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-17`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-17-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-bench/src/lib.rs`
  - [x] `crates/searchright-ranking/src/lib.rs`
  - [x] `crates/searchright-validation/src/lib.rs`
  - [x] `contracts/examples/benchmark-report.yaml`
  - [x] `contracts/examples/ranking-calibration.yaml`
  - [x] `docs/evaluation/benchmark-and-calibration-protocol.md`
  - [x] `benchmarks/README.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-17-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-17-phase-3 -->

- [ ] Execute versioned benchmark suites and publish raw receipts.
- [ ] Complete the prospective multi-model and information-specialist calibration pilot.
- [ ] Assess subgroup, topic and source-specific performance before any ranking claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-17-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
