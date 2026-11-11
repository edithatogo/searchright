# Plan: 40 Canonical CSL-JSON and downstream consumer interoperability

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

## Phase 1: Source implementation

<!-- github-subissue-key: track-40-phase-1 -->

- [ ] Implement the current scope and bind each assertion to source symbols and evidence.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-40-phase-2 -->

- [x] Historical network-free source validation exists (not evidence of current functional completeness).
- [ ] Re-run the static source/evidence validators on the exact revised head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-40-phase-3 -->

- [ ] **40.1 — Canonical CSL-JSON ID/order/field-preservation and content-revision primitive**
  - Prerequisite: Existing serde_json/blake3, no new deps, adversarial unit tests.
  - Acceptance: cargo test --locked -p searchright-interchange csl_corpus.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **40.2 — Versioned rights, lineage, source-record, citation-edge, affiliation and identity sidecars**
  - Prerequisite: 40.1; upstream CSL schema and cross-artifact invariants.
  - Acceptance: python scripts/validate_repository.py.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **40.3 — Pinned ASReview CSV/RIS import and optional extension with corpus-to-label crosswalk**
  - Prerequisite: 40.2; real ASReview consumer tests, no Searchright screening model.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **40.4 — Pinned pybibx adapter with truthful supported data source/neutral export**
  - Prerequisite: 40.2; real pybibx import and field-loss report.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **40.5 — bibliometrix and VOSviewer imports preserving available citation/affiliation edges**
  - Prerequisite: 40.2; independent expected-count and missingness fixtures.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **40.6 — Negative round-trip tests across consumers and immutable revision-bound return events**
  - Prerequisite: 40.3–40.5; no canonical overwrite or silent loss.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-40-phase-4 -->

- [ ] Run an actual Conductor review if available, or equivalent repository-native checks with limitation stated.
- [ ] Apply findings, rebuild, run security/source-integrity and relevant consumer/live tests on the exact head.
- [ ] Reconcile roadmap coverage, traceability and receipts and archive only if every applicable gate passes; otherwise retain active status.
