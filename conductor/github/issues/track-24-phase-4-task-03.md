<!-- searchright-issue-key: track-24-phase-4-task-03 -->
# Track 24 / Phase 4 / Task 03

Parent phase key: `track-24-phase-4`
Conductor plan: `conductor/tracks/24-wasi-http-scale/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all six Track 24 assertions (T24-A001..T24-A006) to symbol_and_gate_mapped with concrete implementation symbols across searchright-plugin-sdk, searchright-policy, searchright-contracts, searchright-store, searchright-mcp, OpenAPI specs, and test_wasi_http_scale_governance.py; create unit test suite in tests/test_wasi_http_scale_governance.py validating OpenAPI 3.1.0 specifications, WIT search-provider contracts, provider component example schema conformance, release signature/trust policy schemas, and Track 24 traceability invariants; harden PolicyDecision enum, validate_request helper, and unit tests in crates/searchright-policy without forbidden fragments; record conductor review receipt and preserve open WASI compilation, Wasmtime resource controls, Arrow/Parquet/DuckDB engines, and Streamable HTTP/OAuth blockers without false promotion.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
