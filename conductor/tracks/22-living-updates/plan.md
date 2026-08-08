# Plan: 22 Living reviews, amendments and update lineage

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-22`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-22-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-living/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/living.rs`
  - [x] Present source path: `crates/searchright-contracts/src/amendment.rs`
  - [x] Present source path: `contracts/examples/living-update.yaml`
  - [x] Present source path: `contracts/examples/protocol-amendment.yaml`
  - [x] Present source path: `docs/adrs/0009-immutable-lineage-and-research-objects.md`
  - [x] Assertion ledger: `conductor/tracks/22-living-updates/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-22-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-22-phase-3 -->

- [ ] Compile and run multi-cycle living-review and amendment scenarios.
- [ ] Pilot a real update and independently reproduce change attribution.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-22-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
