<!-- searchright-issue-key: track-08 -->
# Track 08: PRISMA, PRESS and reporting

Render PRISMA flow/appendix outputs and PRESS/standards assessments directly from evidence without conflating reporting and conduct.

## Source of truth

- Spec: `conductor/tracks/08-prisma-press-reporting/spec.md`
- Plan: `conductor/tracks/08-prisma-press-reporting/plan.md`
- Evidence: `conductor/tracks/08-prisma-press-reporting/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `01, 05, 07`
- Requirements: `SR-023, SR-024, SR-025, SR-026, SR-027`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-08-phase-1`)
- [ ] Phase 2: Source-level verification (`track-08-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-08-phase-3`)
- [ ] Phase 4: Review and closeout (`track-08-phase-4`)

## Claim boundary

All six Track 08 source assertions have deterministic assertion-level mappings at the source-verified evidence ceiling. PRISMA and PRISMA-S outputs are reporting artefacts, not conduct certification; the checked-in PRESS fixture is not independent review evidence; complete PRISMA.jl model equivalence is not claimed.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
