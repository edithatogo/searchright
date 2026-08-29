# Plan: 08 PRISMA, PRESS and reporting

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-08`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-08-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-prisma/src/lib.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/reporting_fixtures.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/snapshots/`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `crates/searchright-cli/src/main.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `contracts/examples/prisma-flow.json`
  - [x] Present source path: `contracts/examples/search-validation.yaml`
  - [x] Present source path: `contracts/examples/standard-assessment.yaml`
  - [x] Present source path: `docs/standards-and-provenance.md`
  - [x] Present source path: `docs/prisma-jl-interop.md`
  - [x] Present source path: `integration/fixtures/prisma-jl/flow-parity-v1.json`
  - [x] Present source path: `verification/fixtures/track-08/press-workflow-complete.yaml`
  - [x] Assertion ledger: `conductor/tracks/08-prisma-press-reporting/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-08-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-08-phase-3 -->

- [ ] Obtain an accountable independent PRESS review of representative generated strategies and appendices.
- [ ] Obtain a hosted compiler-verified receipt for the exact committed revision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-08-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix `c234c41f940b37e153e9bf151458059e63c51b47`: Fail closed on PRESS decisions, bind appendix execution evidence to the compiler version and query fingerprint, and escape active Typst inline markup controls.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
