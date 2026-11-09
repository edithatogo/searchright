<!-- searchright-issue-key: track-25 -->
# Track 25: Provenance and research-object interoperability

Export immutable review lineage as deterministic RO-Crate and W3C PROV research objects.

## Source of truth

- Spec: `conductor/tracks/25-provenance-research-objects/spec.md`
- Plan: `conductor/tracks/25-provenance-research-objects/plan.md`
- Evidence: `conductor/tracks/25-provenance-research-objects/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `05, 22`
- Requirements: `SR-051, SR-052`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-25-phase-1`)
- [x] Phase 2: Source-level verification (`track-25-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-25-phase-3`)
- [ ] Phase 4: Review and closeout (`track-25-phase-4`)

## Claim boundary

Local provenance and review-bundle fixtures passed at f502104a81c43ed0117886ce8cb21f00fdeb7588, including deterministic ordering, input permutation, duplicate-hash and missing-predecessor cases. These self-authored checks do not establish RO-Crate 1.3 or W3C PROV conformance: required root metadata, content-size semantics and entity/activity derivation typing still need correction and an independent consumer. Actual repository deposit remains open. Historical simulated-panel receipts are preparation reviews, not independent acceptance.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
