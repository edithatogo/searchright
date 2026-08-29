<!-- searchright-issue-key: track-11-phase-4-task-03 -->
# Track 11 / Phase 4 / Task 03

Parent phase key: `track-11-phase-4`
Conductor plan: `conductor/tracks/11-agentic-skill/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `a170ff0ebc3226bb3fb390841eb7bb81e8fb1e2c`: Separate approval verification from untrusted proposals; deny generic final exclusions and amendments; enforce bounded, adjacent, purpose-bound and byte-verified handoffs; validate methodology, deduplication, telemetry and receipt freshness boundaries.
  - Review fix `b7f42271ca25860acc83ef16414b514e96f39c2b`: Enforce the complete advisory workflow, exact artifact-bound transition approvals, fixture-versus-live execution, bounded retained artifact bytes, kebab-case wire compatibility, a governed handoff schema, and explicit external closeout gates.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
