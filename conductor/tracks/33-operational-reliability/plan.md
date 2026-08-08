# Plan: 33 Operational observability, backup, restore and incident response

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-33`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-33-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `crates/searchright-ops/src/lib.rs`
  - [x] `crates/searchright-contracts/src/ops.rs`
  - [x] `contracts/examples/component-health.json`
  - [x] `contracts/examples/telemetry-policy.json`
  - [x] `contracts/examples/backup-manifest.json`
  - [x] `contracts/examples/incident-record.json`
  - [x] `docs/operations/reliability.md`
  - [x] `docs/operations/backup-restore.md`
  - [x] `docs/operations/incident-response.md`
  - [x] `.github/workflows/resilience.yml`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-33-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_rust_source_structure.py`
  - [x] `python scripts/check_workflow_hardening.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-33-phase-3 -->

- [ ] Compile and execute operations policy tests and failure-injection scenarios.
- [ ] Run encrypted clean-room backup and restore rehearsals with audit-chain verification.
- [ ] Execute incident, cancellation and recovery exercises and approve deployment-specific service objectives.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-33-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
