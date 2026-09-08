<!-- searchright-issue-key: track-00 -->
# Track 00: Foundation, Conductor and toolchain

Establish the Git repository, Conductor context, pinned toolchain, standards inheritance and reproducible bootstrap.

## Source of truth

- Spec: `conductor/tracks/00-foundation-conductor-toolchain/spec.md`
- Plan: `conductor/tracks/00-foundation-conductor-toolchain/plan.md`
- Evidence: `conductor/tracks/00-foundation-conductor-toolchain/evidence.json`

## Contract

- Horizon: `foundation`
- Status: `external_evidence_required`
- Implementation: `external_evidence_required`
- Evidence: `source_verified`
- Dependencies: `none`
- Requirements: `none`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-00-phase-1`)
- [ ] Phase 2: Source-level verification (`track-00-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-00-phase-3`)
- [ ] Phase 4: Review and closeout (`track-00-phase-4`)

## Claim boundary

Source-verified foundation implementation with exact 8de03a0 whole-tree-equivalent full verification: 87 Python tests, 57 static gates and 392 native tests passed. A separate frozen cargo-vet policy check passed with 43 fully vetted dependencies and 259 existing exemptions, not 259 audited dependencies. Earlier failed receipts remain historical; upstream repository-standards registration and estate conformance remain open. No future-tree validation or track completion is implied.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
