# 09: CLI MVP

## Objective

Provide a stable scriptable CLI over all fixture-backed MVP operations.

## Scope

- Add init/plan/source/strategy/run/import/screen/report command hierarchy
- Support JSON output and actionable diagnostics everywhere
- Add dry-run/apply semantics for writes
- Add shell completions and man pages
- Add snapshot, install and cross-platform end-to-end tests
- Publish CLI compatibility policy

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `09`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
