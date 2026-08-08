# Plan: 18 Alpha release and distribution

Current status: **release_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-18`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-18-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `.github/workflows/release.yml`
  - [x] `scripts/package_source.py`
  - [x] `Dockerfile`
  - [x] `CITATION.cff`
  - [x] `CHANGELOG.md`
  - [x] `SECURITY.md`
  - [x] `.github/workflows/clean-room.yml`
  - [x] `scripts/check_packaging_reproducibility.py`
  - [x] `scripts/mcp_smoke.py`
  - [x] `verification/harness-matrix.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-18-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_packaging_reproducibility.py`
  - [x] `python scripts/check_workflow_hardening.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-18-phase-3 -->

- [ ] Generate Cargo.lock and pass the full release workflow.
- [ ] Build/install-smoke supported binaries and OCI image.
- [ ] Create signed tag, checksums, attestations and public release artefacts.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-18-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
