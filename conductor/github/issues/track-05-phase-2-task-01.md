<!-- searchright-issue-key: track-05-phase-2-task-01 -->
# Track 05 / Phase 2 / Task 01

Parent phase key: `track-05-phase-2`
Conductor plan: `conductor/tracks/05-execution-audit-store/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/reduce_review_events.py --self-test`
  - [x] `python scripts/review_bundle.py self-test`
  - [x] `python scripts/check_research_object_handoff.py`
  - [x] `cargo test -p searchright-contracts -p searchright-governance -p searchright-store --locked`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
