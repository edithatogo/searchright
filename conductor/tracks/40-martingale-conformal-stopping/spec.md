# 40: Canonical CSL-JSON and downstream consumer interoperability

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Canonical CSL-JSON ID/order/field-preservation and content-revision primitive
- Versioned rights, lineage, source-record, citation-edge, affiliation and identity sidecars
- Pinned ASReview CSV/RIS import and optional extension with corpus-to-label crosswalk
- Pinned pybibx adapter with truthful supported data source/neutral export
- bibliometrix and VOSviewer imports preserving available citation/affiliation edges
- Negative round-trip tests across consumers and immutable revision-bound return events

## Scope

- **40.1** Canonical CSL-JSON ID/order/field-preservation and content-revision primitive
- **40.2** Versioned rights, lineage, source-record, citation-edge, affiliation and identity sidecars
- **40.3** Pinned ASReview CSV/RIS import and optional extension with corpus-to-label crosswalk
- **40.4** Pinned pybibx adapter with truthful supported data source/neutral export
- **40.5** bibliometrix and VOSviewer imports preserving available citation/affiliation edges
- **40.6** Negative round-trip tests across consumers and immutable revision-bound return events

## Requirements owned

`SR-096`

## Implementation ownership

- `crates/searchright-interchange/src/lib.rs`
- `crates/searchright-interchange/src/csl_corpus.rs`
- `crates/searchright-contracts/src/interchange.rs`

## Acceptance and exclusions

ASReview screening/stopping and bibliometric network analysis remain downstream.

Source-level planning or implementation is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their own tools; track closure requires independently verifiable admission.

### 40.1: Canonical CSL-JSON ID/order/field-preservation and content-revision primitive

**Prerequisites:** Existing serde_json/blake3, no new deps, adversarial unit tests.

**Acceptance:** cargo test --locked -p searchright-interchange csl_corpus.

**Command:** `undefined`.

### 40.2: Versioned rights, lineage, source-record, citation-edge, affiliation and identity sidecars

**Prerequisites:** 40.1; upstream CSL schema and cross-artifact invariants.

**Acceptance:** python scripts/validate_repository.py.

**Command:** `undefined`.

### 40.3: Pinned ASReview CSV/RIS import and optional extension with corpus-to-label crosswalk

**Prerequisites:** 40.2; real ASReview consumer tests, no Searchright screening model.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.

### 40.4: Pinned pybibx adapter with truthful supported data source/neutral export

**Prerequisites:** 40.2; real pybibx import and field-loss report.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.

### 40.5: bibliometrix and VOSviewer imports preserving available citation/affiliation edges

**Prerequisites:** 40.2; independent expected-count and missingness fixtures.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.

### 40.6: Negative round-trip tests across consumers and immutable revision-bound return events

**Prerequisites:** 40.3–40.5; no canonical overwrite or silent loss.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Keep any optional integration outside the critical path and never advertise an untested source or a paper result.
