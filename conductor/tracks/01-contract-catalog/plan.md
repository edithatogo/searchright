# Plan: 01 Contract catalogue and code generation

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.
Lifecycle: **archived** on **2026-08-29**; canonical source and GitHub keys are retained.


GitHub issue key: `track-01`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-01-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/evidence-search-contracts/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/lib.rs`
  - [x] Present source path: `contracts/schema-catalog.json`
  - [x] Present source path: `contracts/compatibility/schema-surface-0.1.0-alpha.1.json`
  - [x] Present source path: `contracts/compatibility/contract-conformance-matrix.json`
  - [x] Present source path: `contracts/compatibility/rust-schema-parity.json`
  - [x] Present source path: `crates/evidence-search-contracts/src/schema.rs`
  - [x] Present source path: `crates/evidence-search-contracts/tests/schema_parity.rs`
  - [x] Present source path: `crates/evidence-search-contracts/examples/export_schemas.rs`
  - [x] Present source path: `scripts/sync_schema_surface.py`
  - [x] Present source path: `scripts/sync_contract_conformance_matrix.py`
  - [x] Present source path: `scripts/check_rust_schema_parity.py`
  - [x] Present source path: `scripts/generate_contract_bindings.py`
  - [x] Present source path: `scripts/check_contract_bindings.py`
  - [x] Present source path: `sdk/generated-contract-bindings.json`
  - [x] Present source path: `sdk/python/searchright_contracts/__init__.py`
  - [x] Present source path: `sdk/typescript/src/index.ts`
  - [x] Present source path: `docs/adrs/0017-canonical-schema-and-binding-ownership.md`
  - [x] Present source path: `verification/receipts/track-01-contract-generation-2026-08-29.json`
  - [x] Present source path: `verification/receipts/track-01-conductor-review-2026-08-29.json`
  - [x] Present source path: `verification/receipts/track-01-compiler-verification.json`
  - [x] Present source path: `verification/receipts/track-01-review-verification.json`
  - [x] Present source path: `release/public-packages.json`
  - [x] Present source path: `scripts/check_public_package_policy.py`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Assertion ledger: `conductor/tracks/01-contract-catalog/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-01-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/sync_contract_conformance_matrix.py --check`
  - [x] `python scripts/check_contract_bindings.py`
  - [x] `python scripts/check_rust_schema_parity.py --check`
  - [x] `python scripts/check_public_package_policy.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-01-phase-3 -->

- [x] Resolve generated/canonical constraint losses by recording every validation-shape difference and retaining canonical JSON Schema authority rather than claiming exact semantic parity.
- [x] Generate and compiler-import-check thin TypeScript and Python contract packages from the canonical catalogue.
- [x] Record package installation, clients, publication and downstream adoption as unclaimed Track 35 / SR-086 work outside Track 01 acceptance.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-01-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `c393134`: Resolve six integration-panel findings: preserve schema property/literal semantics, bind drift to schema digests, preserve generated TypeScript base fields and Python dictionaries, enforce pinned static typing and exercise compiled parity in CI. Verified locally with 55 static gates, 294 Rust tests and the Python test suite.
  - Review fix `67c0161`: Eliminate duplicate exported binding names, preserve cross-schema references and fail closed on unresolved references.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
