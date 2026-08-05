# 20: Living review and update surveillance

## Objective

Make prior work, reruns, amendments, drift and update reporting first-class.

## Scope

- Immutable run lineage and supersession model
- Date/cursor/high-water update strategies
- Cross-run deduplication and changed-record detection
- Alerting with human triage and no automatic inclusion
- PRISMA-LSR reporting and cadence dashboard
- Reproducible scheduled execution with approval policy

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `20`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
