<!-- searchright-issue-key: track-03 -->
# Track 03: Shared provider runtime and Sourceright extraction

Centralise bounded provider execution, caching, receipts and policy while preparing reversible Sourceright adoption.

## Source of truth

- Spec: `conductor/tracks/03-shared-provider-runtime/spec.md`
- Plan: `conductor/tracks/03-shared-provider-runtime/plan.md`
- Evidence: `conductor/tracks/03-shared-provider-runtime/evidence.json`

## Contract

- Horizon: `foundation`
- Status: `integration_prepared`
- Implementation: `partially_implemented`
- Evidence: `source_verified`
- Dependencies: `01, 02`
- Requirements: `SR-008, SR-009, SR-010, SR-011`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-03-phase-1`)
- [ ] Phase 2: Source-level verification (`track-03-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-03-phase-3`)
- [ ] Phase 4: Review and closeout (`track-03-phase-4`)

## Claim boundary

Locally compiler-tested Searchright implementation and a read-only Sourceright baseline, retained at the repository-wide source-verified ceiling until exact-revision CI evidence exists. The additive core-owned subrequest context has 74 focused native tests and strict Clippy passing; full validation for this slice and live-adapter adoption remain pending. Registry-local cooperative admission is not process-wide rate enforcement or current provider-policy approval; no live provider, downstream integration, dual-run, SemVer, rollback, release or archival claim is made.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
