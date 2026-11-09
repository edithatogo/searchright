<!-- searchright-issue-key: track-32 -->
# Track 32: Cross-repository contract release train and downstream canaries

Coordinate CiteWeft, Searchright/shared core and Sourceright compatibility without coupling repositories or automatically promoting revisions.

## Source of truth

- Spec: `conductor/tracks/32-cross-repository-release-train/spec.md`
- Plan: `conductor/tracks/32-cross-repository-release-train/plan.md`
- Evidence: `conductor/tracks/32-cross-repository-release-train/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `13, 14, 15, 18, 31`
- Requirements: `SR-065, SR-070, SR-076, SR-077`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-32-phase-1`)
- [x] Phase 2: Source-level verification (`track-32-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-32-phase-3`)
- [ ] Phase 4: Review and closeout (`track-32-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating release-train-engineer, downstream-canary-specialist, and rollback-authority-auditor roles, with contiguous promotion order (CiteWeft -> Searchright -> Sourceright), default-deny automation across all 6 lifecycle stages, 4-step rollback with zero evidence deletion, consumer contract suite conformance, and hardened CI workflows verified in tests/test_cross_repository_release_train_governance.py. Simulation instrument and source-level contracts only; live downstream canaries in CiteWeft and Sourceright and production release promotion remain strictly blocked. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
