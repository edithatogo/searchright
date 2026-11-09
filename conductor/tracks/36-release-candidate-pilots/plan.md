# Plan: 36 Release-candidate rehearsal, staged pilots and ecosystem rehearsal

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-36`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-36-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `release/rehearsal.json`
  - [x] Present source path: `scripts/check_release_rehearsal.py`
  - [x] Present source path: `.github/workflows/release-candidate.yml`
  - [x] Present source path: `docs/releases/release-candidate.md`
  - [x] Present source path: `docs/pilots/pilot-protocol.md`
  - [x] Present source path: `registry/status.json`
  - [x] Present source path: `scripts/package_source.py`
  - [x] Present source path: `scripts/check_packaging_reproducibility.py`
  - [x] Present source path: `tests/test_release_candidate_pilots_governance.py`
  - [x] Present source path: `verification/receipts/track-36-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/36-release-candidate-pilots/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-36-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_release_rehearsal.py`
  - [x] `python scripts/check_packaging_reproducibility.py`
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python3 -m unittest discover -s tests -p test_release_candidate_pilots_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-36-phase-3 -->

- [ ] Verify the committed lockfile and execute the complete compiler, test, security, reproducibility and attestation matrix.
- [ ] Complete local, institutional self-hosted and remote single-tenant pilot exit decisions with rollback rehearsals.
- [ ] Validate exact release candidate packets and obtain explicit approval before any release or registry submission.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-36-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_release_candidate_pilots_governance.py with 6 comprehensive unit tests covering release rehearsal script conformance across 11 gates and 3 pilot profiles, mandatory rehearsal safeguards (unexecuted status, dry-run bounds, mandatory rollback), pilot deployment protocol staging (fixture-only local -> institution self-hosted -> remote single-tenant) with PRISMA-S non-negotiable human screening authority and data isolation, ecosystem and registry rehearsal explicit blockers across all 9 submission targets, release-candidate workflow manual dispatch invariants, and reproducible source packaging verification. Upgraded assertions T36-A001..T36-A003 to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 evaluation receipts and preserved open operational gates.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
