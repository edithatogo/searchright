# Plan: 41 Temporal acquisition and versioned living-search corpora

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

## Phase 1: Source implementation

<!-- github-subissue-key: track-41-phase-1 -->

- [ ] Implement the new scope and bind assertions to actual source symbols.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-41-phase-2 -->

- [x] Historical network-free source validation existed (not current behaviour proof).
- [ ] Re-run static and traceability validators on the revised exact head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-41-phase-3 -->

- [ ] **41.1 — Distinct logical corpus/content revision/observation/run/attempt/profile identities**
  - Prerequisite: 40.1–2; immutable identity tests.
  - Acceptance: cargo test --locked -p searchright-living.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **41.2 — New, changed, withdrawn, corrected and delayed-index source observations**
  - Prerequisite: 41.1; three-wave delta fixtures.
  - Acceptance: cargo test --locked -p searchright-living.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **41.3 — Narrow safe invalidation for strategy, metadata, vocabulary and identity changes**
  - Prerequisite: 41.2; dependency-change metamorphic fixtures.
  - Acceptance: cargo test --locked -p searchright-living.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **41.4 — Full-replay equivalence versus incremental results**
  - Prerequisite: 41.3; property tests and explicit ordering semantics.
  - Acceptance: cargo test --locked -p searchright-living.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **41.5 — Qualify Salsa/DVC against simple existing Rust/CI baseline**
  - Prerequisite: 41.4; measured value/maintenance/rollback decision.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-41-phase-4 -->

- [ ] Run an actual Conductor review where available or state limits of repository-native review.
- [ ] Apply review findings and pass exact-head security, compiler and relevant consumer tests.
- [ ] Archive only after all applicable assertions, provenance and external gates pass; otherwise keep open.
