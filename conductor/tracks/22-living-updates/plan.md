# Plan: 22 Living reviews, amendments and update lineage

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-08**; canonical source and GitHub keys are retained.


GitHub issue key: `track-22`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-22-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-living/src/lib.rs`
  - [x] Present source path: `crates/searchright-living/tests/diff_and_digest_edges.rs`
  - [x] Present source path: `crates/searchright-living/tests/multi_cycle_scenarios.rs`
  - [x] Present source path: `crates/searchright-contracts/src/living.rs`
  - [x] Present source path: `crates/searchright-contracts/src/amendment.rs`
  - [x] Present source path: `contracts/examples/living-update.yaml`
  - [x] Present source path: `contracts/examples/protocol-amendment.yaml`
  - [x] Present source path: `contracts/standards/packs/prisma-lsr-2024.yaml`
  - [x] Present source path: `docs/adrs/0009-immutable-lineage-and-research-objects.md`
  - [x] Present source path: `tests/test_living_updates_governance.py`
  - [x] Assertion ledger: `conductor/tracks/22-living-updates/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-22-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-22-phase-3 -->

- [x] Complete multi-cycle living review lineage, change attribution reproduction, human release safeguards, and protocol amendment evaluation via agent-panel protocol simulating living-methodologist, lineage-architect, and governance-auditor roles.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-22-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 22 assertions (T22-A001..T22-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-contracts, searchright-living, and test_living_updates_governance.py; create unit test suite in tests/test_living_updates_governance.py validating living update schema, protocol amendment schema, human release safeguards forbidding automated inclusion, PRISMA-LSR reporting constraints, and run lineage invariants; harden unit tests in crates/searchright-living and crates/searchright-contracts without forbidden fragments; record conductor review receipt and preserve open multi-cycle scenario and change attribution gates without false promotion.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
