# 06: Imports, deduplication and study linkage

## Objective

Import common formats, cluster duplicates conservatively and distinguish records/reports/studies.

## Scope

- Implement RIS, CSL JSON, nbib/PubMed XML and CSV readers
- Preserve source line/range provenance and malformed-record quarantine
- Expand identifier normalization and Unicode title handling
- Add report-to-study linkage model and manual merge/split
- Add large-corpus blocking/indexing without changing deterministic results
- Validate against review datasets and adversarial fixtures

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `06`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
