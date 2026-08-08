# Plan: 16 Maximal quality, context and security harness

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-16`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-16-phase-1 -->

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

## Phase 2: Source-level verification

<!-- github-subissue-key: track-16-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_dependency_graph.py`
  - [x] `python scripts/generate_source_sbom.py --check`
  - [x] `python scripts/run_static_harness.py`
  - [x] `python scripts/check_default_deny.py`
  - [x] `python scripts/check_workflow_hardening.py`
  - [x] `python scripts/generate_source_hash_manifest.py --check`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-16-phase-3 -->

- [ ] Generate and commit Cargo.lock in a networked Rust environment.
- [ ] Run compiler, Clippy, tests, coverage, mutation, fuzz, Kani, Loom, Miri, cargo-careful, CodeQL, Scorecard and full-history secret scans.
- [ ] Run clean-room offline builds, binary comparison, install smoke and MCP transcript.
- [ ] Resolve every critical/high finding and document justified exclusions.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-16-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
