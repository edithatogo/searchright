# 04: Open provider connectors MVP

## Objective

Provide fixture-backed and opt-in live PubMed, Europe PMC, Crossref and OpenAlex adapters.

## Scope

- Finish PubMed ESearch+EFetch/ESummary report retrieval
- Harden Europe PMC response normalization
- Add Crossref and OpenAlex discovery adapters
- Add NCBI/Europe PMC identification and rate guidance
- Capture raw response hashes and replay fixtures
- Run redacted opt-in live smokes

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `04`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
