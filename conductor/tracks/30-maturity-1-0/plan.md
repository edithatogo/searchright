# Plan: 30 Mature 1.0 product, community and evaluation

Current status: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-30`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-30-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `docs/maturity/1.0-gate.md`
  - [x] `conductor/maturity-dossier.json`
  - [x] `ROADMAP.md`
  - [x] `PROJECT_STATUS.md`
  - [x] `conductor/requirements.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-30-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-30-phase-3 -->

- [ ] Pass compiler, fixture, live-provider, interoperability, security and usability gates.
- [ ] Complete Sourceright and estate migrations with rollback evidence.
- [ ] Complete independent methodological evaluation.
- [ ] Publish signed 1.0 release and record registry/community evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-30-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
