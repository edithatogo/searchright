# 16: Maximal quality, context and security harness

## Objective

Achieve >90% tested, secure, reproducible and supply-chain-hardened engineering.

## Scope

- Generate lockfile and make deterministic CI green
- Unit/integration/e2e/property/metamorphic/DST/CDC/fuzz/mutation tests
- Configure llvm-cov >90% and Codecov
- Run cargo-deny/audit/semver/machete, CodeQL, Scorecard, zizmor, actionlint
- Add SSRF, secret leakage, parser bomb and resource exhaustion tests
- Produce machine-readable verification receipt on every release

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `16`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
