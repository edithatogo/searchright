<!-- searchright-issue-key: track-38-phase-3-task-04 -->
# Track 38 / Phase 3 / Task 04

Parent phase key: `track-38-phase-3`
Conductor plan: `conductor/tracks/38-zktls-verifiable-live-execution/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] **38.4 — CLI/MCP parity with denied-by-default consequential operations**
  - Prerequisite: 38.3; same query, source, identity and approval semantics.
  - Acceptance: cargo test --locked -p searchright-cli -p searchright-mcp.
  - Verification: `undefined`.
  - Preserve the tested input/output revisions, outcomes and blockers; do not promote source presence to live or external evidence.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
