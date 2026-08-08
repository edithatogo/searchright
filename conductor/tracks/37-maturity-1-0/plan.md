# Plan: 37 Final mature 1.0 release and long-term operations

Current status: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-37`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-37-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `docs/maturity/1.0-gate.md`
  - [x] `docs/maturity/release-decision.md`
  - [x] `conductor/maturity-dossier.json`
  - [x] `scripts/check_maturity_dossier.py`
  - [x] `ROADMAP.md`
  - [x] `PROJECT_STATUS.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-37-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/run_static_harness.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-37-phase-3 -->

- [ ] Pass every critical maturity domain and resolve or explicitly reject all release-risk exceptions.
- [ ] Approve and sign the release decision tied to the exact Git commit, candidate, SBOM, attestations and downstream/pilot evidence.
- [ ] Publish version 1.0 and separately record public release, registry, software-paper, support and maintenance evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-37-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
