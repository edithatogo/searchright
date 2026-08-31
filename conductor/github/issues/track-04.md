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

Full local validation of the cache-version slice passed on snapshot 6d995108106db62474e2746a2021f29e89603b95 using repository-pinned Rust/Cargo 1.97.1: 82 Python tests, 57 static gates, 431 native tests with zero skipped and 23 cargo-vet governance tests. Earlier 61 focused connector tests and strict Clippy used Homebrew 1.98.0; historical snapshot 8317 and its 429 native tests predate the cache-version fix. Synthetic fixture records bind to issued receipts and version-partitioned memory-cache replay; declared versions do not establish authenticity or parser execution on replay. Future EFetch orchestration adoption is not part of this proof. Shared live-transport/EFetch wiring, per-HTTP admission, exact-head hosted evidence and current-policy/rights approval remain open; no live support, article full text, raw-response-to-issued-receipt chain, historical migration or archival claim. Track 04 remains partially_implemented/source_verified.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
