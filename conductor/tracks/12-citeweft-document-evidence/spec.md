# 12: CiteWeft scholarly extraction and document evidence

## Objective

Adopt CiteWeft as an optional, one-way extraction dependency and preserve neutral document evidence.

## Scope

- Pin the CiteWeft revision and feature-gate the adapter
- Preserve references, callouts, source spans, uncertainty, diagnostics and provenance
- Forbid canonical writes and default full-text retention
- Prepare downstream Sourceright compatibility and rollback

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `12`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Remote writes, downstream merges and compatibility claims without explicit evidence.
