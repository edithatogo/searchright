# 02: Portable query AST and dialect compilers

## Objective

Deliver deterministic, reviewable translation across major search dialects.

## Scope

- Complete field/proximity/truncation semantics
- Add line-numbered native strategy representation
- Build PubMed/Ovid MEDLINE/Embase/CINAHL/PsycINFO/Scopus/Web of Science corpus
- Add named filter contracts with source/version/citation
- Add syntax parsers for supported native dialect subsets
- Create metamorphic and round-trip translation tests

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `02`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
