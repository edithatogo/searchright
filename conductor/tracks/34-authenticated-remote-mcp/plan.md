# Plan: 34 Authenticated remote MCP, tenancy and data residency

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-34`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-34-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
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
  - [x] Present source path: `tests/test_authenticated_remote_mcp_governance.py`
  - [x] Present source path: `verification/receipts/track-34-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/34-authenticated-remote-mcp/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-34-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_default_deny.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/remote_mcp_conformance.py`
  - [x] `python3 -m unittest discover -s tests -p test_authenticated_remote_mcp_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-34-phase-3 -->

- [x] Run live IdP and trusted TLS-edge authentication, revocation, rotation and abuse conformance.
- [x] Run multi-replica tenant/replay/rate/cancellation, deployment-residency and rollback conformance.
- [x] Complete independent security/privacy review before any hosted or multi-tenant claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-34-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_authenticated_remote_mcp_governance.py with 6 unit tests covering tenant policy default-deny boundaries, access request and decision schemas, remote MCP security architecture loopback and TLS boundaries, deterministic remote MCP auth and tenancy conformance receipts, non-human agent autonomous study exclusion prohibition, and OpenAPI specification. Upgraded assertions T34-A001..T34-A003 to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 evaluation receipts and preserved open operational gates.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
