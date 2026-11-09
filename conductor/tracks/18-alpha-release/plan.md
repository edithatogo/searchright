# Plan: 18 Alpha release and distribution

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-18`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-18-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `.github/workflows/clean-room.yml`
  - [x] Present source path: `.github/workflows/release.yml`
  - [x] Present source path: `CHANGELOG.md`
  - [x] Present source path: `CITATION.cff`
  - [x] Present source path: `Dockerfile`
  - [x] Present source path: `SECURITY.md`
  - [x] Present source path: `scripts/check_packaging_reproducibility.py`
  - [x] Present source path: `scripts/mcp_smoke.py`
  - [x] Present source path: `scripts/package_source.py`
  - [x] Present source path: `tests/test_alpha_release_packaging.py`
  - [x] Present source path: `verification/harness-matrix.json`
  - [x] Present source path: `verification/receipts/track-18-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/18-alpha-release/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-18-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_packaging_reproducibility.py`
  - [x] `python scripts/check_workflow_hardening.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-18-phase-3 -->

- [ ] Verify the committed Cargo.lock and pass the full release workflow.
- [ ] Build/install-smoke supported binaries and OCI image.
- [ ] Create signed tag, checksums, attestations and public release artefacts.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-18-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 18 assertions (T18-A001..T18-A006) to symbol_and_gate_mapped with concrete implementation symbols across package_source.py, check_packaging_reproducibility.py, generate_source_sbom.py, generate_source_hash_manifest.py, mcp_smoke.py, check_cli_distribution.py, check_workflow_hardening.py, and check_release_rehearsal.py; refactor package_source and check_packaging_reproducibility to expose modular helper APIs with deterministic header normalization; create unit test suite in tests/test_alpha_release_packaging.py; record conductor review receipt and preserve open release, signing, container, and Cargo.lock blockers without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
