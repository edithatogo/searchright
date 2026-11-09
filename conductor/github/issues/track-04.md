<!-- searchright-issue-key: track-04 -->
# Track 04: Open provider connectors MVP

Provide deterministic open-source adapters and opt-in live execution for major discovery sources.

## Source of truth

- Spec: `conductor/tracks/04-open-connectors-mvp/spec.md`
- Plan: `conductor/tracks/04-open-connectors-mvp/plan.md`
- Evidence: `conductor/tracks/04-open-connectors-mvp/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `03`
- Requirements: `SR-014, SR-015, SR-016`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-04-phase-1`)
- [x] Phase 2: Source-level verification (`track-04-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-04-phase-3`)
- [ ] Phase 4: Review and closeout (`track-04-phase-4`)

## Claim boundary

Full local validation of the cache-version slice and bounded opt-in EFetch orchestration passed on repository-pinned Rust/Cargo 1.97.1 with Homebrew 1.98.0 / 1.99.0 compiler verification: 11 focused Python unit tests, repository-wide validation gates, 75 native connector tests with zero skipped and strict Clippy. Scripted double execution verifies per-subrequest rate admission, isolated cache identities, and shared NCBI rate grouping under live-disabled execution; complete synthetic-page goldens and scripted double execution do not prove live network retrieval, article full text or actual search completeness. Shared live-transport/EFetch network wiring, per-HTTP admission, exact-head hosted evidence and current-policy/rights approval remain open; no live support, article full text, raw-response-to-issued-receipt chain, historical migration or archival claim. Source mappings and local fixtures are separate from the remaining live-product and provider-support requirements.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
