<!-- searchright-issue-key: track-32 -->
# Track 32: Cross-repository contract release train and downstream canaries

Coordinate CiteWeft, Searchright/shared core and Sourceright compatibility without coupling repositories or automatically promoting revisions.

## Source of truth

- Spec: `conductor/tracks/32-cross-repository-release-train/spec.md`
- Plan: `conductor/tracks/32-cross-repository-release-train/plan.md`
- Evidence: `conductor/tracks/32-cross-repository-release-train/evidence.json`

## Contract

- Horizon: `mature`
- Status: `integration_prepared`
- Implementation: `partially_implemented`
- Evidence: `source_verified`
- Dependencies: `13, 14, 15, 18, 31`
- Requirements: `SR-065, SR-070, SR-076, SR-077`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-32-phase-1`)
- [ ] Phase 2: Source-level verification (`track-32-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-32-phase-3`)
- [ ] Phase 4: Review and closeout (`track-32-phase-4`)

## Claim boundary

A source-verified release train does not prove producer-consumer compatibility or publish any component. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
