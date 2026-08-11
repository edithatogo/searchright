# Plan: 00 Foundation, Conductor and toolchain

Current status: **external_evidence_required**. Implementation state: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-00`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-00-phase-1 -->

- [ ] Complete every acceptance assertion; the local implementation is commit-bound, but external evidence remains open.
  - [x] Present source path: `Cargo.toml`
  - [x] Present source path: `rust-toolchain.toml`
  - [x] Present source path: `conductor/upstream.lock.json`
  - [x] Present source path: `conductor/index.md`
  - [x] Present source path: `conductor/product.md`
  - [x] Present source path: `conductor/workflow.md`
  - [x] Present source path: `Cargo.lock`
  - [x] Present source path: `docs/repository-standards-alignment.md`
  - [x] Present source path: `verification/receipts/track-00-local-verification.json`
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

- [x] Install and execute Rust 1.97.1 in a compatible environment.
  - [x] Exact GNU workspace check passed against implementation commit `3f62b5ebc3efe3e619c05dfacdabc01b159f1aec` with the committed `Cargo.lock`.
  - [x] All 55 workspace tests passed against that exact implementation commit.
  - [ ] MSVC validation remains invalid because Git's POSIX `link.exe` shadows the intended linker; this is recorded as an environment limitation, not a source failure.
- [x] Install Conductor in an available Gemini, Antigravity or Claude host and record the host receipt.
  - [x] Gemini CLI reports Conductor 0.4.1 enabled for both user and workspace scopes.
- [ ] Register Searchright in `edithatogo/repository-standards` and run estate conformance. *(external blocker: no entry exists at the pinned revision or current main, and the companion packet denies remote mutation without separate authority)*

## Phase 4: Review and closeout

<!-- github-subissue-key: track-00-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and apply receipt-integrity, cache-authority, endpoint-redaction and status-reconciliation fixes.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
