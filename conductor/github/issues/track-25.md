<!-- searchright-issue-key: track-25 -->
# Track 25: Provenance and research-object interoperability

Export immutable review lineage as deterministic RO-Crate and W3C PROV research objects.

## Source of truth

- Spec: `conductor/tracks/25-provenance-research-objects/spec.md`
- Plan: `conductor/tracks/25-provenance-research-objects/plan.md`
- Evidence: `conductor/tracks/25-provenance-research-objects/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `05, 22`
- Requirements: `SR-051, SR-052`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-25-phase-1`)
- [ ] Phase 2: Source-level verification (`track-25-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-25-phase-3`)
- [ ] Phase 4: Review and closeout (`track-25-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating ro-crate-interop-specialist, prov-lineage-architect, and open-science-governance-auditor roles, with RO-Crate 1.3 profile conformance, W3C PROV-O derivation chains, and fail-closed predecessor validation verified in crates/searchright-provenance/tests/provenance_interop_scenarios.rs and review_bundle.py self-test. Simulation instrument and compiler-tested local boundaries only, not a substitute for live external repository deposit or independent RO-Crate CLI validation. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
