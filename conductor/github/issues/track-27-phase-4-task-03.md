<!-- searchright-issue-key: track-27-phase-4-task-03 -->
# Track 27 / Phase 4 / Task 03

Parent phase key: `track-27-phase-4`
Conductor plan: `conductor/tracks/27-accessibility-internationalisation-usability/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 27 assertions (T27-A001..T27-A005) to symbol_and_gate_mapped with concrete implementation symbols across searchright-diagnostics, searchright-contracts, and test_accessibility_internationalisation_usability_governance.py; create unit test suite in tests/test_accessibility_internationalisation_usability_governance.py validating diagnostic schema conformance, namespaced code rules, blocking remediation invariants, colour-free and glyph-free plain text rendering, locale-neutral machine identifier preservation across message catalogues, and external evaluation independence and design protocols; record conductor review receipt and preserve open plain/JSON/JSONL snapshot and no-colour tests, message catalogue fallback tests, and keyboard/screen-reader/information-specialist usability evaluations without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
