# 12: Sourceright migration and shared releases

## Objective

Adopt evidence-search-core in Sourceright and coordinate compatible versioning/releases.

## Scope

- Create Sourceright branch/PR and compatibility facade
- Run all existing Sourceright fixtures through shared core
- Replace custom generic code in bounded commits
- Document retained Sourceright-specific responsibilities
- Publish coordinated crates/releases and migration guide
- Remove deprecated paths after one release cycle

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `12`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
