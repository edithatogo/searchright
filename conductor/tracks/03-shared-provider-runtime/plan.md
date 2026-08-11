# Plan: 03 Shared provider runtime and Sourceright extraction

Current status: **integration_prepared**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-03`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-03-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/evidence-search-core/src/provider.rs`
  - [x] Present source path: `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] Present source path: `migration/sourceright/replacement-map.yaml`
  - [x] Present source path: `migration/sourceright/parity-cases.json`
  - [x] Present source path: `scripts/check_sourceright_migration.py`
  - [x] Present source path: `release/public-packages.json`
  - [x] Present source path: `public-api/README.md`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Assertion ledger: `conductor/tracks/03-shared-provider-runtime/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-03-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_sourceright_migration.py`
  - [x] `python scripts/check_public_package_policy.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-03-phase-3 -->

- [ ] Close hazard H-002 by resolving, validating and pinning complete DNS answers in each live connector transport.
- [ ] Run old/new Sourceright fixtures in the downstream repository.
- [ ] Complete feature-gated Sourceright cutover, semver review and rollback exercise.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-03-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
