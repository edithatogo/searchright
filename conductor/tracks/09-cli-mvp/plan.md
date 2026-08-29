# Plan: 09 CLI MVP

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-09`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-09-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `.github/workflows/clean-room.yml`
  - [x] Present source path: `crates/searchright-cli/tests/cli_e2e.rs`
  - [x] Present source path: `crates/searchright-cli/tests/snapshots/`
  - [x] Present source path: `crates/searchright-cli/src/main.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `contracts/interface-catalog.json`
  - [x] Present source path: `docs/adrs/0007-shared-application-facade.md`
  - [x] Present source path: `docs/cli-compatibility.md`
  - [x] Present source path: `scripts/check_cli_distribution.py`
  - [x] Assertion ledger: `conductor/tracks/09-cli-mvp/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-09-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `cargo test -p searchright-cli --locked`
  - [x] `cargo clippy -p searchright-cli --all-targets --locked -- -D warnings`
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/check_cli_distribution.py target/debug/searchright`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-09-phase-3 -->

- [ ] Obtain hosted Linux, macOS and Windows CLI snapshot receipts for the exact committed revision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-09-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `8f0bfa2dbc125db339b5cb9b84a9bcd2f192d03e`: Replace reflected parser failures with a stable non-sensitive JSON usage error while completing grouped CLI and distribution coverage.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
