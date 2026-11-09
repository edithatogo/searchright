<!-- searchright-issue-key: track-18-phase-4-task-03 -->
# Track 18 / Phase 4 / Task 03

Parent phase key: `track-18-phase-4`
Conductor plan: `conductor/tracks/18-alpha-release/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 18 assertions (T18-A001..T18-A006) to symbol_and_gate_mapped with concrete implementation symbols across package_source.py, check_packaging_reproducibility.py, generate_source_sbom.py, generate_source_hash_manifest.py, mcp_smoke.py, check_cli_distribution.py, check_workflow_hardening.py, and check_release_rehearsal.py; refactor package_source and check_packaging_reproducibility to expose modular helper APIs with deterministic header normalization; create unit test suite in tests/test_alpha_release_packaging.py; record conductor review receipt and preserve open release, signing, container, and Cargo.lock blockers without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
