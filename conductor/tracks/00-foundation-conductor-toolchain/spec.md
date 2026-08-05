# 00: Foundation, Conductor and toolchain

## Objective

Establish the repository, current Conductor context, toolchain pin, standards inheritance and reproducible bootstrap.

## Scope

- Record upstream Conductor 0.3.0 and supported host installation paths
- Create product, guidelines, stack, workflow, requirements and design context
- Pin Rust stable and document nightly/experimental policy
- Generate and commit Cargo.lock in a networked Rust environment
- Register repository in repository-standards and run estate conformance
- Create one-command bootstrap and verification receipts

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `00`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
