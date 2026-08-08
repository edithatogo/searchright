# Plan: 04 Open provider connectors MVP

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-04`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-04-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-connectors/src/lib.rs`
  - [x] `contracts/examples/provider-manifest.yaml`
  - [x] `contracts/examples/provider-page.yaml`
  - [x] `contracts/examples/source-receipt.yaml`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-04-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-04-phase-3 -->

- [ ] Compile and run connector fixtures.
- [ ] Run authorised, redacted live smokes for each advertised provider.
- [ ] Verify upstream terms, rate limits and response changes at release time.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-04-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
