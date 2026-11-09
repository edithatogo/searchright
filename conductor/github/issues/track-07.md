<!-- searchright-issue-key: track-07 -->
# Track 07: Governed screening workflow

Support independent title/abstract and full-text decisions, conflicts, adjudication, roles and conservative agent authority.

## Source of truth

- Spec: `conductor/tracks/07-screening-workflow/spec.md`
- Plan: `conductor/tracks/07-screening-workflow/plan.md`
- Evidence: `conductor/tracks/07-screening-workflow/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `01, 06`
- Requirements: `SR-020, SR-021, SR-022, SR-058`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-07-phase-1`)
- [x] Phase 2: Source-level verification (`track-07-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-07-phase-3`)
- [ ] Phase 4: Review and closeout (`track-07-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating screening-reviewer, information-specialist, and methodologist roles. Panel output is a simulation instrument, not a substitute for live human studies. Remediation targets: authorization integrity, blinding enforcement, audit-trail immutability, interop formats. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
