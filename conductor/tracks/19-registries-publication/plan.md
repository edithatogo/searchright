# Plan: 19 Registries and scholarly publication

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-19`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-19-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `glama.json`
  - [x] Present source path: `registry/glama/README.md`
  - [x] Present source path: `registry/joss/paper.md`
  - [x] Present source path: `registry/official-mcp/README.md`
  - [x] Present source path: `registry/smithery/README.md`
  - [x] Present source path: `registry/status.json`
  - [x] Present source path: `server.json`
  - [x] Present source path: `tests/test_registries_publication_governance.py`
  - [x] Present source path: `verification/receipts/track-19-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/19-registries-publication/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-19-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-19-phase-3 -->

- [ ] Publish a verified release that registry packets can reference.
- [ ] Submit to each external registry with maintainer approval.
- [ ] Record public listing, rejection or revision evidence.
- [ ] Complete JOSS submission requirements and independent review.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-19-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 19 assertions (T19-A001..T19-A006) to symbol_and_gate_mapped with concrete implementation symbols across validate_repository.py, submit-registries.sh, check_public_package_policy.py, and test_registries_publication_governance.py; create unit test suite in tests/test_registries_publication_governance.py validating registry packet schemas, invariants, dry-run guards, and citation alignment; record conductor review receipt and preserve open external publication, registration, and JOSS review blockers without false promotion.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
