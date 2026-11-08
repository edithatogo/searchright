<!-- searchright-issue-key: track-36-phase-4-task-03 -->
# Track 36 / Phase 4 / Task 03

Parent phase key: `track-36-phase-4`
Conductor plan: `conductor/tracks/36-release-candidate-pilots/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_release_candidate_pilots_governance.py with 6 comprehensive unit tests covering release rehearsal script conformance across 11 gates and 3 pilot profiles, mandatory rehearsal safeguards (unexecuted status, dry-run bounds, mandatory rollback), pilot deployment protocol staging (fixture-only local -> institution self-hosted -> remote single-tenant) with PRISMA-S non-negotiable human screening authority and data isolation, ecosystem and registry rehearsal explicit blockers across all 9 submission targets, release-candidate workflow manual dispatch invariants, and reproducible source packaging verification. Upgraded assertions T36-A001..T36-A003 to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 evaluation receipts and preserved open operational gates.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
