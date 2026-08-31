<!-- searchright-issue-key: track-04 -->
# Track 04: Open provider connectors MVP

Provide deterministic open-source adapters and opt-in live execution for major discovery sources.

## Source of truth

- Spec: `conductor/tracks/04-open-connectors-mvp/spec.md`
- Plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
- Evidence: `conductor/tracks/04-open-connectors-mvp/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `partially_implemented`
- Implementation: `partially_implemented`
- Evidence: `source_verified`
- Dependencies: `03`
- Requirements: `SR-014, SR-015, SR-016`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-04-phase-1`)
- [ ] Phase 2: Source-level verification (`track-04-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-04-phase-3`)
- [ ] Phase 4: Review and closeout (`track-04-phase-4`)

## Claim boundary

The reviewed parser slice passed 35 focused local tests and strict Clippy, including the premature-empty-page guard; final full validation remains pending. Track 04 remains partially implemented at source_verified. EFetch/full reports, runtime receipt binding, authorised live execution, current provider policy and historical identity migration are unproven; no whole-track completion or archival claim is made.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
