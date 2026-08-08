<!-- searchright-issue-key: track-03-phase-1-task-01 -->
# Track 03 / Phase 1 / Task 01

Parent phase key: `track-03-phase-1`
Conductor plan: `conductor/tracks/03-shared-provider-runtime/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Implement and document the track's source deliverables.
  - [x] `crates/evidence-search-core/src/provider.rs`
  - [x] `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] `migration/sourceright/replacement-map.yaml`
  - [x] `migration/sourceright/parity-cases.json`
  - [x] `scripts/check_sourceright_migration.py`
  - [x] `docs/adrs/0012-sourceright-and-estate-migration.md`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
