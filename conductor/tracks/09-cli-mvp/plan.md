# Plan: 09 CLI MVP

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-09`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-09-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-cli/src/main.rs`
  - [x] `crates/searchright/src/engine.rs`
  - [x] `contracts/interface-catalog.json`
  - [x] `docs/adrs/0007-shared-application-facade.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-09-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-09-phase-3 -->

- [ ] Compile binaries and run cross-platform help, JSON, error and installation snapshots.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-09-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
