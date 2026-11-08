<!-- searchright-issue-key: track-28 -->
# Track 28: Institutional governance, privacy and collaboration

Evaluate data handling and least-privilege collaboration policy before sensitive or cross-institution operations.

## Source of truth

- Spec: `conductor/tracks/28-institutional-governance-privacy-collaboration/spec.md`
- Plan: `conductor/tracks/28-institutional-governance-privacy-collaboration/plan.md`
- Evidence: `conductor/tracks/28-institutional-governance-privacy-collaboration/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `05, 07, 13, 24, 27`
- Requirements: `SR-057, SR-058, SR-062`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-28-phase-1`)
- [ ] Phase 2: Source-level verification (`track-28-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-28-phase-3`)
- [ ] Phase 4: Review and closeout (`track-28-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating governance architect, privacy specialist, and adjudication auditor roles, with deny-by-default boundary enforcement, immutable audit records, legal hold protection, and cryptographic approval registry verification in crates/searchright-governance/tests/governance_scenarios.rs. Simulation instrument and compiler-tested local boundaries only, not a substitute for external institutional compliance audit. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
