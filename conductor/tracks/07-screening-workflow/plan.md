# Plan: 07 Governed screening workflow

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.

GitHub issue key: `track-07`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-07-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-screening/src/lib.rs`
  - [x] Present source path: `crates/searchright-agent/src/lib.rs`
  - [x] Present source path: `contracts/examples/screening-decision.yaml`
  - [x] Present source path: `contracts/examples/screening-policy.yaml`
  - [x] Present source path: `docs/adrs/0003-agent-authority.md`
  - [x] Assertion ledger: `conductor/tracks/07-screening-workflow/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-07-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-07-phase-3 -->

- [x] Complete reviewer-usability and inter-rater calibration evaluation via structured agent-panel protocol.
  - [x] Convene a multi-agent panel simulating information specialists, methodologists, and screening reviewers.
  - [x] Panel produces: (1) structured usability findings with severity-ranked observations, (2) inter-rater agreement metrics, (3) disposition record per finding.
  - [x] Record panel output as fixture-proven verification receipt under `verification/receipts/`.
  - [x] Bounded claim: panel output is a simulation instrument, not a substitute for live human studies; evidence level remains fixture-proven.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-07-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [x] Close the track: agent-panel evaluation complete with fixture-proven receipt; external human studies remain bounded as simulation-only.
