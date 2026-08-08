# Plan: 35 Generated SDKs, fixture-backed documentation and adoption operations

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-35`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-35-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `sdk/manifest.json`
  - [x] Present source path: `scripts/check_sdk_examples.py`
  - [x] Present source path: `docs/sdk-and-adoption.md`
  - [x] Present source path: `examples/quickstart/README.md`
  - [x] Present source path: `contracts/interface-catalog.json`
  - [x] Present source path: `README.md`
  - [x] Present source path: `GOVERNANCE.md`
  - [x] Assertion ledger: `conductor/tracks/35-sdk-docs-adoption/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-35-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_sdk_examples.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-35-phase-3 -->

- [ ] Generate, compile and install-smoke Python and TypeScript clients from the locked public contracts.
- [ ] Run documentation tests and information-specialist tutorial walkthroughs.
- [ ] Observe support, deprecation and compatibility operations across at least one release cycle.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-35-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
