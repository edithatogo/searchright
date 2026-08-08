<!-- searchright-issue-key: track-16-phase-1-task-01 -->
# Track 16 / Phase 1 / Task 01

Parent phase key: `track-16-phase-1`
Conductor plan: `conductor/tracks/16-quality-security-harness/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Implement and document the track's source deliverables.
  - [x] `.github/workflows/ci.yml`
  - [x] `.github/workflows/coverage.yml`
  - [x] `.github/workflows/nightly.yml`
  - [x] `.github/workflows/security.yml`
  - [x] `scripts/run_static_harness.py`
  - [x] `scripts/check_rust_dependency_graph.py`
  - [x] `verification/sbom/source-components.cdx.json`
  - [x] `docs/security/threat-model.md`
  - [x] `codecov.yml`
  - [x] `.github/workflows/formal.yml`
  - [x] `.github/workflows/fuzz.yml`
  - [x] `.github/workflows/clean-room.yml`
  - [x] `scripts/check_default_deny.py`
  - [x] `scripts/check_workflow_hardening.py`
  - [x] `scripts/mcp_smoke.py`
  - [x] `verification/harness-matrix.json`
  - [x] `docs/quality/maximal-harness.md`
  - [x] `fuzz/Cargo.toml`
  - [x] `fuzz/fuzz_targets/query_contract.rs`
  - [x] `fuzz/fuzz_targets/document_evidence.rs`
  - [x] `fuzz/fuzz_targets/audit_event.rs`
  - [x] `scripts/generate_source_hash_manifest.py`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
