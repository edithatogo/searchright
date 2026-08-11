# Plan: 00 Foundation, Conductor and toolchain

Current status: **external_evidence_required**. Implementation state: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-00`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-00-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `Cargo.toml`
  - [x] Present source path: `rust-toolchain.toml`
  - [x] Present source path: `conductor/upstream.lock.json`
  - [x] Present source path: `conductor/index.md`
  - [x] Present source path: `conductor/product.md`
  - [x] Present source path: `conductor/workflow.md`
  - [x] Present source path: `Cargo.lock`
  - [x] Present source path: `docs/repository-standards-alignment.md`
  - [x] Present source path: `verification/receipts/track-00-local-verification.json`
  - [x] Present source path: `verification/receipts/track-00-conductor-review.json`
  - [x] Present source path: `scripts/install-conductor.sh`
  - [x] Present source path: `scripts/install-conductor.ps1`
  - [x] Assertion ledger: `conductor/tracks/00-foundation-conductor-toolchain/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-00-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-00-phase-3 -->

- [ ] Register Searchright in edithatogo/repository-standards and execute estate conformance at an approved upstream revision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-00-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
