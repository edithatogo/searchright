# Plan: 30 Maturity gate and gap closure

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-30`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-30-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `docs/maturity/gap-register.md`
  - [x] `conductor/maturity-dossier.json`
  - [x] `scripts/check_maturity_dossier.py`
  - [x] `PROJECT_STATUS.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-30-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python scripts/check_roadmap_coverage.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-30-phase-3 -->

- [ ] Generate compiler, fixture, live-provider, migration, usability and external-evaluation receipts for every critical domain.
- [ ] Review and approve any explicit release-risk exception without hiding the open gap.
- [ ] Keep autonomous end-to-end review and final agent exclusions outside the release claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-30-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
