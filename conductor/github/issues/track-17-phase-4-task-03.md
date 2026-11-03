<!-- searchright-issue-key: track-17-phase-4-task-03 -->
# Track 17 / Phase 4 / Task 03

Parent phase key: `track-17-phase-4`
Conductor plan: `conductor/tracks/17-benchmarks-calibration/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 17 assertions (T17-A001..T17-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-bench, check_benchmark_governance.py, check_calibration_harness.py, check_dataset_provenance.py, check_hf_dataset_eval_contracts.py, and check_methodology_benchmarks.py; implement SealedSplitManifest, evaluate_sealed_benchmark, validate_public_benchmark_report, work_saved_over_sampling, expected_calibration_error, and brier_score with zero unwrap/expect and 13 passing unit tests; add test_benchmark_calibration_governance.py test suite; record conductor review receipt and preserve open external benchmark and prospective calibration blockers.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
