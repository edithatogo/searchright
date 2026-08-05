# 01: Contract catalogue and code generation

## Objective

Stabilise versioned review, source, query, record, screening, audit and reporting contracts.

## Scope

- Validate all JSON Schemas and examples
- Generate schemas from Rust types and diff against checked-in canonical forms
- Add compatibility fixtures and migration/version rules
- Add report/study linkage and protocol-amendment contracts
- Generate TypeScript/Python bindings only as thin contract packages
- Publish contract conformance matrix

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `01`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
