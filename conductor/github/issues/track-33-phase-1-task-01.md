<!-- searchright-issue-key: track-33-phase-1-task-01 -->
# Track 33 / Phase 1 / Task 01

Parent phase key: `track-33-phase-1`
Conductor plan: `conductor/tracks/33-operational-reliability/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-ops/src/lib.rs`
  - [x] `crates/searchright-contracts/src/ops.rs`
  - [x] `contracts/examples/component-health.json`
  - [x] `contracts/examples/telemetry-policy.json`
  - [x] `contracts/examples/backup-manifest.json`
  - [x] `contracts/examples/incident-record.json`
  - [x] `docs/operations/reliability.md`
  - [x] `docs/operations/backup-restore.md`
  - [x] `docs/operations/incident-response.md`
  - [x] `.github/workflows/resilience.yml`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
