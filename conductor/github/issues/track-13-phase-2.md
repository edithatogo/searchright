<!-- searchright-issue-key: track-13-phase-2 -->
# Track 13 / Phase 2: Source-level verification

Parent track key: `track-13`
Conductor plan: `conductor/tracks/13-integration-passports-github-context/plan.md`

## Phase tasks

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
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Evidence rule

Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.
