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

The additive core-owned subrequest context passed full local validation on snapshot 209f3cd3838d22e0b7a6170b19a03571e154b257 with repository-pinned Rust/Cargo 1.97.1: 75 Python tests, 57 static gates, 395 native tests with zero skipped, and 23 cargo-vet governance tests. The earlier 74 focused native tests and strict Clippy used Homebrew 1.98.0. Evidence remains source_verified/partially_implemented; the validated snapshot is not a claim about merged current main. Registry-local cooperative admission is not process-wide rate enforcement or current provider-policy approval. Live-adapter adoption, transport adversarial/hosted evidence, complete downstream parity matrix, Sourceright cutover, SemVer and rollback remain open; no live, release or archival claim.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
