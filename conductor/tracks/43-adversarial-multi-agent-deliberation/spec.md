# 43: Search strategy auditing and SEARCH-REPLAY evaluation

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Evidence-location-aware PRESS/PRISMA-S strategy audit
- Literal and plausible query reconstruction scenarios with frozen assumptions
- Record-set differences, known-study retention and interpretation uncertainty envelopes
- Optional agent audits with deterministic verification and human amendments
- SEARCH-REPLAY pilot/confirmation protocol, research datasets and publication contracts

## Scope

- **43.1** Evidence-location-aware PRESS/PRISMA-S strategy audit
- **43.2** Literal and plausible query reconstruction scenarios with frozen assumptions
- **43.3** Record-set differences, known-study retention and interpretation uncertainty envelopes
- **43.4** Optional agent audits with deterministic verification and human amendments
- **43.5** SEARCH-REPLAY pilot/confirmation protocol, research datasets and publication contracts

## Requirements owned

`SR-099`

## Implementation ownership

- `crates/searchright-validation/src/lib.rs`
- `crates/searchright-agent/src/lib.rs`
- `docs/research/programme.md`

## Acceptance and exclusions

No new multi-agent screening arbitration or simulated independent human reviewers.

Source-level planning is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their downstream owners; track closure requires verified acceptance.

### 43.1: Evidence-location-aware PRESS/PRISMA-S strategy audit

**Prerequisites:** Existing standards; distinguish missing, ambiguous, contradictory, inaccessible.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 43.2: Literal and plausible query reconstruction scenarios with frozen assumptions

**Prerequisites:** 43.1; negative tests for auto-repair.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 43.3: Record-set differences, known-study retention and interpretation uncertainty envelopes

**Prerequisites:** 43.2/38.6; index drift separate and no false confidence intervals.

**Acceptance:** cargo test --locked -p searchright-validation.

**Command:** `undefined`.

### 43.4: Optional agent audits with deterministic verification and human amendments

**Prerequisites:** 43.1–3; prompt injection and false expert-independence tests.

**Acceptance:** cargo test --locked -p searchright-agent.

**Command:** `undefined`.

### 43.5: SEARCH-REPLAY pilot/confirmation protocol, research datasets and publication contracts

**Prerequisites:** 43.3–4; independently reviewable design and rights gates.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Do not advertise an untested source or research result.
