<!-- searchright-issue-key: track-26-phase-4-task-03 -->
# Track 26 / Phase 4 / Task 03

Parent phase key: `track-26-phase-4`
Conductor plan: `conductor/tracks/26-formal-assurance-contract-evolution/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 26 assertions (T26-A001..T26-A005) to symbol_and_gate_mapped with concrete implementation symbols across searchright-assurance, check_schema_migrations.py, workflow trace schemas, and tests/test_formal_assurance_contract_evolution.py and tests/test_mcp_authority_state_machine.py; create unit test suite in tests/test_formal_assurance_contract_evolution.py validating workflow-trace schemas, fail-closed schema migration registry policies, non-destructive forward migrations with rollback support, and check_schema_migrations.py execution; integrate Hypothesis stateful property testing over the searchright-mcp stdio binary verifying screening authority boundaries; record conductor review receipt and preserve open Kani/Loom/Miri proofs, model-check traces, contract compatibility fixtures, semver cross-version contract tests, and persisted-data migration drill blockers without false promotion.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
