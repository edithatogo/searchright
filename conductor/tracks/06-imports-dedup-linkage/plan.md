# Plan: 06 Imports, deduplication and study linkage

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-06`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-06-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-dedup/src/lib.rs`
  - [x] `crates/searchright-interchange/src/lib.rs`
  - [x] `crates/searchright-study/src/lib.rs`
  - [x] `contracts/examples/interchange-receipt.yaml`
  - [x] `contracts/examples/study-graph.yaml`
  - [x] `docs/adrs/0005-record-report-study-separation.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-06-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-06-phase-3 -->

- [ ] Compile and run golden round-trip, property and metamorphic tests.
- [ ] Evaluate linkage and deduplication against independently adjudicated corpora.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-06-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
