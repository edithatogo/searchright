# 38: Bounded live search, recovery and source receipts

## Objective

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

- Authorised provider contracts and explicit query/source/budget policy
- PubMed and Europe PMC cursor paging through the existing shared runtime
- Atomic checkpoints, resume and idempotent record emission
- CLI/MCP parity with denied-by-default consequential operations
- Canonical CSL export plus source-record retention and loss receipts
- Authorised two-source live run and independent retained-input replay

## Scope

- **38.1** Authorised provider contracts and explicit query/source/budget policy
- **38.2** PubMed and Europe PMC cursor paging through the existing shared runtime
- **38.3** Atomic checkpoints, resume and idempotent record emission
- **38.4** CLI/MCP parity with denied-by-default consequential operations
- **38.5** Canonical CSL export plus source-record retention and loss receipts
- **38.6** Authorised two-source live run and independent retained-input replay

## Requirements owned

`SR-094`

## Implementation ownership

- `crates/evidence-search-core/src/lib.rs`
- `crates/searchright-connectors/src/lib.rs`
- `crates/searchright/src/engine.rs`
- `crates/searchright-cli/src/main.rs`
- `crates/searchright-mcp/src/lib.rs`

## Acceptance and exclusions

TLSNotary optional and must be real notary evidence, not local Merkle/hash claims.

Source-level planning or implementation is not compiler, authorised live, real-consumer or independent methodological evidence. Screening/stopping, extraction/entailment and VOI remain with their own tools; track closure requires independently verifiable admission.

### 38.1: Authorised provider contracts and explicit query/source/budget policy

**Prerequisites:** Provider contract fixtures; fail-closed unauthorized requests.

**Acceptance:** cargo test --locked -p evidence-search-core.

**Command:** `undefined`.

### 38.2: PubMed and Europe PMC cursor paging through the existing shared runtime

**Prerequisites:** 38.1; retry/429, failed-page, duplicate-page fixtures.

**Acceptance:** cargo test --locked -p searchright-connectors.

**Command:** `undefined`.

### 38.3: Atomic checkpoints, resume and idempotent record emission

**Prerequisites:** 38.2; crash/restart and cancellation fixture.

**Acceptance:** cargo test --locked -p evidence-search-core -p searchright-store.

**Command:** `undefined`.

### 38.4: CLI/MCP parity with denied-by-default consequential operations

**Prerequisites:** 38.3; same query, source, identity and approval semantics.

**Acceptance:** cargo test --locked -p searchright-cli -p searchright-mcp.

**Command:** `undefined`.

### 38.5: Canonical CSL export plus source-record retention and loss receipts

**Prerequisites:** 38.4 and 40.1; no dropped/native source records.

**Acceptance:** cargo test --locked -p searchright-interchange.

**Command:** `undefined`.

### 38.6: Authorised two-source live run and independent retained-input replay

**Prerequisites:** 38.5, rights and access; externally checked redacted receipts.

**Acceptance:** ./scripts/verify.sh.

**Command:** `undefined`.


Follow `conductor/refinements/38-44-search-scope-20261110.json` for task identity and sequence. Keep any optional integration outside the critical path and never advertise an untested source or a paper result.
