# Plan: 06 Imports, deduplication and study linkage

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-06`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-06-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-dedup/src/lib.rs`
  - [x] Present source path: `crates/searchright-interchange/src/lib.rs`
  - [x] Present source path: `crates/searchright-study/src/lib.rs`
  - [x] Present source path: `contracts/examples/interchange-receipt.yaml`
  - [x] Present source path: `contracts/examples/study-graph.yaml`
  - [x] Present source path: `docs/adrs/0005-record-report-study-separation.md`
  - [x] Assertion ledger: `conductor/tracks/06-imports-dedup-linkage/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-06-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-06-phase-3 -->

- [ ] Compile and run golden import/export round-trip tests across every supported format.
- [ ] Evaluate linkage and deduplication against independently adjudicated corpora.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-06-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
