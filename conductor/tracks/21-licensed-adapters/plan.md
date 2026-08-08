# Plan: 21 Licensed BYO-access adapters

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-21`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-21-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-licensed/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/licensed.rs`
  - [x] Present source path: `contracts/licensed/index.json`
  - [x] Present source path: `contracts/licensed/embase.yaml`
  - [x] Present source path: `contracts/licensed/scopus.yaml`
  - [x] Present source path: `contracts/licensed/web-of-science.yaml`
  - [x] Present source path: `contracts/examples/licensed-adapter.yaml`
  - [x] Assertion ledger: `conductor/tracks/21-licensed-adapters/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-21-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-21-phase-3 -->

- [ ] Obtain authorised user credentials and confirm current vendor terms.
- [ ] Implement/test vendor-specific live transports without storing credentials.
- [ ] Record redacted user-run smoke receipts; no bundled access is permitted.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-21-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
