<!-- searchright-issue-key: track-33-phase-2-task-01 -->
# Track 33 / Phase 2 / Task 01

Parent phase key: `track-33-phase-2`
Conductor plan: `conductor/tracks/33-operational-reliability/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/check_workflow_hardening.py`
  - [x] `python scripts/recovery_rehearsal.py --self-test`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
