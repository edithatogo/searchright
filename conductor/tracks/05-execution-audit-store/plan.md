# Plan: 05 Execution, audit and local storage

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-05`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-05-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/evidence-search-core/src/audit.rs`
  - [x] `crates/searchright-store/src/lib.rs`
  - [x] `contracts/examples/search-run.yaml`
  - [x] `contracts/examples/audit-event.json`
  - [x] `docs/adrs/0002-contract-first-and-event-ledger.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-05-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-05-phase-3 -->

- [ ] Compile and run audit tamper, replay, lock and crash-recovery tests on supported platforms.
- [ ] Complete external review of retention and deletion semantics.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-05-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
