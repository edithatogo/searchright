<!-- searchright-issue-key: track-32-phase-2-task-01 -->
# Track 32 / Phase 2 / Task 01

Parent phase key: `track-32-phase-2`
Conductor plan: `conductor/tracks/32-cross-repository-release-train/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_release_train.py`
  - [x] `python scripts/sync_ecosystem_lock.py --check`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/check_integration_passports.py`
  - [x] `python scripts/check_consumer_contracts.py`
  - [x] `python scripts/check_integration_drift.py`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
