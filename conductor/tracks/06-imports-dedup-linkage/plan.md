# Plan: 06 Imports, deduplication and study linkage

Current status: **partial**.

## Phase 1: Implement RIS, CSL JSON, nbib/PubMed XML and CSV readers / Preserve source line/range provenance and malformed-record quarantine

- [ ] Implement RIS, CSL JSON, nbib/PubMed XML and CSV readers
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Preserve source line/range provenance and malformed-record quarantine
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Phase 2: Expand identifier normalization and Unicode title handling / Add report-to-study linkage model and manual merge/split

- [ ] Expand identifier normalization and Unicode title handling
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Add report-to-study linkage model and manual merge/split
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Phase 3: Add large-corpus blocking/indexing without changing deterministic results / Validate against review datasets and adversarial fixtures

- [ ] Add large-corpus blocking/indexing without changing deterministic results
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Validate against review datasets and adversarial fixtures
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Review and closeout

- [ ] Run repository verification and track-specific gates.
- [ ] Record evidence receipt and unresolved blockers.
- [ ] Run Conductor review; append a review-fixes phase for any gaps.
- [ ] Update `conductor/tracks.md` without overstating external completion.
