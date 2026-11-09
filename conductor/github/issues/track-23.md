<!-- searchright-issue-key: track-23 -->
# Track 23: Active-learning prioritisation and calibrated agents

Provide transparent advisory ranking with uncertainty and no default autonomous exclusion.

## Source of truth

- Spec: `conductor/tracks/23-active-learning-agents/spec.md`
- Plan: `conductor/tracks/23-active-learning-agents/plan.md`
- Evidence: `conductor/tracks/23-active-learning-agents/evidence.json`

## Contract

- Horizon: `beta`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `07, 17, 22`
- Requirements: `SR-046`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-23-phase-1`)
- [x] Phase 2: Source-level verification (`track-23-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-23-phase-3`)
- [ ] Phase 4: Review and closeout (`track-23-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating screening-methodologist, ranking-architect, and safety-governance-auditor roles, with outcome-gradable benchmarks, adversary robustness, and context isolation verified in crates/searchright-ranking/tests/ranking_adversary_scenarios.rs. Simulation instrument only, not a substitute for live prospective clinical review screening or human gold calibration decisions. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
