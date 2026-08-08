# Plan: 32 Cross-repository contract release train and downstream canaries

Current status: **integration_prepared**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-32`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-32-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
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
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
