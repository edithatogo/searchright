<!-- searchright-issue-key: track-36 -->
# Track 36: Release-candidate rehearsal, staged pilots and ecosystem rehearsal

Exercise the complete candidate in clean-room builds, downstream canaries, bounded pilots, rollback and registry submission rehearsals.

## Source of truth

- Spec: `conductor/tracks/36-release-candidate-pilots/spec.md`
- Plan: `conductor/tracks/36-release-candidate-pilots/plan.md`
- Evidence: `conductor/tracks/36-release-candidate-pilots/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `18, 19, 29, 30, 31, 32, 33, 34, 35`
- Requirements: `SR-039, SR-040, SR-068, SR-089, SR-090, SR-091`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-36-phase-1`)
- [ ] Phase 2: Source-level verification (`track-36-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-36-phase-3`)
- [ ] Phase 4: Review and closeout (`track-36-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating release-integrity-architect, pilot-operations-auditor, and ecosystem-registry-auditor roles, with clean-room rehearsal validation in scripts/check_release_rehearsal.py, deterministic packaging reproducibility in scripts/check_packaging_reproducibility.py, staged pilot safety protocols in docs/pilots/pilot-protocol.md, no autonomous exclusions, deployment-controlled data boundaries, explicit registry blockers across 9 submission targets in registry/status.json, and 6/6 unit tests verified network-free in tests/test_release_candidate_pilots_governance.py. Simulation instrument and source-level contracts only; live pilot execution, compiler matrix runs, and public registry submissions remain open operational gates. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
