<!-- searchright-issue-key: track-04-phase-4-task-03 -->
# Track 04 / Phase 4 / Task 03

Parent phase key: `track-04-phase-4`
Conductor plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `7972c0b96419dd3a63137f5957f98636259e0926`: Preserve stable PubMed/Crossref identity, correct Europe PMC/OpenAlex identity forward-only, reject malformed/incomplete retrieval and verify complete synthetic page goldens.
  - [x] Review fix: Pending commit: add bounded offline EFetch citation/abstract XML parsing and request construction, exact PMID reconciliation, structured metadata and complete synthetic-page golden without switching live transport.
  - [x] Review fix: Pending commit: add bounded XML digest/root/path baseline checks, restricted parser-source declarations and seven Python regressions without promoting static shape checks into execution evidence.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
