# 39: Conditional dialect semantics and translation fidelity

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Supported Boolean, field, proximity, truncation and controlled-vocabulary model
- Semantic comparison classification under declared assumptions
- Property and metamorphic differential tests with verified witnesses
- Measure OxiDD/SMT reuse against existing implementation, do not adopt without benefit
- Held-out real expert strategies and known-study retention benchmark

## Scope

- **39.1** Supported Boolean, field, proximity, truncation and controlled-vocabulary model
- **39.2** Semantic comparison classification under declared assumptions
- **39.3** Property and metamorphic differential tests with verified witnesses
- **39.4** Measure OxiDD/SMT reuse against existing implementation, do not adopt without benefit
- **39.5** Held-out real expert strategies and known-study retention benchmark

## Requirements owned

`SR-095`

## Implementation ownership

- `crates/evidence-search-core/src/compiler.rs`
- `crates/evidence-search-contracts/src/query.rs`
- `crates/searchright-validation/src/lib.rs`

## Acceptance and exclusions

A BDD proof is never itself proof of live database retrieval equivalence.

Source-level planning or implementation is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their own tools; track closure requires independently verifiable admission.

### 39.1: Supported Boolean, field, proximity, truncation and controlled-vocabulary model

**Prerequisites:** Existing compiler fixtures; unsupported semantics explicit Unknown.

**Acceptance:** cargo test --locked -p evidence-search-core.

**Command:** `undefined`.

### 39.2: Semantic comparison classification under declared assumptions

**Prerequisites:** 39.1; equivalence/broadening/narrowing/incomparable/unknown fixtures.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 39.3: Property and metamorphic differential tests with verified witnesses

**Prerequisites:** 39.2; independently evaluated counterexamples.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 39.4: Measure OxiDD/SMT reuse against existing implementation, do not adopt without benefit

**Prerequisites:** 39.3; memory, security, MSRV, licence, maintenance and parity report.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 39.5: Held-out real expert strategies and known-study retention benchmark

**Prerequisites:** 39.1–4, lawful source access and independent assessors.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Keep any optional integration outside the critical path and never advertise an untested source or a paper result.
