<!-- searchright-issue-key: track-41 -->
# Track 41: Demand-driven incremental living review computation

Recompute living systematic review updates through a demand-driven memoized dependency DAG with early cutoff and bisimulation equivalence to full replay.

## Source of truth

- Spec: `conductor/tracks/41-salsa-incremental-living-reviews/spec.md`
- Plan: `conductor/tracks/41-salsa-incremental-living-reviews/plan.md`
- Evidence: `conductor/tracks/41-salsa-incremental-living-reviews/evidence.json`

## Contract

- Horizon: `mature`
- Status: `scaffolded`
- Implementation: `scaffolded`
- Evidence: `source_verified`
- Dependencies: `06, 07, 08, 22, 24`
- Requirements: `SR-097`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-41-phase-1`)
- [ ] Phase 2: Source-level verification (`track-41-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-41-phase-3`)
- [ ] Phase 4: Review and closeout (`track-41-phase-4`)

## Claim boundary

Source-verified specification and granular execution playbook only; demand-driven memoized DAG evaluation and bisimulation equivalence proofs remain open implementation gates. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
