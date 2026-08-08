# Plan: 14 Sourceright migration and shared releases

Current status: **integration_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-14`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-14-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `migration/sourceright/README.md`
  - [x] `migration/sourceright/replacement-map.yaml`
  - [x] `migration/sourceright/parity-cases.json`
  - [x] `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] `scripts/check_sourceright_migration.py`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-14-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_sourceright_migration.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-14-phase-3 -->

- [ ] Open and merge the downstream Sourceright integration after dual-run parity.
- [ ] Generate compiled parity receipts and exercise rollback.
- [ ] Remove superseded downstream code only after the deletion gate passes.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-14-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
