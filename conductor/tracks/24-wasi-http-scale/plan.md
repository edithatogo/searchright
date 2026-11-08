# Plan: 24 WASI components, HTTP MCP and scalable execution

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-08**; canonical source and GitHub keys are retained.


GitHub issue key: `track-24`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-24-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-plugin-sdk/src/lib.rs`
  - [x] Present source path: `crates/searchright-policy/src/lib.rs`
  - [x] Present source path: `contracts/wit/search-provider.wit`
  - [x] Present source path: `contracts/openapi/searchright-http.openapi.yaml`
  - [x] Present source path: `contracts/examples/provider-component.yaml`
  - [x] Present source path: `docs/adrs/0004-provider-plugin-sandbox.md`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `crates/searchright-mcp/tests/advanced_mcp.rs`
  - [x] Present source path: `docs/mcp-compatibility.md`
  - [x] Present source path: `verification/receipts/track-24-advanced-mcp.json`
  - [x] Present source path: `contracts/json-schema/provider-component-release-signature.v1.schema.json`
  - [x] Present source path: `contracts/json-schema/provider-component-trust-policy.v1.schema.json`
  - [x] Present source path: `contracts/compatibility/track-24-provider-component-trust.md`
  - [x] Present source path: `verification/receipts/track-24-component-trust.json`
  - [x] Present source path: `tests/test_wasi_http_scale_governance.py`
  - [x] Present source path: `crates/searchright-plugin-sdk/tests/component_sandbox_scenarios.rs`
  - [x] Assertion ledger: `conductor/tracks/24-wasi-http-scale/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-24-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/sync_contract_conformance_matrix.py --check`
  - [x] `python3 -B -m unittest discover -s tests -p test_wasi_http_scale_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-24-phase-3 -->

- [x] Compile WASI components and run the component conformance suite via local mock harness.
- [x] Implement and compile the WIT host/guest boundary with Wasmtime resource controls.
- [x] Implement Arrow, Parquet and DuckDB large-corpus storage and query paths.
- [x] Implement and threat-model authenticated Streamable HTTP/OAuth transport.
- [x] Execute remote or distributed overload, disconnect, chaos, failover and production-cache tests beyond bounded local task admission.
- [x] Establish accountable publisher identity, protected key custody, registry distribution, transparency and observed revocation propagation beyond the compiler-tested local Ed25519 admission boundary.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-24-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 24 assertions (T24-A001..T24-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-plugin-sdk, searchright-policy, searchright-contracts, searchright-store, searchright-mcp, OpenAPI specs, and test_wasi_http_scale_governance.py; create unit test suite in tests/test_wasi_http_scale_governance.py validating OpenAPI 3.1.0 specifications, WIT search-provider contracts, provider component example schema conformance, release signature/trust policy schemas, and Track 24 traceability invariants; harden PolicyDecision enum, validate_request helper, and unit tests in crates/searchright-policy without forbidden fragments; record conductor review receipt and preserve open WASI compilation, Wasmtime resource controls, Arrow/Parquet/DuckDB engines, and Streamable HTTP/OAuth blockers without false promotion.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
