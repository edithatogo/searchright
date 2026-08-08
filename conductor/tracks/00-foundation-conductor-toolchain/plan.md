# Plan: 00 Foundation, Conductor and toolchain

Current status: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-00`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-00-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `Cargo.toml`
  - [x] `rust-toolchain.toml`
  - [x] `conductor/upstream.lock.json`
  - [x] `conductor/product.md`
  - [x] `conductor/workflow.md`
  - [x] `scripts/install-conductor.sh`
  - [x] `scripts/install-conductor.ps1`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-00-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-00-phase-3 -->

- [ ] Install and execute Rust 1.97.1 in a compatible environment.
- [ ] Install Conductor in an available Gemini, Antigravity or Claude host and record the host receipt.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-00-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
