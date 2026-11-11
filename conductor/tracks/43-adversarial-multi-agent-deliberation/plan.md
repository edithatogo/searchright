# Plan: 43 Search strategy auditing and SEARCH-REPLAY evaluation

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

## Phase 1: Source implementation

<!-- github-subissue-key: track-43-phase-1 -->

- [ ] Implement the new scope and bind assertions to actual source symbols.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-43-phase-2 -->

- [x] Historical network-free source validation existed (not current behaviour proof).
- [ ] Re-run static and traceability validators on the revised exact head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-43-phase-3 -->

- [ ] **43.1 — Evidence-location-aware PRESS/PRISMA-S strategy audit**
  - Prerequisite: Existing standards; distinguish missing, ambiguous, contradictory, inaccessible.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **43.2 — Literal and plausible query reconstruction scenarios with frozen assumptions**
  - Prerequisite: 43.1; negative tests for auto-repair.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **43.3 — Record-set differences, known-study retention and interpretation uncertainty envelopes**
  - Prerequisite: 43.2/38.6; index drift separate and no false confidence intervals.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **43.4 — Optional agent audits with deterministic verification and human amendments**
  - Prerequisite: 43.1–3; prompt injection and false expert-independence tests.
  - Acceptance: cargo test --locked -p searchright-agent.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.
- [ ] **43.5 — SEARCH-REPLAY pilot/confirmation protocol, research datasets and publication contracts**
  - Prerequisite: 43.3–4; independently reviewable design and rights gates.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Record input/output revisions and test evidence before completion.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-43-phase-4 -->

- [ ] Run an actual Conductor review where available or state limits of repository-native review.
- [ ] Apply review findings and pass exact-head security, compiler and relevant consumer tests.
- [ ] Archive only after all applicable assertions, provenance and external gates pass; otherwise keep open.
