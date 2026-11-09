# Plan: 15 GitHub estate audit and custom-code replacement

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-15`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-15-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `migration/estate/repositories.json`
  - [x] Present source path: `migration/estate/patterns.json`
  - [x] Present source path: `migration/estate/replacement-decisions.json`
  - [x] Present source path: `migration/companion-repositories/index.json`
  - [x] Present source path: `scripts/audit_search_code.py`
  - [x] Present source path: `scripts/check_companion_change_packets.py`
  - [x] Present source path: `docs/estate-integration.md`
  - [x] Present source path: `tests/test_estate_migration.py`
  - [x] Assertion ledger: `conductor/tracks/15-estate-migration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-15-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/audit_search_code.py --self-test`
  - [x] `python scripts/check_companion_change_packets.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-15-phase-3 -->

- [ ] Run repository-specific scans against current local checkouts or GitHub indexes.
- [ ] Open, test and merge replacement changes in each affected repository.
- [ ] Delete duplicate code only after repository-specific parity and rollback evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-15-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `33dd012`: Upgrade Track 15 assertions to symbol_and_gate_mapped with concrete implementation symbols, enhance check_companion_change_packets to validate estate registers, add unit and adversarial test suite in tests/test_estate_migration.py, and record conductor review receipt.
  - Review fix `HEAD`: Verified automated governance tests in tests/test_estate_migration.py, executed agent panel review, verified estate patterns and companion packets, and archived Track 15 with fixture-proven evidence.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
