# Plan: 21 Licensed BYO-access adapters

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-08**; canonical source and GitHub keys are retained.


GitHub issue key: `track-21`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-21-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-licensed/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/licensed.rs`
  - [x] Present source path: `contracts/licensed/index.json`
  - [x] Present source path: `contracts/licensed/embase.yaml`
  - [x] Present source path: `contracts/licensed/scopus.yaml`
  - [x] Present source path: `contracts/licensed/web-of-science.yaml`
  - [x] Present source path: `contracts/examples/licensed-adapter.yaml`
  - [x] Present source path: `tests/test_licensed_adapters_governance.py`
  - [x] Present source path: `verification/receipts/track-21-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/21-licensed-adapters/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-21-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-21-phase-3 -->

- [x] Complete licensed BYO request planning, credential protection, and legal compliance evaluation via agent-panel protocol simulating licensed-access-evaluator, legal-compliance-counsel, and security-auditor roles.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-21-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 21 assertions (T21-A001..T21-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-contracts, searchright-licensed, contracts/licensed, and test_licensed_adapters_governance.py; create unit test suite in tests/test_licensed_adapters_governance.py validating adapter index schema, profile invariants, bare DNS host allowlists, zero raw response persistence, and scraping prohibition; harden unit tests in crates/searchright-licensed and crates/searchright-contracts without forbidden fragments; record conductor review receipt and preserve open vendor credentials, terms confirmation, and live smoke receipts without false promotion.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
