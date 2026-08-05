# 03: Shared provider runtime and Sourceright extraction

## Objective

Move product-neutral execution concerns from Sourceright into evidence-search-core with parity and rollback.

## Scope

- Inventory Sourceright live_providers and provider modules
- Define compatibility adapter and feature flag
- Port retries, cache, endpoint, rate and receipt behavior
- Run old/new fixture parity and snapshot differences
- Review semver/public API implications
- Switch Sourceright then remove superseded code after evidence

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `03`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
