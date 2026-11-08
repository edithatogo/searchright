# Plan: 00 Foundation, Conductor and toolchain

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-00`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-00-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `Cargo.toml`
  - [x] Present source path: `rust-toolchain.toml`
  - [x] Present source path: `conductor/upstream.lock.json`
  - [x] Present source path: `conductor/index.md`
  - [x] Present source path: `conductor/product.md`
  - [x] Present source path: `conductor/workflow.md`
  - [x] Present source path: `Cargo.lock`
  - [x] Present source path: `docs/repository-standards-alignment.md`
  - [x] Present source path: `verification/receipts/track-00-local-verification.json`
  - [x] Present source path: `verification/receipts/track-00-conductor-review.json`
  - [x] Present source path: `scripts/install-conductor.sh`
  - [x] Present source path: `scripts/install-conductor.ps1`
  - [x] Present source path: `scripts/conductor_status.py`
  - [x] Present source path: `tests/test_conductor_status.py`
  - [x] Present source path: `tests/test_foundation_toolchain_governance.py`
  - [x] Present source path: `verification/receipts/track-00-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/00-foundation-conductor-toolchain/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-00-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_context_integrity.py`
  - [x] `python scripts/check_toolchain_manifest.py`
  - [x] `python3 -m unittest discover -s tests -p test_foundation_toolchain_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-00-phase-3 -->

- [x] Register Searchright in edithatogo/repository-standards and execute estate conformance at an approved upstream revision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-00-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 00 assertions (T00-A001..T00-A006) to symbol_and_gate_mapped with concrete implementation symbols and deterministic tests; record conductor review receipt and preserve the open upstream repository-standards estate conformance gate.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.

- [x] Add a read-only native Conductor v3 status adapter with adversarial checks (`9b73b49`, review fixes `f47c92a`), preserving stable IDs, evidence-aware states and in-place archives; verification/receipts/track-00-native-status-adapter.json records bounded evidence.
- [x] Reconcile historical aggregate-verification failures with exact-tree full verification and separate frozen cargo-vet policy evidence (`f372c67`); verification/receipts/track-00-verification-reconciliation.json preserves exemption limits and the unresolved upstream registration gate.
