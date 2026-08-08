# Plan: 03 Shared provider runtime and Sourceright extraction

Current status: **integration_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-03`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-03-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/evidence-search-core/src/provider.rs`
  - [x] `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] `migration/sourceright/replacement-map.yaml`
  - [x] `migration/sourceright/parity-cases.json`
  - [x] `scripts/check_sourceright_migration.py`
  - [x] `docs/adrs/0012-sourceright-and-estate-migration.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-03-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_sourceright_migration.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-03-phase-3 -->

- [ ] Compile and run the shared provider runtime tests.
- [ ] Run old/new Sourceright fixtures in the downstream repository.
- [ ] Complete feature-gated Sourceright cutover, semver review and rollback exercise.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-03-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
