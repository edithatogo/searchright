# Plan: 24 WASI components, HTTP MCP and scalable execution

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-24`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-24-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-plugin-sdk/src/lib.rs`
  - [x] Present source path: `crates/searchright-policy/src/lib.rs`
  - [x] Present source path: `contracts/wit/search-provider.wit`
  - [x] Present source path: `contracts/openapi/searchright-http.openapi.yaml`
  - [x] Present source path: `contracts/examples/provider-component.yaml`
  - [x] Present source path: `docs/adrs/0004-provider-plugin-sandbox.md`
  - [x] Assertion ledger: `conductor/tracks/24-wasi-http-scale/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-24-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-24-phase-3 -->

- [ ] Compile WASI components and run the component conformance suite.
- [ ] Implement and threat-model authenticated Streamable HTTP/OAuth transport.
- [ ] Run MCP version-compatibility, cancellation, task, cache and load tests.
- [ ] Establish component signing, revocation and distribution evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-24-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
