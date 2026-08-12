# Plan: 34 Authenticated remote MCP, tenancy and data residency

Current status: **integration_prepared**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-34`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-34-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-access/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/access.rs`
  - [x] Present source path: `contracts/examples/tenant-policy.json`
  - [x] Present source path: `contracts/examples/access-request.json`
  - [x] Present source path: `contracts/examples/access-decision.json`
  - [x] Present source path: `docs/security/authenticated-remote-mcp.md`
  - [x] Present source path: `contracts/openapi/searchright-http.openapi.yaml`
  - [x] Present source path: `docs/security/threat-model.md`
  - [x] Present source path: `crates/searchright-mcp/src/remote.rs`
  - [x] Present source path: `crates/searchright-mcp/src/bin/searchright-mcp-http.rs`
  - [x] Present source path: `scripts/remote_mcp_conformance.py`
  - [x] Assertion ledger: `conductor/tracks/34-authenticated-remote-mcp/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-34-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_default_deny.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/remote_mcp_conformance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-34-phase-3 -->

- [ ] Run live IdP and trusted TLS-edge authentication, revocation, rotation and abuse conformance.
- [ ] Run multi-replica tenant/replay/rate/cancellation, deployment-residency and rollback conformance.
- [ ] Complete independent security/privacy review before any hosted or multi-tenant claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-34-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
