# Plan: 01 Contract catalogue and code generation

Current status: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-01`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-01-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-contracts/src/lib.rs`
  - [x] `contracts/schema-catalog.json`
  - [x] `contracts/standards/index.json`
  - [x] `docs/contracts.md`
  - [x] `docs/adrs/0008-standards-packs-and-methodology-boundary.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-01-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-01-phase-3 -->

- [ ] Compile Rust contract types and run schema/Rust round-trip tests.
- [ ] Generate compatibility fixtures from compiled public types.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-01-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
