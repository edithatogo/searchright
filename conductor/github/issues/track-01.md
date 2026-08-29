<!-- searchright-issue-key: track-01 -->
# Track 01: Contract catalogue and code generation

Maintain versioned schemas, examples, standards packs and Rust wire types from one catalogue.

## Source of truth

- Spec: `conductor/tracks/01-contract-catalog/spec.md`
- Plan: `conductor/tracks/01-contract-catalog/plan.md`
- Evidence: `conductor/tracks/01-contract-catalog/evidence.json`

## Contract

- Horizon: `foundation`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `compiler_verified`
- Dependencies: `00`
- Requirements: `SR-001, SR-002, SR-003, SR-004, SR-017, SR-049`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-01-phase-1`)
- [ ] Phase 2: Source-level verification (`track-01-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-01-phase-3`)
- [ ] Phase 4: Review and closeout (`track-01-phase-4`)

## Claim boundary

All 68 canonical schemas/examples and exact digests are checked, contract-only Python and TypeScript types are deterministically generated and compiler-import checked, and 10 explicitly Rust-owned roots are compared with all 246 observed validation-shape differences recorded. Canonical JSON Schema retains validation and binding-generation authority; exact semantic parity, installable clients, publication and downstream Track 35 conformance are not claimed.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
