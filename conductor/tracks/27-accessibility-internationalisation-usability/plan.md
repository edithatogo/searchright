# Plan: 27 Accessibility, internationalisation and usability

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-27`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-27-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-diagnostics/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/diagnostic.rs`
  - [x] Present source path: `contracts/examples/diagnostic.yaml`
  - [x] Present source path: `docs/adrs/0010-accessible-diagnostics-and-institutional-governance.md`
  - [x] Present source path: `docs/evaluation/external-methodological-evaluation.md`
  - [x] Assertion ledger: `conductor/tracks/27-accessibility-internationalisation-usability/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-27-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-27-phase-3 -->

- [ ] Compile and run plain/JSON/JSONL snapshots and no-colour tests.
- [ ] Create message catalogues and execute locale-fallback tests.
- [ ] Complete keyboard, screen-reader and information-specialist usability evaluation.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-27-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
