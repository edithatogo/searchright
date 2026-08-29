# Plan: 10 MCP stdio server MVP

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.

GitHub issue key: `track-10`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-10-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-mcp/src/main.rs`
  - [x] Present source path: `crates/searchright-mcp/src/effect_policy.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `crates/searchright-mcp/tests/advanced_mcp.rs`
  - [x] Present source path: `crates/searchright-mcp/tests/live_client_conformance.rs`
  - [x] Present source path: `crates/searchright/src/authority.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `crates/searchright-store/src/lib.rs`
  - [x] Present source path: `contracts/json-schema/plan-review-result.v1.schema.json`
  - [x] Present source path: `contracts/json-schema/press-review-result.v1.schema.json`
  - [x] Present source path: `contracts/json-schema/press-review.v1.schema.json`
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

- [x] Complete the contracted planning, fixture-execution and screening authority paths through an opaque trusted-host verifier while default MCP and CLI adapters fail closed.
- [x] Extend bounded local resources and prompts to plan, run, queue, report and update-workflow contract metadata with no canonical authority claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-10-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `232003620e3a087f4025ca7a8e957f92cb169b51`: Close authority-boundary, immutable idempotency, exact source-preservation, structured-output, resource and prompt review findings while restoring architecture, source-structure and generated-state gates.
  - Review fix `8912bf8440d5b073a9aa7e1f673acbed5ce674d8`: Fix exact official-client receipt invocation through libtest and make interrupted receipt generation safely retryable without weakening the clean source-tree gate.
  - Review fix `945694cc1004d0dacf09f3bd60a42107f1d46f84`: Close the workspace-wide strict Clippy gate by keeping the screening decision enum import scoped to store tests.
  - Review fix `896e88ac39456be6d535eefc61b725c60c0a16b3`: Make advanced MCP pagination and authority-spoof assertions panic-safe under the workspace Clippy policy.
  - Review fix `c72fc1e76d2100116e0ab2f4e260889c5c5c04ba`: Remove the unused searchright facade UUID declaration reported by the hosted cargo-machete gate and regenerate the source SBOM and hash manifest.
  - Review fix `c1414d5cb592c67a544f0c9390fa0e9e7fd41411`: Refresh the locked searchright package dependency list after removing the redundant UUID declaration so locked builds remain reproducible.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
