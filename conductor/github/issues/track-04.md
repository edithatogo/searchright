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

Historical snapshot 8317f4f3211d0f5571c0529a878b6413a12ba44a passed 82 Python tests, 57 static gates, 429 native tests and 23 cargo-vet governance tests using Homebrew Rust/Cargo 1.98.0; it predates the cache-version fix and does not validate that fix. The current cache-version slice passes 61 focused connector tests and strict Clippy on Homebrew 1.98.0; new full validation with repository-pinned 1.97.1 remains pending. Synthetic fixture records bind to issued receipts and version-partitioned memory-cache replay; cache versions are declarations, not parser authenticity or parser execution on replay. Shared live-transport/EFetch wiring and per-HTTP rate admission remain pending; no live/current-policy support, article full text, end-to-end raw-response-to-receipt chain, historical migration or archival claim. Track 04 remains partially_implemented/source_verified.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
