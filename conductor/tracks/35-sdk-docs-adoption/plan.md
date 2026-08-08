# Plan: 35 Generated SDKs, fixture-backed documentation and adoption operations

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-35`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-35-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `sdk/manifest.json`
  - [x] `scripts/check_sdk_examples.py`
  - [x] `docs/sdk-and-adoption.md`
  - [x] `examples/quickstart/README.md`
  - [x] `contracts/interface-catalog.json`
  - [x] `README.md`
  - [x] `GOVERNANCE.md`

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
