<!-- searchright-issue-key: track-28-phase-4-task-03 -->
# Track 28 / Phase 4 / Task 03

Parent phase key: `track-28-phase-4`
Conductor plan: `conductor/tracks/28-institutional-governance-privacy-collaboration/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 28 assertions (T28-A001..T28-A005) to symbol_and_gate_mapped with concrete symbols across searchright-governance, searchright-contracts, searchright-screening and scripts/redaction.py; add tests/test_institutional_governance_privacy_collaboration.py covering policy/request/decision consistency, deny-by-default blocker codes, human-approval and scoped lifecycle authority, receipt minimisation, adjudication independence and threat-model gates; fix the clippy assert_is_empty lint in the searchright-governance unit tests; record the conductor review receipt and preserve the open compiled negative-scenario, institutional review and cross-institution pilot gates without false promotion.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
