# 41: Temporal acquisition and versioned living-search corpora

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Distinct logical corpus/content revision/observation/run/attempt/profile identities
- New, changed, withdrawn, corrected and delayed-index source observations
- Narrow safe invalidation for strategy, metadata, vocabulary and identity changes
- Full-replay equivalence versus incremental results
- Qualify Salsa/DVC against simple existing Rust/CI baseline

## Scope

- **41.1** Distinct logical corpus/content revision/observation/run/attempt/profile identities
- **41.2** New, changed, withdrawn, corrected and delayed-index source observations
- **41.3** Narrow safe invalidation for strategy, metadata, vocabulary and identity changes
- **41.4** Full-replay equivalence versus incremental results
- **41.5** Qualify Salsa/DVC against simple existing Rust/CI baseline

## Requirements owned

`SR-097`

## Implementation ownership

- `crates/searchright-living/src/lib.rs`
- `crates/searchright-dedup/src/lib.rs`
- `crates/searchright-interchange/src/csl_corpus.rs`

## Acceptance and exclusions

Salsa and DVC are optional; fresh live source observation is never inferred from cache.

Source-level planning is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their downstream owners; track closure requires verified acceptance.

### 41.1: Distinct logical corpus/content revision/observation/run/attempt/profile identities

**Prerequisites:** 40.1–2; immutable identity tests.

**Acceptance:** cargo test --locked -p searchright-living.

**Command:** `undefined`.

### 41.2: New, changed, withdrawn, corrected and delayed-index source observations

**Prerequisites:** 41.1; three-wave delta fixtures.

**Acceptance:** cargo test --locked -p searchright-living.

**Command:** `undefined`.

### 41.3: Narrow safe invalidation for strategy, metadata, vocabulary and identity changes

**Prerequisites:** 41.2; dependency-change metamorphic fixtures.

**Acceptance:** cargo test --locked -p searchright-living.

**Command:** `undefined`.

### 41.4: Full-replay equivalence versus incremental results

**Prerequisites:** 41.3; property tests and explicit ordering semantics.

**Acceptance:** cargo test --locked -p searchright-living.

**Command:** `undefined`.

### 41.5: Qualify Salsa/DVC against simple existing Rust/CI baseline

**Prerequisites:** 41.4; measured value/maintenance/rollback decision.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Do not advertise an untested source or research result.
