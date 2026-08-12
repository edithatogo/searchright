<!-- searchright-issue-key: track-16-phase-2-task-01 -->
# Track 16 / Phase 2 / Task 01

Parent phase key: `track-16-phase-2`
Conductor plan: `conductor/tracks/16-quality-security-harness/plan.md`
Canonical task state: **source task complete**.

## Canonical task

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

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
