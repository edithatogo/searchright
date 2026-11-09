# Plan: 32 Cross-repository contract release train and downstream canaries

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-32`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-32-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `integration/release-train.json`
  - [x] Present source path: `integration/ecosystem-lock.json`
  - [x] Present source path: `release/public-packages.json`
  - [x] Present source path: `contracts/compatibility/schema-surface-0.1.0-alpha.1.json`
  - [x] Present source path: `scripts/check_release_train.py`
  - [x] Present source path: `scripts/sync_ecosystem_lock.py`
  - [x] Present source path: `.github/workflows/integration-release-train.yml`
  - [x] Present source path: `docs/release-train.md`
  - [x] Present source path: `integration/consumer-contract-suite.json`
  - [x] Present source path: `integration/passports/index.json`
  - [x] Present source path: `tests/test_cross_repository_release_train_governance.py`
  - [x] Present source path: `verification/receipts/track-32-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/32-cross-repository-release-train/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-32-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_release_train.py`
  - [x] `python scripts/sync_ecosystem_lock.py --check`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/check_integration_passports.py`
  - [x] `python scripts/check_consumer_contracts.py`
  - [x] `python scripts/check_integration_drift.py`
  - [x] `python3 -m unittest discover -s tests -p test_cross_repository_release_train_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-32-phase-3 -->

- [ ] Run compiler-backed consumer contract suites in each producer and consumer repository.
- [ ] Execute CiteWeft and Sourceright downstream canaries against exact Searchright candidate revisions.
- [ ] Approve promotion or rollback explicitly and record cross-repository receipts.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-32-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_cross_repository_release_train_governance.py with 6 comprehensive unit tests covering release train structure, contiguous promotion order, default-deny automatic promotion across all 6 stages and ecosystem lock, rollback policy without evidence deletion, public package non-publishable bounds, consumer contract suite conformance, and integration release train workflow hardening. Upgraded all 3 assertions (T32-A001..T32-A003) to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 panel receipts and preserved higher evidence gates open.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
