# Plan: 15 GitHub estate audit and custom-code replacement

Current status: **integration_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-15`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-15-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `migration/estate/repositories.json`
  - [x] `migration/estate/patterns.json`
  - [x] `migration/estate/replacement-decisions.json`
  - [x] `migration/estate-migration-manifest.yaml`
  - [x] `scripts/audit_search_code.py`
  - [x] `docs/estate-integration.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-15-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/audit_search_code.py --self-test`
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
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
