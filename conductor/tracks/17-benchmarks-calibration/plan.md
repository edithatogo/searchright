# Plan: 17 Benchmarks, search validation and human calibration

Current status: **scaffolded**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-17`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-17-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `benchmarks/methodology/manifest.json`
  - [x] Present source path: `benchmarks/methodology/README.md`
  - [x] Present source path: `scripts/check_methodology_benchmarks.py`
  - [x] Present source path: `crates/searchright-bench/src/lib.rs`
  - [x] Present source path: `crates/searchright-ranking/src/lib.rs`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `docs/evaluation/benchmark-and-calibration-protocol.md`
  - [x] Assertion ledger: `conductor/tracks/17-benchmarks-calibration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-17-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_methodology_benchmarks.py`
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
