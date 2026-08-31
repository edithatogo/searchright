<!-- searchright-issue-key: track-04-phase-4-task-03 -->
# Track 04 / Phase 4 / Task 03

Parent phase key: `track-04-phase-4`
Conductor plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `ff359cf9261872beb55a663e4eea6f043e58e443`: Rebased parser integrity fix: preserve stable PubMed/Crossref identity, correct Europe PMC/OpenAlex identity forward-only, reject malformed/incomplete retrieval and verify complete synthetic page goldens; historical validation remains bound to its recorded revisions.
  - Review fix `880800bc1deebf8ee3f5dc7ab7c489b1b77a93c7`: Rebased bounded offline EFetch parser/request, exact PMID reconciliation, structured metadata and synthetic-page golden; XML static baseline and seven Python regressions remain distinct from execution evidence.
  - [x] Review fix: Pending validation: parser-to-FixtureProvider/ProviderRegistry receipt binding, memory-cache replay and budget-warning tests on the rebased Track03 budget fix; not a live raw-response provenance chain.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
