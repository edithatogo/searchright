# Plan: 33 Operational observability, backup, restore and incident response

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.

GitHub issue key: `track-33`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-33-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-ops/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/ops.rs`
  - [x] Present source path: `contracts/examples/component-health.json`
  - [x] Present source path: `contracts/examples/telemetry-policy.json`
  - [x] Present source path: `contracts/examples/backup-manifest.json`
  - [x] Present source path: `contracts/examples/incident-record.json`
  - [x] Present source path: `docs/operations/reliability.md`
  - [x] Present source path: `docs/operations/backup-restore.md`
  - [x] Present source path: `docs/operations/incident-response.md`
  - [x] Present source path: `.github/workflows/resilience.yml`
  - [x] Present source path: `scripts/recovery_rehearsal.py`
  - [x] Present source path: `verification/recovery/rehearsal.json`
  - [x] Present source path: `docs/operations/recovery-reference-rehearsal.md`
  - [x] Present source path: `tests/test_operational_reliability_governance.py`
  - [x] Present source path: `verification/receipts/track-33-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/33-operational-reliability/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-33-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/check_workflow_hardening.py`
  - [x] `python scripts/recovery_rehearsal.py --self-test`
  - [x] `python3 -m unittest discover -s tests -p test_operational_reliability_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-33-phase-3 -->

- [x] Compile and execute operations policy tests and failure-injection scenarios.
- [ ] Run encrypted clean-room backup and restore rehearsals with audit-chain verification.
- [ ] Execute incident, cancellation and recovery exercises and approve deployment-specific service objectives.
- [ ] Run deployment-specific encrypted clean-room backup and restore drills against approved recovery objectives.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-33-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_operational_reliability_governance.py with 6 comprehensive unit tests covering component health contracts, liveness vs readiness distinction, degraded operation states, default-disabled telemetry with strict attribute prohibitions, encrypted backup manifests with mandatory restore testing, incident record structure with containment and postmortem mandates, recovery rehearsal execution and receipt validation, and resilience workflow hardening. Upgraded all 3 assertions (T33-A001..T33-A003) to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 panel receipts and preserved higher evidence gates open.
  - [x] Review fix: 2026-11-09 Track 30 reconciliation: restore unmet original requirements and stable issue keys; preserve source/test improvements and historical fixture reviews. Simulation is not external acceptance. See conductor/tracks/30-maturity-gap-closure/plan.md. Retained bounded compiler/fixture evidence: https://github.com/edithatogo/searchright/actions/runs/42566040226/job/128992651041 at f502104a81c43ed0117886ce8cb21f00fdeb7588; later commits require their own admission.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
