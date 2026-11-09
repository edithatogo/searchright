# Plan: 14 Sourceright migration and shared releases

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-14`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-14-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `migration/sourceright/README.md`
  - [x] Present source path: `migration/sourceright/replacement-map.yaml`
  - [x] Present source path: `migration/sourceright/parity-cases.json`
  - [x] Present source path: `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] Present source path: `scripts/check_sourceright_migration.py`
  - [x] Present source path: `migration/companion-repositories/sourceright.json`
  - [x] Present source path: `docs/integration-architecture.md`
  - [x] Present source path: `tests/test_sourceright_migration_governance.py`
  - [x] Assertion ledger: `conductor/tracks/14-sourceright-migration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-14-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_sourceright_migration.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-14-phase-3 -->

- [x] Open and merge the downstream Sourceright integration after dual-run parity.
- [x] Generate compiled parity receipts and exercise rollback.
- [x] Remove superseded downstream code only after the deletion gate passes.
- [x] Publish and evaluate Sourceright scholarly-integrity signals with a Searchright advisory-only consumer contract, provenance fixtures and false-positive review.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-14-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `97c36d0`: Upgrade Track 14 assertions to symbol_and_gate_mapped with native tests, add unit tests for LegacyRuntimeConfig execution_policy and provider_mode in searchright-sourceright-compat, refactor check_sourceright_migration main entry point, and record conductor review receipt.
  - Review fix `HEAD`: Implemented tests/test_sourceright_migration_governance.py, executed agent panel review, verified replacement maps and parity matrices, and archived Track 14 with fixture-proven evidence.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
