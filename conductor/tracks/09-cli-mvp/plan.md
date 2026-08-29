# Plan: 09 CLI MVP

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.
Lifecycle: **archived** on **2026-08-29**; canonical source and GitHub keys are retained.


GitHub issue key: `track-09`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-09-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `.github/workflows/clean-room.yml`
  - [x] Present source path: `.github/workflows/ci.yml`
  - [x] Present source path: `crates/searchright-cli/assets/completions/`
  - [x] Present source path: `crates/searchright-cli/tests/cli_e2e.rs`
  - [x] Present source path: `crates/searchright-cli/tests/snapshots/`
  - [x] Present source path: `crates/searchright-cli/src/main.rs`
  - [x] Present source path: `crates/searchright-licensed/src/lib.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `contracts/interface-catalog.json`
  - [x] Present source path: `docs/adrs/0007-shared-application-facade.md`
  - [x] Present source path: `docs/cli-compatibility.md`
  - [x] Present source path: `scripts/check_cli_distribution.py`
  - [x] Present source path: `supply-chain/audits.toml`
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

- [x] Run the exact Track 09 implementation head on hosted Linux, Windows and macOS and preserve successful PR check evidence before semantic archival.
- [x] Bind the sealed correctness, security, testing, methodology and adversarial panel disposition to the exact reviewed implementation head before semantic archival.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-09-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `8f0bfa2dbc125db339b5cb9b84a9bcd2f192d03e`: Replace reflected parser failures with a stable non-sensitive JSON usage error while completing grouped CLI and distribution coverage.
  - Review fix `dc28b6ac2221aa43a05f1ddf0b0c9641a63fd513`: Exercise built and installed CLI snapshots on the hosted Linux, macOS and Windows CI matrix, including Windows executable resolution.
  - Review fix `de00f29127b303824b4e643e7fd5d573fcfc1451`: Close no-clobber, canonical dispatch, actionable diagnostic, endpoint credential-reflection and distribution-coverage review findings.
  - Review fix `46945efd3726b451674cdff8555b29a671df147b`: Close architecture-fitness and generated SBOM/source-manifest review gates on the sealed candidate.
  - Review fix `ac403d4bd37fd3365f1d1be6b42082aafad4fc42`: Close hosted Windows snapshot, patch coverage and supply-chain findings with narrow executable-name normalization, static five-shell assets, exact dependency audits and removal of the rejected dynamic-completion dependency.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
