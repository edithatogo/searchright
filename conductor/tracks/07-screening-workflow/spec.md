# 07: Governed screening workflow

## Objective

Support independent title/abstract and full-text decisions, conflicts, reasons and agent authority.

## Scope

- Persist blinded reviewer decisions
- Implement role assignments and adjudication
- Tie decisions to eligibility version and evidence excerpts
- Enforce one primary full-text exclusion reason
- Add audit-preserving amendment/re-screen flows
- Add screening exports/adapters for common review tools

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `07`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
