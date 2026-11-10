# 44: Generic versioned corpus and downstream-research handoff

## Objective

tobac/voiage own effects, bias transport and VOI; source neutrality is the default.

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

## Scope

- **44.1** Consumer-neutral versioned CSL+sidecar+rights+identity manifest
- **44.2** Deterministic full and delta corpora with typed change notices
- **44.3** Optional tobac/voiage adapter, with no invented effect or risk-of-bias calculation
- **44.4** Consumer compatibility, SemVer, access and tamper negative tests
- **44.5** Reproducible package and versioned API release once exact-head gates pass

## Requirements owned

`SR-100`

## Implementation ownership

- `crates/searchright-interchange/src/lib.rs`
- `crates/searchright-provenance/src/lib.rs`
- `crates/searchright-living/src/lib.rs`

## Acceptance

### 44.1: Consumer-neutral versioned CSL+sidecar+rights+identity manifest

- Prerequisites: 40/42; reference resolution and access invariants
- Evidence: cargo test --locked -p searchright-interchange
- Test: `undefined`

### 44.2: Deterministic full and delta corpora with typed change notices

- Prerequisites: 44.1/41; full-replay fixture parity
- Evidence: cargo test --locked -p searchright-living
- Test: `undefined`

### 44.3: Optional tobac/voiage adapter, with no invented effect or risk-of-bias calculation

- Prerequisites: 44.2; real downstream parse and loss accounting
- Evidence: ./scripts/verify.sh
- Test: `undefined`

### 44.4: Consumer compatibility, SemVer, access and tamper negative tests

- Prerequisites: 44.3; two independently qualified imports
- Evidence: ./scripts/verify.sh
- Test: `undefined`

### 44.5: Reproducible package and versioned API release once exact-head gates pass

- Prerequisites: 44.4; no publication/registry acceptance claim
- Evidence: ./scripts/verify.sh
- Test: `undefined`

Planned scope is not working, live, externally reproducible or validated behaviour. Searchright owns systematic search and CSL-JSON corpora; screening, extraction, entailment, bibliometrics and VOI belong downstream. No track archive without verified acceptance.
