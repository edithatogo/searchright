# 13: Integration passports, GitHub issue hierarchy and context spine

## Objective

Make cross-repository contracts, issue hierarchy and agent context deterministic and auditable.

## Scope

- Create pinned integration passports, consumer-driven contract suites and drift checks
- Map the roadmap epic to track issues and plan phases to native subissues
- Provide dry-run-first idempotent GitHub synchronisation
- Maintain a canonical context, decision, hazard and claim-boundary spine

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `13`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Remote writes, downstream merges and compatibility claims without explicit evidence.

## Planned scoped-evidence consumer

The [rcagent consumer note](../../../docs/integration/rcagent-evidence-view.md)
adds a bounded use case for existing neutral query/provider/receipt contracts.
It keeps public literature discovery separate from local case retrieval and
requires consumer-supplied permitted-view binding before any optional adapter.
This is pending integration work, not another review engine, an admitted
passport, a new executable contract or a promotion of this track's evidence.
