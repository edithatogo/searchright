<!-- searchright-issue-key: track-14-phase-4-task-03 -->
# Track 14 / Phase 4 / Task 03

Parent phase key: `track-14-phase-4`
Conductor plan: `conductor/tracks/14-sourceright-migration/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `97c36d0`: Upgrade Track 14 assertions to symbol_and_gate_mapped with native tests, add unit tests for LegacyRuntimeConfig execution_policy and provider_mode in searchright-sourceright-compat, refactor check_sourceright_migration main entry point, and record conductor review receipt.
  - Review fix `HEAD`: Implemented tests/test_sourceright_migration_governance.py, executed agent panel review, verified replacement maps and parity matrices, and archived Track 14 with fixture-proven evidence.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
