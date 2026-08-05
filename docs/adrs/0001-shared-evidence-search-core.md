# ADR 0001: Shared evidence-search core

Status: accepted

## Decision

Create `evidence-search-core` as a product-neutral crate consumed by Searchright
and Sourceright. It owns query ASTs, dialect compilation, provider execution,
rate/retry policy, evidence receipts and audit primitives. Response caching and replay are staged follow-on capabilities.

## Consequences

Sourceright retains CSL and citation verification. Searchright owns review
planning, deduplication, screening and PRISMA. Generic runtime code is migrated
once fixture parity is demonstrated.
