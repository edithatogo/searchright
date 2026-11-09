<!-- searchright-issue-key: track-22 -->
# Track 22: Living reviews, amendments and update lineage

Make updates, protocol changes, prior-run deduplication and cadence explicit and immutable.

## Source of truth

- Spec: `conductor/tracks/22-living-updates/spec.md`
- Plan: `conductor/tracks/22-living-updates/plan.md`
- Evidence: `conductor/tracks/22-living-updates/evidence.json`

## Contract

- Horizon: `beta`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `05, 06, 07, 08`
- Requirements: `SR-025, SR-045, SR-049, SR-052`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-22-phase-1`)
- [x] Phase 2: Source-level verification (`track-22-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-22-phase-3`)
- [ ] Phase 4: Review and closeout (`track-22-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating living-methodologist, lineage-architect, and governance-auditor roles, with multi-cycle scenarios verified in crates/searchright-living/tests/multi_cycle_scenarios.rs. Panel output is a simulation instrument, not a substitute for live clinical review surveillance or human protocol amendment committee decisions. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
