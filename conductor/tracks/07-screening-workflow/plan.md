# Plan: 07 Governed screening workflow

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-07`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-07-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-screening/src/lib.rs`
  - [x] Present source path: `crates/searchright-agent/src/lib.rs`
  - [x] Present source path: `contracts/examples/screening-decision.yaml`
  - [x] Present source path: `contracts/examples/screening-policy.yaml`
  - [x] Present source path: `docs/adrs/0003-agent-authority.md`
  - [x] Assertion ledger: `conductor/tracks/07-screening-workflow/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-07-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-07-phase-3 -->

- [ ] Compile and execute state-machine and authority-negative tests.
- [ ] Complete reviewer usability and inter-rater calibration evaluation.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-07-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
