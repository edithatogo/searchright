# Plan: 28 Institutional governance, privacy and collaboration

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-28`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-28-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-governance/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/governance.rs`
  - [x] Present source path: `contracts/examples/institutional-policy.yaml`
  - [x] Present source path: `contracts/examples/data-handling-request.yaml`
  - [x] Present source path: `contracts/examples/data-handling-decision.yaml`
  - [x] Present source path: `docs/adrs/0010-accessible-diagnostics-and-institutional-governance.md`
  - [x] Present source path: `docs/security/threat-model.md`
  - [x] Assertion ledger: `conductor/tracks/28-institutional-governance-privacy-collaboration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-28-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-28-phase-3 -->

- [ ] Compile and run governance/authority negative scenarios.
- [ ] Obtain institutional privacy, records and security review for a deployment profile.
- [ ] Pilot multi-reviewer and cross-institution artefact exchange with minimised data.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-28-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
