<!-- searchright-issue-key: track-25-phase-4-task-03 -->
# Track 25 / Phase 4 / Task 03

Parent phase key: `track-25-phase-4`
Conductor plan: `conductor/tracks/25-provenance-research-objects/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 25 assertions (T25-A001..T25-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-provenance, review_bundle.py, RO-Crate and W3C PROV boundaries, review-bundle-manifest and handoff-plan schemas, and test_provenance_research_objects_governance.py; create unit test suite in tests/test_provenance_research_objects_governance.py validating RO-Crate 1.3 schema and context, W3C PROV entities/activities/agents/predecessor derivations, review bundle packaging and deterministic policy invariants, default-deny handoff plans, and Track 25 traceability invariants; record conductor review receipt and preserve open golden provenance export tests, independent RO-Crate validation tooling, and OSF/Zenodo repository handoff pilot blockers without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
