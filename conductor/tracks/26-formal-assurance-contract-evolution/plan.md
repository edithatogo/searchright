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
  - [x] Present source path: `.github/workflows/formal.yml`
  - [x] Present source path: `fuzz/Cargo.toml`
  - [x] Present source path: `verification/harness-matrix.json`
  - [x] Assertion ledger: `conductor/tracks/26-formal-assurance-contract-evolution/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-26-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/check_workflow_hardening.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-26-phase-3 -->

- [ ] Compile and execute Kani proofs, Loom permutations, Miri and cargo-careful checks.
- [ ] Compile and execute model-check traces and forbidden-transition tests.
- [ ] Add compatibility fixtures for each public contract version.
- [ ] Run semver and cross-version MCP/CLI contract tests.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-26-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
