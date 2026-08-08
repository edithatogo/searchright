# 17: Benchmarks, search validation and human calibration

## Objective

Evaluate query translation, deduplication and screening against rights-cleared corpora.

## Scope

- Define benchmark report and anti-leakage contracts
- Integrate SYNERGY with dataset provenance and splits
- Create search-strategy gold sets and known-item recall tests
- Measure dedup precision/recall and cluster review burden
- Run prospective multi-model pilot and human calibration
- Publish limitations and reproducible benchmark artefacts

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `17`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
