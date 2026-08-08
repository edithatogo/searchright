<!-- searchright-issue-key: track-26-phase-1-task-01 -->
# Track 26 / Phase 1 / Task 01

Parent phase key: `track-26-phase-1`
Conductor plan: `conductor/tracks/26-formal-assurance-contract-evolution/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-assurance/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/assurance.rs`
  - [x] Present source path: `contracts/examples/workflow-trace.yaml`
  - [x] Present source path: `docs/adrs/0011-assurance-and-evidence-ladder.md`
  - [x] Present source path: `crates/searchright-assurance/tests/loom_authority.rs`
  - [x] Present source path: `.github/workflows/formal.yml`
  - [x] Present source path: `fuzz/Cargo.toml`
  - [x] Present source path: `verification/harness-matrix.json`
  - [x] Present source path: `contracts/migrations/registry.json`
  - [x] Present source path: `contracts/migrations/github-issue-hierarchy-v1-to-v2.json`
  - [x] Present source path: `scripts/check_schema_migrations.py`
  - [x] Present source path: `docs/contracts/schema-evolution.md`
  - [x] Assertion ledger: `conductor/tracks/26-formal-assurance-contract-evolution/traceability.json`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
