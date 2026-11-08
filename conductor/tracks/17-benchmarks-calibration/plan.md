# Plan: 17 Benchmarks, search validation and human calibration

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-17`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-17-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `benchmarks/methodology/manifest.json`
  - [x] Present source path: `benchmarks/methodology/README.md`
  - [x] Present source path: `scripts/check_methodology_benchmarks.py`
  - [x] Present source path: `crates/searchright-bench/src/lib.rs`
  - [x] Present source path: `crates/searchright-ranking/src/lib.rs`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `docs/evaluation/benchmark-and-calibration-protocol.md`
  - [x] Present source path: `scripts/check_benchmark_governance.py`
  - [x] Present source path: `scripts/check_calibration_harness.py`
  - [x] Present source path: `scripts/check_dataset_provenance.py`
  - [x] Present source path: `scripts/check_hf_dataset_eval_contracts.py`
  - [x] Present source path: `tests/test_benchmark_calibration_governance.py`
  - [x] Present source path: `verification/receipts/track-17-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/17-benchmarks-calibration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-17-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_methodology_benchmarks.py`
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_benchmark_governance.py`
  - [x] `python scripts/check_calibration_harness.py`
  - [x] `python scripts/check_dataset_provenance.py`
  - [x] `python scripts/check_hf_dataset_eval_contracts.py`
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
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 17 assertions (T17-A001..T17-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-bench, check_benchmark_governance.py, check_calibration_harness.py, check_dataset_provenance.py, check_hf_dataset_eval_contracts.py, and check_methodology_benchmarks.py; implement SealedSplitManifest, evaluate_sealed_benchmark, validate_public_benchmark_report, work_saved_over_sampling, expected_calibration_error, and brier_score with zero unwrap/expect and 13 passing unit tests; add test_benchmark_calibration_governance.py test suite; record conductor review receipt and preserve open external benchmark and prospective calibration blockers.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
