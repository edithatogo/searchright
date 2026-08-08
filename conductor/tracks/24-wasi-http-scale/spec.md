# 24: WASI plugins, advanced MCP and scale

## Objective

Create sandboxed providers, authenticated remote MCP and large-corpus storage.

## Scope

- Implement WIT host/guest SDK and signed manifest verification
- Adopt Wasmtime resource/fuel/network capability controls
- Add Arrow/Parquet/DuckDB feature for large corpora
- Implement MCP discovery/tasks/subscriptions/cache/Streamable HTTP
- Add OAuth, tenancy, quotas, observability and deployment threat model
- Load/chaos/failover and compatibility testing

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `24`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
