# Plan: 42 Search provenance, integrity notices and independent evidence packages

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**; historical issue/phase/task identifiers remain stable. The superseded algorithmic scope is not a current implementation requirement.

## Phase 1: Source implementation

<!-- github-subissue-key: track-42-phase-1 -->

- [ ] Implement the new scope and bind assertions to actual source symbols.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-42-phase-2 -->

- [x] Historical network-free source validation existed (not current behaviour proof).
- [ ] Re-run static and traceability validators on the revised exact head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-42-phase-3 -->

- [ ] **42.1 — Reference-field provenance and rights sidecars with source observation links**
  - Prerequisite: 40.2/38.5; fail unresolved identities and restricted disclosures.
  - Acceptance: cargo test --locked -p searchright-provenance.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **42.2 — Deterministic manifest of native query, source, CSL, sidecar, code and environment revisions**
  - Prerequisite: 42.1; cross-platform content and tamper tests.
  - Acceptance: cargo test --locked -p searchright-provenance.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **42.3 — Correction, withdrawal and retraction notices, without changing downstream conclusions**
  - Prerequisite: 42.2; notification propagation tests.
  - Acceptance: cargo test --locked -p searchright-living.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **42.4 — Independent preserved-input replay and missing/tampered asset rejection**
  - Prerequisite: 42.2–3; non-author verifier fixture.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **42.5 — Evaluate RO-Crate compatibility, optional TLSNotary and OpenLineage**
  - Prerequisite: 42.4; independently validated versions and no fabricated proof.
  - Acceptance: python scripts/validate_repository.py.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-42-phase-4 -->

- [ ] Run an actual Conductor review where available or state limits of repository-native review.
- [ ] Apply review findings and pass exact-head security, compiler and relevant consumer tests.
- [ ] Archive only after all applicable assertions, provenance and external gates pass; otherwise keep open.
