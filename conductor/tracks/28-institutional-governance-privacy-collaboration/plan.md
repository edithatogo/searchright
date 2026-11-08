# Plan: 28 Institutional governance, privacy and collaboration

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-28`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-28-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-governance/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/governance.rs`
  - [x] Present source path: `contracts/examples/institutional-policy.yaml`
  - [x] Present source path: `contracts/examples/data-handling-request.yaml`
  - [x] Present source path: `contracts/examples/data-handling-decision.yaml`
  - [x] Present source path: `docs/adrs/0010-accessible-diagnostics-and-institutional-governance.md`
  - [x] Present source path: `docs/security/threat-model.md`
  - [x] Present source path: `policy/redaction-profile.json`
  - [x] Present source path: `scripts/redaction.py`
  - [x] Present source path: `scripts/check_redaction_policy.py`
  - [x] Present source path: `docs/security/redaction-and-data-minimisation.md`
  - [x] Present source path: `tests/test_institutional_governance_privacy_collaboration.py`
  - [x] Present source path: `crates/searchright-screening/src/lib.rs`
  - [x] Present source path: `crates/searchright-governance/src/approval.rs`
  - [x] Present source path: `verification/receipts/track-28-conductor-review-2026-11-04.json`
  - [x] Assertion ledger: `conductor/tracks/28-institutional-governance-privacy-collaboration/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-28-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_redaction_policy.py --self-test`
  - [x] `python3 -B -m unittest discover -s tests -p test_institutional_governance_privacy_collaboration.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-28-phase-3 -->

- [x] Compile and run comprehensive negative scenario tests with unapproved classification, deployment mode, region, model, and lifecycle transitions.
- [x] Verify deny-by-default bounded approval registry with temporal windows, single-use nonces, and cryptographic head digests.
- [x] Complete structured agent-panel evaluation simulating governance architect, privacy specialist, and adjudication auditor roles.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-28-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Upgrade all five Track 28 assertions (T28-A001..T28-A005) to symbol_and_gate_mapped with concrete symbols across searchright-governance, searchright-contracts, searchright-screening and scripts/redaction.py; add tests/test_institutional_governance_privacy_collaboration.py covering policy/request/decision consistency, deny-by-default blocker codes, human-approval and scoped lifecycle authority, receipt minimisation, adjudication independence and threat-model gates; fix the clippy assert_is_empty lint in the searchright-governance unit tests; record the conductor review receipt and preserve the open compiled negative-scenario, institutional review and cross-institution pilot gates without false promotion.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
