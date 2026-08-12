<!-- searchright-issue-key: track-31 -->
# Track 31: GitHub remote, nested issues and Project v2 control plane

Create and synchronise the remote repository, epic, tracks, phases, tasks, Project fields/views and repository protections from declarative source.

## Source of truth

- Spec: `conductor/tracks/31-github-control-plane/spec.md`
- Plan: `conductor/tracks/31-github-control-plane/plan.md`
- Evidence: `conductor/tracks/31-github-control-plane/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `source_verified`
- Dependencies: `00, 13, 16, 30`
- Requirements: `SR-066, SR-071, SR-072, SR-073, SR-074, SR-075`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-31-phase-1`)
- [ ] Phase 2: Source-level verification (`track-31-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-31-phase-3`)
- [ ] Phase 4: Review and closeout (`track-31-phase-4`)

## Claim boundary

The remote and credential-backed synchronisation have been observed, but the full bootstrap and revised control-plane code require a new exact-merged-main receipt. GitHub remains a coordination projection and cannot promote product implementation, methodology or release maturity.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
