# Plan: 25 Provenance and research-object interoperability

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-25`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-25-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-provenance/src/lib.rs`
  - [x] Present source path: `scripts/review_bundle.py`
  - [x] Present source path: `contracts/json-schema/review-bundle-manifest.v1.schema.json`
  - [x] Present source path: `contracts/examples/review-bundle-manifest.json`
  - [x] Present source path: `examples/review-bundle/example.srpack`
  - [x] Present source path: `docs/standards-and-provenance.md`
  - [x] Present source path: `docs/vertical-slice-definition-of-done.md`
  - [x] Present source path: `tests/test_provenance_research_objects_governance.py`
  - [x] Assertion ledger: `conductor/tracks/25-provenance-research-objects/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-25-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/review_bundle.py self-test`
  - [x] `python scripts/validate_repository.py`
  - [x] `python3 -B -m unittest discover -s tests -p test_provenance_research_objects_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-25-phase-3 -->

- [ ] Compile and run golden provenance export tests.
- [ ] Validate produced RO-Crates with independent tooling.
- [ ] Pilot OSF/Zenodo/repository deposit without automatic write authority.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-25-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 25 assertions (T25-A001..T25-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-provenance, review_bundle.py, RO-Crate and W3C PROV boundaries, review-bundle-manifest and handoff-plan schemas, and test_provenance_research_objects_governance.py; create unit test suite in tests/test_provenance_research_objects_governance.py validating RO-Crate 1.3 schema and context, W3C PROV entities/activities/agents/predecessor derivations, review bundle packaging and deterministic policy invariants, default-deny handoff plans, and Track 25 traceability invariants; record conductor review receipt and preserve open golden provenance export tests, independent RO-Crate validation tooling, and OSF/Zenodo repository handoff pilot blockers without false promotion.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
