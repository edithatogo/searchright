<!-- searchright-issue-key: track-12 -->
# Track 12: CiteWeft scholarly extraction and document evidence

Integrate CiteWeft through a pinned optional adapter while preserving spans, uncertainty, provenance and the no-canonical-write boundary.

## Source of truth

- Spec: `conductor/tracks/12-citeweft-document-evidence/spec.md`
- Plan: `conductor/tracks/12-citeweft-document-evidence/plan.md`
- Evidence: `conductor/tracks/12-citeweft-document-evidence/evidence.json`

## Contract

- Horizon: `alpha`
- Status: `integration_prepared`
- Implementation: `partially_implemented`
- Evidence: `source_verified`
- Dependencies: `01, 06, 07`
- Requirements: `SR-063, SR-064`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-12-phase-1`)
- [ ] Phase 2: Source-level verification (`track-12-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-12-phase-3`)
- [ ] Phase 4: Review and closeout (`track-12-phase-4`)

## Claim boundary

CiteWeft-compatible extraction evidence is source-implemented; no GROBID compatibility, canonicalisation or full-text production claim is made. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
