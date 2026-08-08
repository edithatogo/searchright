# ADR 0012: Replace duplicate provider code through parity, not bulk deletion

- Status: accepted
- Date: 2026-08-06

## Context

Sourceright already contains provider HTTP, retry, cache and endpoint behavior.
Other repositories may contain bespoke systematic-search scripts. Approximate
code search is insufficient evidence that code is equivalent or safe to remove.

## Decision

`evidence-search-core` owns product-neutral query execution, policy, receipts and
audit primitives. Sourceright retains citation and CSL semantics. Migration uses
an inspected-symbol map, deterministic parity cases, feature-gated rollback,
semver review and a downstream compiler/test gate. The wider estate uses a
candidate inventory followed by repository-specific replacement decisions.

## Consequences

- Shared-core adoption is incremental and reversible.
- Existing caches remain until replay/cache parity is demonstrated.
- Remote code is never described as replaced until the target repository records
  the integration and deletion evidence.
