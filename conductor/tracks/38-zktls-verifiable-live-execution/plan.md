# Plan: 38 Verifiable live two-source execution and zkTLS provenance receipts

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-38`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-38-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `conductor/tracks/38-zktls-verifiable-live-execution/spec.md`
  - [x] Present source path: `crates/evidence-search-contracts/src/lib.rs`
  - [x] Present source path: `crates/evidence-search-core/src/lib.rs`
  - [x] Present source path: `crates/searchright-connectors/src/lib.rs`
  - [x] Present source path: `crates/searchright-provenance/src/lib.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `crates/searchright-cli/src/main.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Assertion ledger: `conductor/tracks/38-zktls-verifiable-live-execution/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-38-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/check_traceability.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-38-phase-3 -->

- [ ] Define TlsTranscriptCommitment and redacted header byte-range contracts in crates/evidence-search-contracts/src/lib.rs and crates/searchright-contracts/src/lib.rs with unit validation tests.
- [ ] Implement PaginationCheckpoint, deterministic HTTP 429 Retry-After backoff, and duplicate-emission prevention on resume in crates/evidence-search-core/src/lib.rs.
- [ ] Implement 4 KiB chunk SHA-256 Merkle root computation, secret-preserving byte redaction, and atomic checkpoint persistence in crates/searchright-connectors/src/lib.rs and crates/searchright-store/src/lib.rs.
- [ ] Wire explicit LiveExecutionAuthorization and two-source PubMed + Europe PMC execution through crates/searchright/src/engine.rs, crates/searchright-cli/src/main.rs, and crates/searchright-mcp/src/lib.rs.
- [ ] Implement offline .srpack zkTLS transcript Merkle verification in crates/searchright-provenance/src/lib.rs and record authorised redacted live receipts.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-38-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
