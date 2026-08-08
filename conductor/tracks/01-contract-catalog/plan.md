# Plan: 01 Contract catalogue and code generation

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-01`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-01-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/evidence-search-contracts/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/lib.rs`
  - [x] Present source path: `contracts/schema-catalog.json`
  - [x] Present source path: `contracts/compatibility/schema-surface-0.1.0-alpha.1.json`
  - [x] Present source path: `scripts/sync_schema_surface.py`
  - [x] Present source path: `release/public-packages.json`
  - [x] Present source path: `scripts/check_public_package_policy.py`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Assertion ledger: `conductor/tracks/01-contract-catalog/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-01-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/check_public_package_policy.py`
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
