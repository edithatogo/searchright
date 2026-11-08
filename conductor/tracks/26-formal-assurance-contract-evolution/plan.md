# Plan: 26 Formal assurance and contract evolution

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-26`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-26-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-assurance/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/assurance.rs`
  - [x] Present source path: `contracts/examples/workflow-trace.yaml`
  - [x] Present source path: `docs/adrs/0011-assurance-and-evidence-ladder.md`
  - [x] Present source path: `crates/searchright-assurance/tests/loom_authority.rs`
  - [x] Present source path: `crates/searchright-assurance/tests/workflow_model.rs`
  - [x] Present source path: `.github/workflows/formal.yml`
  - [x] Present source path: `fuzz/Cargo.toml`
  - [x] Present source path: `verification/harness-matrix.json`
  - [x] Present source path: `contracts/migrations/registry.json`
  - [x] Present source path: `contracts/migrations/github-issue-hierarchy-v1-to-v2.json`
  - [x] Present source path: `scripts/check_schema_migrations.py`
  - [x] Present source path: `docs/contracts/schema-evolution.md`
  - [x] Present source path: `tests/test_formal_assurance_contract_evolution.py`
  - [x] Present source path: `tests/test_mcp_authority_state_machine.py`
  - [x] Present source path: `verification/receipts/track-26-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/26-formal-assurance-contract-evolution/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-26-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/check_workflow_hardening.py`
  - [x] `python scripts/check_schema_migrations.py`
  - [x] `python3 -B -m unittest discover -s tests -p test_formal_assurance_contract_evolution.py`
  - [x] `python3 -B -m unittest discover -s tests -p test_mcp_authority_state_machine.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-26-phase-3 -->

- [ ] Compile and execute Kani proofs, Loom permutations, Miri and cargo-careful checks.
- [ ] Compile and execute model-check traces and forbidden-transition tests.
- [ ] Add compatibility fixtures for each public contract version.
- [ ] Run semver and cross-version MCP/CLI contract tests.
- [ ] Compile and execute representative persisted-data migrations, backward readers and rollback projections.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-26-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 26 assertions (T26-A001..T26-A005) to symbol_and_gate_mapped with concrete implementation symbols across searchright-assurance, check_schema_migrations.py, workflow trace schemas, and tests/test_formal_assurance_contract_evolution.py and tests/test_mcp_authority_state_machine.py; create unit test suite in tests/test_formal_assurance_contract_evolution.py validating workflow-trace schemas, fail-closed schema migration registry policies, non-destructive forward migrations with rollback support, and check_schema_migrations.py execution; integrate Hypothesis stateful property testing over the searchright-mcp stdio binary verifying screening authority boundaries; record conductor review receipt and preserve open Kani/Loom/Miri proofs, model-check traces, contract compatibility fixtures, semver cross-version contract tests, and persisted-data migration drill blockers without false promotion.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
