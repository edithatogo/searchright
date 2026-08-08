<!-- searchright-issue-key: track-33-phase-1-task-01 -->
# Track 33 / Phase 1 / Task 01

Parent phase key: `track-33-phase-1`
Conductor plan: `conductor/tracks/33-operational-reliability/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-ops/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/ops.rs`
  - [x] Present source path: `contracts/examples/component-health.json`
  - [x] Present source path: `contracts/examples/telemetry-policy.json`
  - [x] Present source path: `contracts/examples/backup-manifest.json`
  - [x] Present source path: `contracts/examples/incident-record.json`
  - [x] Present source path: `docs/operations/reliability.md`
  - [x] Present source path: `docs/operations/backup-restore.md`
  - [x] Present source path: `docs/operations/incident-response.md`
  - [x] Present source path: `.github/workflows/resilience.yml`
  - [x] Present source path: `scripts/recovery_rehearsal.py`
  - [x] Present source path: `verification/recovery/rehearsal.json`
  - [x] Present source path: `docs/operations/recovery-reference-rehearsal.md`
  - [x] Assertion ledger: `conductor/tracks/33-operational-reliability/traceability.json`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
