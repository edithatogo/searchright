# Plan: 12 CiteWeft scholarly extraction and document evidence

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-12`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-12-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-citeweft/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/document.rs`
  - [x] Present source path: `contracts/json-schema/document-evidence.v1.schema.json`
  - [x] Present source path: `contracts/examples/document-evidence.json`
  - [x] Present source path: `integration/citeweft-compatibility.json`
  - [x] Present source path: `docs/citeweft-integration.md`
  - [x] Present source path: `docs/adrs/0013-citeweft-document-evidence-boundary.md`
  - [x] Present source path: `scripts/check_citeweft_integration.py`
  - [x] Present source path: `tests/test_citeweft_governance.py`
  - [x] Assertion ledger: `conductor/tracks/12-citeweft-document-evidence/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-12-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_citeweft_integration.py`
  - [x] `python scripts/check_rust_dependency_graph.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-12-phase-3 -->

- [x] Compile the pinned CiteWeft dependency and adapter on supported platforms.
- [ ] Run golden CiteWeft/GROBID extraction fixtures preserving spans, uncertainty and diagnostics.
- [ ] Run downstream Sourceright consumer-driven compatibility and rollback tests.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-12-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `0ea3f2d`: Upgrade Track 12 assertions to symbol_and_gate_mapped with native tests, add unit tests for from_scholarly_document and adversarial contract validation, and record conductor review receipt.
  - Review fix `HEAD`: Implemented tests/test_citeweft_governance.py, executed agent panel review, verified CiteWeft dependency pins and non-canonical write boundaries, and archived Track 12 with fixture-proven evidence.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
