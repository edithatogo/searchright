# 10: MCP stdio server MVP

## Objective

Expose typed, authority-annotated MCP tools over the same facade.

## Scope

- Complete contracted planning/execution/screening tools
- Return structuredContent matching outputSchema
- Add resources for plans, runs, queues and reports
- Add prompts for planning/PRESS/update workflows
- Run official SDK/examples and protocol transcript conformance
- Test cancellation, pagination, errors and backwards compatibility

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `10`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
