# StandardFlow integration boundary

SearchRight and StandardFlow are complementary systems, not interchangeable implementations.

## Authority

SearchRight owns review planning, source selection, query translation, retrieval, deduplication, screening, record–report–study linkage, living-update lineage and the authoritative PRISMA count ledger. StandardFlow owns standards identity and versions, applicability, normalized requirements, evidence expectations and standards-derived artefact rendering.

## Exchange

SearchRight may consume an exact-pinned, rights-cleared Canonical Standard Pack and artefact recipe. It may emit a versioned `review_flow` evidence envelope containing its authoritative counts and lineage reference. StandardFlow may validate the envelope against the selected recipe and render it; it cannot alter screening decisions, silently repair inconsistent counts or infer absent lineage.

## Failure and rollback

The integration is local, read-only, network-off and write-off by default. SearchRight retains its current versioned standards packs and native rendering path as the degraded mode. Pin changes require review of schemas, rights, fixtures and semantic differences. A failed canary restores the prior pin and preserves the receipt.

The verified StandardFlow foundation is pinned at `f248d7c4dd40dd34cb45b5b6150b4750a5b28864` from `edithatogo/standards_check#59`. The primary foundation contract and Rust gates passed at this revision; merge, consumer parity and external validation remain separate evidence.
