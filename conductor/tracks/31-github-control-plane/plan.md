# Plan: 31 GitHub remote, nested issues and Project v2 control plane

Current status: **integration_prepared**. Evidence level: **source_verified**.

GitHub issue key: `track-31`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-31-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `conductor/github/project.json`
  - [x] `conductor/github/project-readme.md`
  - [x] `conductor/github/repository-settings.json`
  - [x] `conductor/github/issue-hierarchy.json`
  - [x] `scripts/bootstrap_github.py`
  - [x] `scripts/sync_github_issues.py`
  - [x] `scripts/sync_github_project.py`
  - [x] `scripts/check_github_project.py`
  - [x] `.github/workflows/github-control-plane.yml`
  - [x] `docs/github-operating-system.md`
  - [x] `contracts/json-schema/github-issue-hierarchy.v2.schema.json`
  - [x] `contracts/json-schema/github-project.v1.schema.json`

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

- [ ] Create or verify the public remote and push the committed main branch.
- [ ] Supply an authenticated GitHub token with repository, Issues and Projects permission through protected environments.
- [ ] Execute bootstrap and preserve observed repository, ruleset, issue, subissue and Project receipts.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-31-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
