<!-- searchright-issue-key: track-03-phase-4-task-03 -->
# Track 03 / Phase 4 / Task 03

Parent phase key: `track-03-phase-4`
Conductor plan: `conductor/tracks/03-shared-provider-runtime/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Disable proxy bypass, deny non-global address forms and bound streamed responses.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Fail closed on blank parity approvals and incomplete migration-case coverage.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Reconcile source-level claims with pinned Searchright and Sourceright evidence.
  - Review fix `07144ea13c9fd5c4f1105d8a3c6582d3e3d25dfb`: Require exact, unique parity-case and dimension coverage before cutover readiness.
  - Review fix `07144ea13c9fd5c4f1105d8a3c6582d3e3d25dfb`: Reconcile the fixture-parser migration table with the four fixture-backed adapters.
  - Review fix `ca265354de725701085cd3e9d7a466a8c955f15d`: Reject missing or reassigned migration catalogue cells and Rust catalogue drift with mutation regressions.
  - Review fix `ca265354de725701085cd3e9d7a466a8c955f15d`: Preserve transport-execution evidence and complete downstream matrix gates; v1 summary readiness cannot authorize cutover.
  - [x] Review fix: Report an exact record-budget stop with known continuation once; preserve terminal-page and within-page overflow distinctions.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
