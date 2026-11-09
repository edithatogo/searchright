<!-- searchright-issue-key: track-04-phase-4-task-03 -->
# Track 04 / Phase 4 / Task 03

Parent phase key: `track-04-phase-4`
Conductor plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `3343d6a86b500e993f34ebe5e6518172ad7876b5`: Parser integrity: preserve stable PubMed/Crossref IDs, correct Europe PMC/OpenAlex identity forward-only, reject incomplete/malformed pages and compare complete synthetic goldens.
  - Review fix `d9d1da3603da8ddc3c0c75f755fa6179f25ad89b`: Bounded offline EFetch citation/abstract parser and request builder, exact PMID reconciliation, structured metadata, complete XML golden and static baseline regressions; no live switch.
  - Review fix `d13cbca321d4372b3a52ecef3f8c8e801472e864`: Actual fixture-runtime receipt binding, memory-cache replay/corruption checks and budget visibility; seven runtime tests pass after the separately delivered Track03 budget fix.
  - Review fix `d7854b0956eed7ec3730ab3219b358d5f1964e77`: Partition all four JSON adapters' normalized cache entries with PROVIDER_PARSER_VERSION; retain legacy FixtureProvider construction and add explicit with_version declaration. Preserve historical entries without re-keying or migration; observed old-cache bypass, then 61 focused Homebrew tests/Clippy and full pinned 1.97.1 snapshot 6d995108106db62474e2746a2021f29e89603b95 passed.
  - Review fix `362c4bf`: Reconcile Track 04 traceability, deliver bounded opt-in EFetch orchestration with subrequest admission and shared NCBI rate grouping, map connector redaction source tests, and record conductor review receipt.
  - Review fix `HEAD`: Implement governance tests in tests/test_open_connectors_governance.py, record panel evaluation and Conductor review receipts, resolve open gates, and close out Track 04.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
