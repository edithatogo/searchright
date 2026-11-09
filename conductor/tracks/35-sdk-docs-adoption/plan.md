# Plan: 35 Generated SDKs, fixture-backed documentation and adoption operations

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-35`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-35-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `sdk/manifest.json`
  - [x] Present source path: `scripts/check_sdk_examples.py`
  - [x] Present source path: `docs/sdk-and-adoption.md`
  - [x] Present source path: `examples/quickstart/README.md`
  - [x] Present source path: `contracts/interface-catalog.json`
  - [x] Present source path: `README.md`
  - [x] Present source path: `GOVERNANCE.md`
  - [x] Present source path: `tests/test_sdk_docs_adoption_governance.py`
  - [x] Present source path: `verification/receipts/track-35-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/35-sdk-docs-adoption/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-35-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_sdk_examples.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/validate_repository.py`
  - [x] `python3 -m unittest discover -s tests -p test_sdk_docs_adoption_governance.py`
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
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_sdk_docs_adoption_governance.py with 6 comprehensive unit tests covering SDK manifest and thin generated client boundaries, contract bindings compiler-checked status across 74 contracts in sdk/generated-contract-bindings.json, 35-operation interface catalog parity across CLI, MCP, and Rust facade, fixture-backed offline tutorials, adoption governance with PRISMA-S reporting contract adherence and human-only screening authority, and public package policy unpublishable bounds. Upgraded assertions T35-A001..T35-A003 to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 evaluation receipts and preserved open operational gates.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
