# Plan: 20 Grey literature, registers and supplementary discovery

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.
Lifecycle: **archived** on **2026-08-29**; canonical source and GitHub keys are retained.


GitHub issue key: `track-20`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-20-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-discovery/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/discovery.rs`
  - [x] Present source path: `contracts/examples/discovery-run.yaml`
  - [x] Present source path: `contracts/fixtures/discovery-source-methods.json`
  - [x] Present source path: `contracts/fixtures/opencitations-forward.json`
  - [x] Present source path: `docs/provider-model.md`
  - [x] Present source path: `docs/supplementary-discovery.md`
  - [x] Present source path: `scripts/check_broader_discovery.py`
  - [x] Assertion ledger: `conductor/tracks/20-broader-discovery/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-20-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_broader_discovery.py`
  - [x] `cargo test -p searchright-contracts -p searchright-discovery`
  - [x] `cargo clippy -p searchright-contracts -p searchright-discovery --all-targets -- -D warnings`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-20-phase-3 -->

- [x] Compile and run citation-chaining and supplementary-discovery scenarios.
- [x] Add authorised source-specific live adapters or documented manual methods.
- [x] Run a structured simulated information-specialist and grey-literature methodology panel while retaining its non-human limitation.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-20-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Replace delimiter-ambiguous non-cryptographic citation edge IDs with length-framed BLAKE3 identifiers and reject padded, control-bearing or oversized fixture identifiers (`d0e73ea`).
  - [x] Review fix: Reconcile the final review receipt, metadata dates, closeout fields and semantic archive lifecycle while retaining canonical paths and stable GitHub keys.
  - [x] Review fix: Resolve candidate evidence by bounded-walk distances, cap cumulative traversal work and output memberships, and verify order/depth semantics against an exhaustive small-graph oracle.
  - [x] Review fix: Enforce seed, identifier, aggregate-byte and custom-method limits; reject control-bearing citing tokens.
  - [x] Review fix: Screen manual log identifiers, candidate identifiers and coverage rationales, and map assertions to exact runtime tests.
  - [x] Review fix: Retain historical Track 20 closeout task IDs explicitly without growing other tracks when review fixes are added.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.

- [x] Replace delimiter-ambiguous non-cryptographic citation edge IDs with length-framed BLAKE3 identifiers and reject padded, control-bearing or oversized fixture identifiers (`d0e73ea`).
- [x] Reconcile the final review receipt, metadata dates, closeout fields and semantic archive lifecycle while retaining canonical paths and stable GitHub keys.
