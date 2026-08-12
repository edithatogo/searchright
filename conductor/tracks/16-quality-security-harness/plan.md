# Plan: 16 Maximal quality, context and security harness

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-16`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-16-phase-1 -->

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
  - [x] Present source path: `verification/gate-catalog.json`
  - [x] Present source path: `scripts/check_gate_catalog.py`
  - [x] Present source path: `verification/architecture-policy.json`
  - [x] Present source path: `scripts/check_architecture_fitness.py`
  - [x] Present source path: `verification/evidence-debt.json`
  - [x] Present source path: `scripts/generate_evidence_debt.py`
  - [x] Present source path: `policy/redaction-profile.json`
  - [x] Present source path: `scripts/redaction.py`
  - [x] Present source path: `scripts/check_redaction_policy.py`
  - [x] Present source path: `docs/quality/gate-catalog-and-evidence-ceilings.md`
  - [x] Present source path: `docs/quality/evidence-debt.md`
  - [x] Present source path: `docs/quality/architecture-fitness.md`
  - [x] Present source path: `docs/security/redaction-and-data-minimisation.md`
  - [x] Present source path: `verification/coverage-policy.json`
  - [x] Present source path: `scripts/check_coverage_policy.py`
  - [x] Present source path: `tests/test_coverage_policy.py`
  - [x] Present source path: `docs/security/cargo-vet-governance.md`
  - [x] Present source path: `scripts/check_cargo_vet_governance.py`
  - [x] Present source path: `supply-chain/exemption-proposals.json`
  - [x] Present source path: `verification/receipts/track-16-hosted-pr-560c78c.json`
  - [x] Assertion ledger: `conductor/tracks/16-quality-security-harness/traceability.json`

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
  - [x] `python scripts/check_traceability.py`
  - [x] `python scripts/check_public_package_policy.py`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/check_gate_catalog.py --check`
  - [x] `python scripts/generate_evidence_debt.py --check`
  - [x] `python scripts/check_architecture_fitness.py`
  - [x] `python scripts/check_redaction_policy.py --self-test`
  - [x] `python scripts/check_coverage_policy.py`
  - [x] `python scripts/check_cargo_vet_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-16-phase-3 -->

- [ ] Push the local admission-policy commits and obtain green required checks on the exact new PR head; results at 560c78c do not prove later commits.
- [ ] Add meaningful tests and ratchet measured line coverage from the 61.02 percent baseline to greater than 90 percent for maturity.
- [ ] Replace temporary exact-version cargo-vet exemptions with qualifying peer or Searchright audits before their 2026-11-10 review deadline.
- [ ] Run mutation and push-context OpenSSF Scorecard on the exact candidate, then resolve every remaining critical/high finding or governed exclusion.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-16-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
