<!-- searchright-issue-key: track-21-phase-4-task-03 -->
# Track 21 / Phase 4 / Task 03

Parent phase key: `track-21-phase-4`
Conductor plan: `conductor/tracks/21-licensed-adapters/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 21 assertions (T21-A001..T21-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-contracts, searchright-licensed, contracts/licensed, and test_licensed_adapters_governance.py; create unit test suite in tests/test_licensed_adapters_governance.py validating adapter index schema, profile invariants, bare DNS host allowlists, zero raw response persistence, and scraping prohibition; harden unit tests in crates/searchright-licensed and crates/searchright-contracts without forbidden fragments; record conductor review receipt and preserve open vendor credentials, terms confirmation, and live smoke receipts without false promotion.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
