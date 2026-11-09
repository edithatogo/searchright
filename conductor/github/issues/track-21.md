<!-- searchright-issue-key: track-21 -->
# Track 21: Licensed BYO-access adapters

Provide credential-free request planning and explicit licence/capability profiles for Embase, Scopus and Web of Science.

## Source of truth

- Spec: `conductor/tracks/21-licensed-adapters/spec.md`
- Plan: `conductor/tracks/21-licensed-adapters/plan.md`
- Evidence: `conductor/tracks/21-licensed-adapters/evidence.json`

## Contract

- Horizon: `beta`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `03, 13, 16`
- Requirements: `SR-044`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-21-phase-1`)
- [x] Phase 2: Source-level verification (`track-21-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-21-phase-3`)
- [ ] Phase 4: Review and closeout (`track-21-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating licensed-access-evaluator, legal-compliance-counsel, and security-auditor roles. Panel output is a simulation instrument, not a substitute for vendor licensing agreements or live user-credentialed audits. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
