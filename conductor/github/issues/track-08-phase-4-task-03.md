<!-- searchright-issue-key: track-08-phase-4-task-03 -->
# Track 08 / Phase 4 / Task 03

Parent phase key: `track-08-phase-4`
Conductor plan: `conductor/tracks/08-prisma-press-reporting/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `c234c41f940b37e153e9bf151458059e63c51b47`: Fail closed on PRESS decisions, bind appendix execution evidence to the compiler version and query fingerprint, and escape active Typst inline markup controls.
  - Review fix `b48dff0`: Reconcile Track 08 traceability, deliverables, and assertion mappings across all reporting renderers and PRESS workflows; resolve strict YAML ingestion and Clippy assert hygiene in shared facade and CLI/MCP test surfaces.
  - Review fix `HEAD`: Implement governance tests in tests/test_prisma_press_reporting_governance.py, record panel evaluation and Conductor review receipts, resolve open gates, and close out Track 08.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.
  - [x] Review fix: Preserve the earlier bounded render-format and PRISMA.jl projection-parity work in the existing completed Phase 4 reporting tasks and source receipts; restoring the two original Phase 3 keys does not reopen that completed component work or claim independent PRESS acceptance.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
