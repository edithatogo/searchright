<!-- searchright-issue-key: track-40-phase-3-task-01 -->
# Track 40 / Phase 3 / Task 01

Parent phase key: `track-40-phase-3`
Conductor plan: `conductor/tracks/40-martingale-conformal-stopping/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] **40.1 — Canonical CSL-JSON ID/order/field-preservation and content-revision primitive**
  - Prerequisite: Existing serde_json/blake3, no new deps, adversarial unit tests.
  - Acceptance: cargo test --locked -p searchright-interchange csl_corpus.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
