<!-- searchright-issue-key: track-16-phase-1-task-01 -->
# Track 16 / Phase 1 / Task 01

Parent phase key: `track-16-phase-1`
Conductor plan: `conductor/tracks/16-quality-security-harness/plan.md`
Canonical task state: **open evidence or implementation task**.

## Canonical task

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `.github/workflows/ci.yml`
  - [x] Present source path: `.github/workflows/coverage.yml`
  - [x] Present source path: `.github/workflows/nightly.yml`
  - [x] Present source path: `.github/workflows/security.yml`
  - [x] Present source path: `scripts/run_static_harness.py`
  - [x] Present source path: `scripts/check_rust_dependency_graph.py`
  - [x] Present source path: `verification/sbom/source-components.cdx.json`
  - [x] Present source path: `docs/security/threat-model.md`
  - [x] Present source path: `codecov.yml`
  - [x] Present source path: `.github/workflows/formal.yml`
  - [x] Present source path: `.github/workflows/fuzz.yml`
  - [x] Present source path: `.github/workflows/clean-room.yml`
  - [x] Present source path: `scripts/check_default_deny.py`
  - [x] Present source path: `scripts/check_workflow_hardening.py`
  - [x] Present source path: `scripts/mcp_smoke.py`
  - [x] Present source path: `verification/harness-matrix.json`
  - [x] Present source path: `docs/quality/maximal-harness.md`
  - [x] Present source path: `fuzz/Cargo.toml`
  - [x] Present source path: `fuzz/fuzz_targets/query_contract.rs`
  - [x] Present source path: `fuzz/fuzz_targets/document_evidence.rs`
  - [x] Present source path: `fuzz/fuzz_targets/audit_event.rs`
  - [x] Present source path: `scripts/generate_source_hash_manifest.py`
  - [x] Present source path: `supply-chain/config.toml`
  - [x] Present source path: `supply-chain/audits.toml`
  - [x] Present source path: `supply-chain/README.md`
  - [x] Present source path: `public-api/README.md`
  - [x] Present source path: `.github/workflows/public-api.yml`
  - [x] Present source path: `scripts/check_traceability.py`
  - [x] Present source path: `scripts/check_public_package_policy.py`
  - [x] Present source path: `scripts/sync_schema_surface.py`
  - [x] Present source path: `docs/vertical-slice-definition-of-done.md`
  - [x] Assertion ledger: `conductor/tracks/16-quality-security-harness/traceability.json`

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
