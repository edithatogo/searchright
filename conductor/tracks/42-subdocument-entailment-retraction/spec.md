# 42: Search provenance, integrity notices and independent evidence packages

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Reference-field provenance and rights sidecars with source observation links
- Deterministic manifest of native query, source, CSL, sidecar, code and environment revisions
- Correction, withdrawal and retraction notices, without changing downstream conclusions
- Independent preserved-input replay and missing/tampered asset rejection
- Evaluate RO-Crate compatibility, optional TLSNotary and OpenLineage

## Scope

- **42.1** Reference-field provenance and rights sidecars with source observation links
- **42.2** Deterministic manifest of native query, source, CSL, sidecar, code and environment revisions
- **42.3** Correction, withdrawal and retraction notices, without changing downstream conclusions
- **42.4** Independent preserved-input replay and missing/tampered asset rejection
- **42.5** Evaluate RO-Crate compatibility, optional TLSNotary and OpenLineage

## Requirements owned

`SR-098`

## Implementation ownership

- `crates/searchright-provenance/src/lib.rs`
- `crates/searchright-living/src/lib.rs`
- `crates/searchright-interchange/src/csl_corpus.rs`

## Acceptance and exclusions

Document extraction stays with CiteWeft, canonical reference verification with Sourceright.

Source-level planning is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their downstream owners; track closure requires verified acceptance.

### 42.1: Reference-field provenance and rights sidecars with source observation links

**Prerequisites:** 40.2/38.5; fail unresolved identities and restricted disclosures.

**Acceptance:** cargo test --locked -p searchright-provenance.

**Command:** `undefined`.

### 42.2: Deterministic manifest of native query, source, CSL, sidecar, code and environment revisions

**Prerequisites:** 42.1; cross-platform content and tamper tests.

**Acceptance:** cargo test --locked -p searchright-provenance.

**Command:** `undefined`.

### 42.3: Correction, withdrawal and retraction notices, without changing downstream conclusions

**Prerequisites:** 42.2; notification propagation tests.

**Acceptance:** cargo test --locked -p searchright-living.

**Command:** `undefined`.

### 42.4: Independent preserved-input replay and missing/tampered asset rejection

**Prerequisites:** 42.2–3; non-author verifier fixture.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.

### 42.5: Evaluate RO-Crate compatibility, optional TLSNotary and OpenLineage

**Prerequisites:** 42.4; independently validated versions and no fabricated proof.

**Acceptance:** python scripts/validate_repository.py.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Do not advertise an untested source or research result.
