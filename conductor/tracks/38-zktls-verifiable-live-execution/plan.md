# Plan: 38 Bounded live search, recovery and source receipts

Current status: **scaffolded**. Implementation state: **scaffolded**. Current evidence ceiling: **source_verified**. The historical issue/phase/task identifiers in metadata remain stable; superseded screening/VOI algorithms are not implementation requirements.

## Phase 1: Source implementation

<!-- github-subissue-key: track-38-phase-1 -->

- [ ] Implement the current scope and bind each assertion to source symbols and evidence.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-38-phase-2 -->

- [x] Historical network-free source validation exists (not evidence of current functional completeness).
- [ ] Re-run the static source/evidence validators on the exact revised head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-38-phase-3 -->

- [ ] **38.1 — Authorised provider contracts and explicit query/source/budget policy**
  - Prerequisite: Provider contract fixtures; fail-closed unauthorized requests.
  - Acceptance: cargo test --locked -p evidence-search-core.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **38.2 — PubMed and Europe PMC cursor paging through the existing shared runtime**
  - Prerequisite: 38.1; retry/429, failed-page, duplicate-page fixtures.
  - Acceptance: cargo test --locked -p searchright-connectors.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **38.3 — Atomic checkpoints, resume and idempotent record emission**
  - Prerequisite: 38.2; crash/restart and cancellation fixture.
  - Acceptance: cargo test --locked -p evidence-search-core -p searchright-store.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **38.4 — CLI/MCP parity with denied-by-default consequential operations**
  - Prerequisite: 38.3; same query, source, identity and approval semantics.
  - Acceptance: cargo test --locked -p searchright-cli -p searchright-mcp.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **38.5 — Canonical CSL export plus source-record retention and loss receipts**
  - Prerequisite: 38.4 and 40.1; no dropped/native source records.
  - Acceptance: cargo test --locked -p searchright-interchange.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.
- [ ] **38.6 — Authorised two-source live run and independent retained-input replay**
  - Prerequisite: 38.5, rights and access; externally checked redacted receipts.
  - Acceptance: ./scripts/verify.sh.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-38-phase-4 -->

- [ ] Run an actual Conductor review if available, or equivalent repository-native checks with limitation stated.
- [ ] Apply findings, rebuild, run security/source-integrity and relevant consumer/live tests on the exact head.
- [ ] Reconcile roadmap coverage, traceability and receipts and archive only if every applicable gate passes; otherwise retain active status.
