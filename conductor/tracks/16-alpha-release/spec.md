# 16: Alpha release and distribution

## Objective

Ship signed, install-tested technical-preview binaries/crate/image without overclaiming.

## Scope

- Set semver/public API policy and generate Cargo.lock
- Create cross-platform release matrix
- Generate checksums, SBOM, signatures and provenance
- Install-smoke CLI/MCP and package dry-runs
- Publish docs site, changelog and migration notes
- Tag only after release evidence contract passes

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
