<!-- searchright-issue-key: track-26-phase-1-task-01 -->
# Track 26 / Phase 1 / Task 01

Parent phase key: `track-26-phase-1`
Conductor plan: `conductor/tracks/26-formal-assurance-contract-evolution/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-assurance/src/lib.rs`
  - [x] `crates/searchright-contracts/src/assurance.rs`
  - [x] `contracts/examples/workflow-trace.yaml`
  - [x] `docs/adrs/0011-assurance-and-evidence-ladder.md`
  - [x] `crates/searchright-assurance/tests/loom_authority.rs`
  - [x] `.github/workflows/formal.yml`
  - [x] `fuzz/Cargo.toml`
  - [x] `verification/harness-matrix.json`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
