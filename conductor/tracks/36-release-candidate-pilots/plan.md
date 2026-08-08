# Plan: 36 Release-candidate rehearsal, staged pilots and ecosystem rehearsal

Current status: **release_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-36`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-36-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `release/rehearsal.json`
  - [x] `scripts/check_release_rehearsal.py`
  - [x] `.github/workflows/release-candidate.yml`
  - [x] `docs/releases/release-candidate.md`
  - [x] `docs/pilots/pilot-protocol.md`
  - [x] `registry/status.json`
  - [x] `scripts/package_source.py`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-36-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_release_rehearsal.py`
  - [x] `python scripts/check_packaging_reproducibility.py`
  - [x] `python scripts/check_maturity_dossier.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-36-phase-3 -->

- [ ] Generate a committed lockfile and execute the complete compiler, test, security, reproducibility and attestation matrix.
- [ ] Complete local, institutional self-hosted and remote single-tenant pilot exit decisions with rollback rehearsals.
- [ ] Validate exact release candidate packets and obtain explicit approval before any release or registry submission.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-36-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
