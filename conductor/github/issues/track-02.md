<!-- searchright-issue-key: track-02 -->
# Track 02: Portable query AST and dialect compilers

Deliver deterministic, reviewable query translation with explicit fidelity and loss warnings.

## Source of truth

- Spec: `conductor/tracks/02-query-ast-dialects/spec.md`
- Plan: `conductor/tracks/02-query-ast-dialects/plan.md`
- Evidence: `conductor/tracks/02-query-ast-dialects/evidence.json`

## Contract

- Horizon: `foundation`
- Status: `external_evidence_required`
- Implementation: `source_implemented`
- Evidence: `compiler_verified`
- Dependencies: `01`
- Requirements: `SR-005, SR-006, SR-007`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-02-phase-1`)
- [ ] Phase 2: Source-level verification (`track-02-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-02-phase-3`)
- [ ] Phase 4: Review and closeout (`track-02-phase-4`)

## Claim boundary

Compiler-backed tests cover the declared project-authored query subsets, source preservation and fail-closed normalization. Complete vendor-language semantics, cross-database retrieval equivalence, real-filter currency, topic-specific search adequacy and owner approval are not claimed.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
