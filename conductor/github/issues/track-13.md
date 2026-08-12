<!-- searchright-issue-key: track-13 -->
# Track 13: Integration passports, GitHub issue hierarchy and context spine

Make repository boundaries, pinned compatibility, Conductor-to-GitHub hierarchy and agent context machine-readable and drift-checked.

## Source of truth

- Spec: `conductor/tracks/13-integration-passports-github-context/spec.md`
- Plan: `conductor/tracks/13-integration-passports-github-context/plan.md`
- Evidence: `conductor/tracks/13-integration-passports-github-context/evidence.json`

## Contract

- Horizon: `alpha`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `source_verified`
- Dependencies: `00, 01, 03, 09, 10, 11, 12`
- Requirements: `SR-065, SR-066, SR-067, SR-070`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-13-phase-1`)
- [ ] Phase 2: Source-level verification (`track-13-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-13-phase-3`)
- [ ] Phase 4: Review and closeout (`track-13-phase-4`)

## Claim boundary

A durable read-only audit at commit a0660f8 observed the public repository, all 568 then-canonical issues and all 567 expected native subissue relationships with zero drift and zero mutations. Track 13 topology is unchanged in the current source, but the audit does not cover later source revisions, downstream contract execution, scheduled live-estate drift or merged compatibility.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
