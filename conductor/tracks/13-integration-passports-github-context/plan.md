# Plan: 13 Integration passports, GitHub issue hierarchy and context spine

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-13`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-13-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `integration/locks.json`
  - [x] Present source path: `integration/passports/index.json`
  - [x] Present source path: `contracts/json-schema/integration-passport.v1.schema.json`
  - [x] Present source path: `contracts/json-schema/github-issue-hierarchy.v1.schema.json`
  - [x] Present source path: `conductor/github/issue-hierarchy.json`
  - [x] Present source path: `conductor/github/README.md`
  - [x] Present source path: `scripts/check_integration_passports.py`
  - [x] Present source path: `scripts/render_github_issues.py`
  - [x] Present source path: `scripts/check_github_issue_hierarchy.py`
  - [x] Present source path: `scripts/sync_github_issues.py`
  - [x] Present source path: `CONTEXT.md`
  - [x] Present source path: `context/manifest.json`
  - [x] Present source path: `context/decision-ledger.json`
  - [x] Present source path: `context/claim-boundaries.json`
  - [x] Present source path: `scripts/check_context_integrity.py`
  - [x] Present source path: `crates/searchright-contracts/src/integration.rs`
  - [x] Present source path: `contracts/examples/integration-passport.json`
  - [x] Present source path: `contracts/examples/github-issue-hierarchy.json`
  - [x] Present source path: `integration/passports/citeweft-document-evidence.json`
  - [x] Present source path: `scripts/check_integration_drift.py`
  - [x] Present source path: `scripts/sync_context_lock.py`
  - [x] Present source path: `context/capability-matrix.json`
  - [x] Present source path: `context/hazard-log.json`
  - [x] Present source path: `context/evidence-ledger.json`
  - [x] Present source path: `.github/workflows/issue-sync.yml`
  - [x] Present source path: `.github/workflows/integration-drift.yml`
  - [x] Present source path: `docs/integration-architecture.md`
  - [x] Present source path: `contracts/json-schema/consumer-contract-suite.v1.schema.json`
  - [x] Present source path: `contracts/examples/consumer-contract-suite.json`
  - [x] Present source path: `integration/consumer-contract-suite.json`
  - [x] Present source path: `scripts/check_consumer_contracts.py`
  - [x] Present source path: `docs/adrs/0016-federated-consumer-contracts.md`
  - [x] Present source path: `integration/github/portfolio-project.json`
  - [x] Present source path: `integration/github/portfolio-readme.md`
  - [x] Present source path: `scripts/check_portfolio_project.py`
  - [x] Present source path: `scripts/plan_github_portfolio.py`
  - [x] Present source path: `scripts/check_licence_firewall.py`
  - [x] Assertion ledger: `conductor/tracks/13-integration-passports-github-context/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-13-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_integration_passports.py`
  - [x] `python scripts/check_consumer_contracts.py`
  - [x] `python scripts/check_github_issue_hierarchy.py`
  - [x] `python scripts/check_context_integrity.py`
  - [x] `python scripts/check_integration_drift.py`
  - [x] `python scripts/render_github_issues.py --check`
  - [x] `python scripts/sync_context_lock.py --check`
  - [x] `python scripts/check_portfolio_project.py`
  - [x] `python scripts/check_licence_firewall.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-13-phase-3 -->

- [ ] Re-run the read-only GitHub convergence audit after PR #569 merges to bind observed control-plane parity to the exact merged main revision.
- [ ] Execute consumer-driven contract tests in each pinned downstream repository.
- [ ] Record scheduled integration-drift receipts against the live repository estate.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-13-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
