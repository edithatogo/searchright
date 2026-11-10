# Plan: 39 Conditional dialect semantics and translation fidelity

Current status: **scaffolded**. Implementation state: **scaffolded**. Current evidence ceiling: **source_verified**. The historical issue/phase/task identifiers in metadata remain stable; superseded screening/VOI algorithms are not implementation requirements.

## Phase 1: Source implementation

<!-- github-subissue-key: track-39-phase-1 -->

- [ ] Implement the current scope and bind each assertion to source symbols and evidence.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-39-phase-2 -->

- [x] Historical network-free source validation exists (not evidence of current functional completeness).
- [ ] Re-run the static source/evidence validators on the exact revised head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-39-phase-3 -->

- [ ] **39.1 — Supported Boolean, field, proximity, truncation and controlled-vocabulary model**
  - Prerequisite: Existing compiler fixtures; unsupported semantics explicit Unknown.
  - Acceptance: cargo test --locked -p evidence-search-core.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **39.2 — Semantic comparison classification under declared assumptions**
  - Prerequisite: 39.1; equivalence/broadening/narrowing/incomparable/unknown fixtures.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **39.3 — Property and metamorphic differential tests with verified witnesses**
  - Prerequisite: 39.2; independently evaluated counterexamples.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **39.4 — Measure OxiDD/SMT reuse against existing implementation, do not adopt without benefit**
  - Prerequisite: 39.3; memory, security, MSRV, licence, maintenance and parity report.
  - Acceptance: cargo test --locked -p searchright-validation.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **39.5 — Held-out real expert strategies and known-study retention benchmark**
  - Prerequisite: 39.1–4, lawful source access and independent assessors.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-39-phase-4 -->

- [ ] Run an actual Conductor review if available, or equivalent repository-native checks with limitation stated.
- [ ] Apply findings, rebuild, run security/source-integrity and relevant consumer/live tests on the exact head.
- [ ] Reconcile roadmap coverage, traceability and receipts and archive only if every applicable gate passes; otherwise retain active status.
