# Plan: 27 Accessibility, internationalisation and usability

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-27`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-27-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-diagnostics/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/diagnostic.rs`
  - [x] Present source path: `contracts/examples/diagnostic.yaml`
  - [x] Present source path: `contracts/json-schema/diagnostic.v1.schema.json`
  - [x] Present source path: `docs/adrs/0010-accessible-diagnostics-and-institutional-governance.md`
  - [x] Present source path: `docs/evaluation/external-methodological-evaluation.md`
  - [x] Present source path: `tests/test_accessibility_internationalisation_usability_governance.py`
  - [x] Present source path: `crates/searchright-diagnostics/tests/diagnostic_scenarios.rs`
  - [x] Present source path: `verification/receipts/track-27-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/27-accessibility-internationalisation-usability/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-27-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python3 -B -m unittest discover -s tests -p test_accessibility_internationalisation_usability_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-27-phase-3 -->

- [x] Compile and run plain/JSON/JSONL snapshots and no-colour tests.
- [x] Create message catalogues and execute locale-fallback tests.
- [ ] Complete keyboard, screen-reader and information-specialist usability evaluation.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-27-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 27 assertions (T27-A001..T27-A005) to symbol_and_gate_mapped with concrete implementation symbols across searchright-diagnostics, searchright-contracts, and test_accessibility_internationalisation_usability_governance.py; create unit test suite in tests/test_accessibility_internationalisation_usability_governance.py validating diagnostic schema conformance, namespaced code rules, blocking remediation invariants, colour-free and glyph-free plain text rendering, locale-neutral machine identifier preservation across message catalogues, and external evaluation independence and design protocols; record conductor review receipt and preserve open plain/JSON/JSONL snapshot and no-colour tests, message catalogue fallback tests, and keyboard/screen-reader/information-specialist usability evaluations without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
