# Plan: 12 CiteWeft scholarly extraction and document evidence

Current status: **integration_prepared**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-12`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-12-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-citeweft/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/document.rs`
  - [x] Present source path: `contracts/json-schema/document-evidence.v1.schema.json`
  - [x] Present source path: `contracts/examples/document-evidence.json`
  - [x] Present source path: `integration/citeweft-compatibility.json`
  - [x] Present source path: `docs/citeweft-integration.md`
  - [x] Present source path: `docs/adrs/0013-citeweft-document-evidence-boundary.md`
  - [x] Present source path: `scripts/check_citeweft_integration.py`
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

- [ ] Compile the pinned CiteWeft dependency and adapter on supported platforms.
- [ ] Run golden CiteWeft/GROBID extraction fixtures preserving spans, uncertainty and diagnostics.
- [ ] Run downstream Sourceright consumer-driven compatibility and rollback tests.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-12-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
