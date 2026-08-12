# Plan: 31 GitHub remote, nested issues and Project v2 control plane

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-31`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-31-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `conductor/github/project.json`
  - [x] Present source path: `conductor/github/project-readme.md`
  - [x] Present source path: `conductor/github/repository-settings.json`
  - [x] Present source path: `conductor/github/issue-hierarchy.json`
  - [x] Present source path: `scripts/bootstrap_github.py`
  - [x] Present source path: `scripts/sync_github_issues.py`
  - [x] Present source path: `scripts/sync_github_project.py`
  - [x] Present source path: `scripts/check_github_project.py`
  - [x] Present source path: `.github/workflows/github-control-plane.yml`
  - [x] Present source path: `docs/github-operating-system.md`
  - [x] Present source path: `contracts/json-schema/github-issue-hierarchy.v2.schema.json`
  - [x] Present source path: `contracts/json-schema/github-project.v1.schema.json`
  - [x] Assertion ledger: `conductor/tracks/31-github-control-plane/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-31-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/render_github_issues.py --check`
  - [x] `python scripts/check_github_issue_hierarchy.py`
  - [x] `python scripts/check_github_project.py`
  - [x] `python scripts/check_workflow_hardening.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-31-phase-3 -->

- [ ] Run the full bootstrap against the exact merged main revision and preserve its observed receipt.
- [ ] Re-run the credential-backed issue, Project and read-only control-plane audit against that exact merged revision.
- [ ] Complete the independent Conductor review and semantic in-place archival after exact-main evidence passes.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-31-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
