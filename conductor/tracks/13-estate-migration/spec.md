# 13: GitHub estate audit and custom-code replacement

## Objective

Find, classify and replace duplicate systematic-search code across edithatogo repositories.

## Scope

- Run code/repository search for providers, PubMed, PRISMA, dedup and screening
- Populate migration manifest with exact files/owners/status
- Prioritise active research repos and shared skill repos
- Create thin adapters or issues per repository
- Verify behavior before deletion and preserve provenance
- Re-run estate conformance and close superseded tracks

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `13`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
