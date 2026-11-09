<!-- searchright-issue-key: track-23-phase-4-task-03 -->
# Track 23 / Phase 4 / Task 03

Parent phase key: `track-23-phase-4`
Conductor plan: `conductor/tracks/23-active-learning-agents/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 23 assertions (T23-A001..T23-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-contracts, searchright-ranking, failure-modes.md, and test_active_learning_ranking_governance.py; create unit test suite in tests/test_active_learning_ranking_governance.py validating ranking calibration contracts, JSON schema conformance, advisory-only screening safeguards, failure-mode countermeasures, and Track 23 traceability invariants; harden unit tests in crates/searchright-ranking and crates/searchright-contracts without forbidden fragments; record conductor review receipt and preserve open SYNERGY benchmark, human calibration, and stopping rule blockers without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
