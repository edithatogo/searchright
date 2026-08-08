# Plan: 32 Cross-repository contract release train and downstream canaries

Current status: **integration_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-32`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-32-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `integration/release-train.json`
  - [x] `scripts/check_release_train.py`
  - [x] `.github/workflows/integration-release-train.yml`
  - [x] `docs/release-train.md`
  - [x] `integration/consumer-contract-suite.json`
  - [x] `integration/passports/index.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-32-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_release_train.py`
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
