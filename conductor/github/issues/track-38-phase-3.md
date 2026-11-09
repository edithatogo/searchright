<!-- searchright-issue-key: track-38-phase-3 -->
# Track 38 / Phase 3: Higher-evidence gates

Parent track key: `track-38`
Conductor plan: `conductor/tracks/38-zktls-verifiable-live-execution/plan.md`

## Task subissues

- [ ] T01: Define TlsTranscriptCommitment and redacted header byte-range contracts in crates/evidence-search-contracts/src/lib.rs and crates/searchright-contracts/src/lib.rs with unit validation tests. (`track-38-phase-3-task-01`)
- [ ] T02: Implement PaginationCheckpoint, deterministic HTTP 429 Retry-After backoff, and duplicate-emission prevention on resume in crates/evidence-search-core/src/lib.rs. (`track-38-phase-3-task-02`)
- [ ] T03: Implement 4 KiB chunk SHA-256 Merkle root computation, secret-preserving byte redaction, and atomic checkpoint persistence in crates/searchright-connectors/src/lib.rs and crates/searchright-store/src/lib.rs. (`track-38-phase-3-task-03`)
- [ ] T04: Wire explicit LiveExecutionAuthorization and two-source PubMed + Europe PMC execution through crates/searchright/src/engine.rs, crates/searchright-cli/src/main.rs, and crates/searchright-mcp/src/lib.rs. (`track-38-phase-3-task-04`)
- [ ] T05: Implement offline .srpack zkTLS transcript Merkle verification in crates/searchright-provenance/src/lib.rs and record authorised redacted live receipts. (`track-38-phase-3-task-05`)

## Evidence rule

Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.
