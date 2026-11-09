# Plan: 37 Final mature 1.0 release and long-term operations

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-37`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-37-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `docs/maturity/1.0-gate.md`
  - [x] Present source path: `docs/maturity/release-decision.md`
  - [x] Present source path: `docs/maturity/gap-register.md`
  - [x] Present source path: `conductor/maturity-dossier.json`
  - [x] Present source path: `scripts/check_maturity_dossier.py`
  - [x] Present source path: `ROADMAP.md`
  - [x] Present source path: `PROJECT_STATUS.md`
  - [x] Present source path: `tests/test_maturity_1_0_governance.py`
  - [x] Present source path: `verification/receipts/track-37-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/37-maturity-1-0/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-37-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/run_static_harness.py`
  - [x] `python3 -m unittest discover -s tests -p test_maturity_1_0_governance.py`
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
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_maturity_1_0_governance.py with 6 comprehensive unit tests covering maturity dossier schema conformance across 17 domains, fail-closed decision boundary (not_ready retaining 14 critical domain blockers), exhaustive ready release decision evidence contracts (9 mandatory fields: accountable_reviewer, exact_git_commit, release_candidate, sbom, attestations, downstream_canaries, pilot_exits, rollback_plan, support_plan plus approved: true), status snapshot validation against stale and volatile assertions, and network-free execution of scripts/check_maturity_dossier.py. Upgraded assertions T37-A001..T37-A003 to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 evaluation receipts and preserved open operational gates.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
