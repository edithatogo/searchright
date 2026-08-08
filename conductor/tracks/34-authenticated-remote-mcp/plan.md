# Plan: 34 Authenticated remote MCP, tenancy and data residency

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-34`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-34-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-access/src/lib.rs`
  - [x] `crates/searchright-contracts/src/access.rs`
  - [x] `contracts/examples/tenant-policy.json`
  - [x] `contracts/examples/access-request.json`
  - [x] `contracts/examples/access-decision.json`
  - [x] `docs/security/authenticated-remote-mcp.md`
  - [x] `contracts/openapi/searchright-http.openapi.yaml`
  - [x] `docs/security/threat-model.md`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-34-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_default_deny.py`
  - [x] `python scripts/check_rust_source_structure.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-34-phase-3 -->

- [ ] Implement and compile the authenticated Streamable HTTP transport behind a separate feature/deployment boundary.
- [ ] Run OAuth/OIDC, token rotation, tenant isolation, region, rate-limit, cancellation and abuse conformance tests.
- [ ] Complete independent security/privacy review before any hosted or multi-tenant claim.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-34-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
