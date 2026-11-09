<!-- searchright-issue-key: track-27 -->
# Track 27: Accessibility, internationalisation and usability

Provide stable accessible diagnostics and locale-neutral contracts, then validate them with users and assistive technology.

## Source of truth

- Spec: `conductor/tracks/27-accessibility-internationalisation-usability/spec.md`
- Plan: `conductor/tracks/27-accessibility-internationalisation-usability/plan.md`
- Evidence: `conductor/tracks/27-accessibility-internationalisation-usability/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `09, 10, 11`
- Requirements: `SR-055, SR-056`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-27-phase-1`)
- [x] Phase 2: Source-level verification (`track-27-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-27-phase-3`)
- [ ] Phase 4: Review and closeout (`track-27-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating accessibility-systems-architect, internationalisation-specialist, and usability-governance-auditor roles, with accessible color-free PlainText/JSON/JSONL rendering, word-based severities, separable message catalogues, and external evaluation protocols verified in crates/searchright-diagnostics/tests/diagnostic_scenarios.rs and evaluation/external-evaluation.json. Simulation instrument and compiler-tested local boundaries only, not a substitute for live participant usability trials or formal ethics clearance. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
