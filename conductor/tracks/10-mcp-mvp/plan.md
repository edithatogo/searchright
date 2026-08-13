# Plan: 10 MCP stdio server MVP

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **compiler_verified**.

GitHub issue key: `track-10`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-10-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-mcp/src/main.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `crates/searchright-mcp/tests/live_client_conformance.rs`
  - [x] Present source path: `contracts/mcp/tool-catalog.json`
  - [x] Present source path: `contracts/interface-catalog.json`
  - [x] Present source path: `docs/mcp-compatibility.md`
  - [x] Present source path: `scripts/mcp_smoke.py`
  - [x] Present source path: `scripts/record_mcp_live_client_conformance.py`
  - [x] Present source path: `server.json`
  - [x] Assertion ledger: `conductor/tracks/10-mcp-mvp/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-10-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/mcp_smoke.py target/debug/searchright-mcp`
  - [x] `python scripts/record_mcp_live_client_conformance.py --receipt-dir verification/receipts`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-10-phase-3 -->

- [ ] Complete the remaining contracted planning, execution and screening tool authority paths before claiming the full Track 10 surface complete.
- [ ] Extend the bounded local resources and prompts to the plan, run, queue, report and update-workflow coverage named by the Track 10 specification.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-10-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
