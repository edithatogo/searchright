# Plan: 25 Provenance and research-object interoperability

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-25`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-25-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-provenance/src/lib.rs`
  - [x] `docs/adrs/0009-immutable-lineage-and-research-objects.md`
  - [x] `docs/standards-and-provenance.md`
  - [x] `contracts/examples/living-update.yaml`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-25-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
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
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
