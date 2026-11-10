<!-- searchright-issue-key: track-38-phase-3-task-03 -->
# Track 38 / Phase 3 / Task 03

Parent phase key: `track-38-phase-3`
Conductor plan: `conductor/tracks/38-zktls-verifiable-live-execution/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] **38.3 — Atomic checkpoints, resume and idempotent record emission**
  - Prerequisite: 38.2; crash/restart and cancellation fixture.
  - Acceptance: cargo test --locked -p evidence-search-core -p searchright-store.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
