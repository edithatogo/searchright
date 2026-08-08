# Plan: 20 Grey literature, registers and supplementary discovery

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-20`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-20-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-discovery/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/discovery.rs`
  - [x] Present source path: `contracts/examples/discovery-run.yaml`
  - [x] Present source path: `docs/provider-model.md`
  - [x] Assertion ledger: `conductor/tracks/20-broader-discovery/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-20-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-20-phase-3 -->

- [ ] Compile and run citation-chaining and supplementary-discovery scenarios.
- [ ] Add authorised source-specific live adapters or documented manual methods.
- [ ] Review website and grey-literature methods with information specialists.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-20-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
