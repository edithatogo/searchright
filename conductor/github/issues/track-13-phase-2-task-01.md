<!-- searchright-issue-key: track-13-phase-2-task-01 -->
# Track 13 / Phase 2 / Task 01

Parent phase key: `track-13-phase-2`
Conductor plan: `conductor/tracks/13-integration-passports-github-context/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_integration_passports.py`
  - [x] `python scripts/check_consumer_contracts.py`
  - [x] `python scripts/check_github_issue_hierarchy.py`
  - [x] `python scripts/check_context_integrity.py`
  - [x] `python scripts/check_integration_drift.py`
  - [x] `python scripts/render_github_issues.py --check`
  - [x] `python scripts/sync_context_lock.py --check`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
