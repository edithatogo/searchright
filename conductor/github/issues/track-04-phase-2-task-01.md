<!-- searchright-issue-key: track-04-phase-2-task-01 -->
# Track 04 / Phase 2 / Task 01

Parent phase key: `track-04-phase-2`
Conductor plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_provider_contract_baselines.py`
  - [x] `python scripts/check_provider_policies.py`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
