<!-- searchright-issue-key: track-26 -->
# Track 26: Formal assurance and contract evolution

Model workflow/authority invariants and govern compatibility, migration, deprecation and rollback.

## Source of truth

- Spec: `conductor/tracks/26-formal-assurance-contract-evolution/spec.md`
- Plan: `conductor/tracks/26-formal-assurance-contract-evolution/plan.md`
- Evidence: `conductor/tracks/26-formal-assurance-contract-evolution/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `01, 07, 10, 13, 22`
- Requirements: `SR-053, SR-054, SR-069`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-26-phase-1`)
- [x] Phase 2: Source-level verification (`track-26-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-26-phase-3`)
- [ ] Phase 4: Review and closeout (`track-26-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating formal-assurance-architect, contract-evolution-specialist, and quality-governance-auditor roles, with finite-state lifecycle transitions, human authority gates, agent screening restrictions, and fail-closed schema migration policies verified in crates/searchright-assurance/tests/assurance_scenarios.rs, scripts/check_schema_migrations.py, and tests/test_mcp_authority_state_machine.py. Simulation instrument and compiler-tested local boundaries only, not a substitute for formal Kani mathematical proofs or production persisted data migration drills. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
