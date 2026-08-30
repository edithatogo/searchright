<!-- searchright-issue-key: track-11 -->
# Track 11: Systematic-search agent skill and workflows

Package planning, PRESS, execution, deduplication, screening and reporting workflows with explicit human checkpoints.

## Source of truth

- Spec: `conductor/tracks/11-agentic-skill/spec.md`
- Plan: `conductor/tracks/11-agentic-skill/plan.md`
- Evidence: `conductor/tracks/11-agentic-skill/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `partially_implemented`
- Implementation: `partially_implemented`
- Evidence: `compiler_verified`
- Dependencies: `10`
- Requirements: `SR-022, SR-026, SR-031`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-11-phase-1`)
- [ ] Phase 2: Source-level verification (`track-11-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-11-phase-3`)
- [ ] Phase 4: Review and closeout (`track-11-phase-4`)

## Claim boundary

Compiler-verified local workflow, handoff and deterministic authority behaviour are established. The initial Codex receipt remains invalidated; a fresh context-isolated Codex CLI 0.144.1 / gpt-5.6-sol run passed all twelve public authority fixtures with strict event integrity. This is bounded fixture compliance, not general host support or methodological calibration. Claude evaluation remains pending. The downstream caller has a focused-tested draft upstream contribution; full consumer validation, upstream adoption and maintenance acceptance, calibration-policy reconciliation and accountable adjudication, and registry selection, authorization and acceptance remain pending.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
