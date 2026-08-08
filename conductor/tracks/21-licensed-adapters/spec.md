# 21: Licensed BYO-access adapters

## Objective

Support Embase, Scopus and Web of Science without bundling access or evading terms.

## Scope

- Legal/licence/terms review and adapter boundary
- Credential redaction and local-only secret injection
- Provider-specific pagination/rate/cache contracts
- Synthetic and user-supplied fixture conformance
- Opt-in smoke tooling that stores no licensed payload by default
- Separate support claims per provider/platform

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `21`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
