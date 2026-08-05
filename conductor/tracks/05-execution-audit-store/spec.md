# 05: Execution, audit and local storage

## Objective

Create replayable search runs, source receipts, audit chains and crash-safe local state.

## Scope

- Add canonical query+parameter hashing
- Persist records and receipts transactionally
- Add event schema registry and migration
- Add crash/restart/idempotency tests
- Implement privacy/retention/export/delete policy
- Add RO-Crate and OSF artefact export plan

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `05`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
