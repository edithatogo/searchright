# ADR 0013: CiteWeft document-evidence boundary

- Status: Accepted
- Date: 2026-08-06

## Decision

Adopt CiteWeft as an optional lower-level scholarly extraction dependency through `searchright-citeweft`. Persist only Searchright's backend-neutral `DocumentEvidence` contract. No extraction backend may write canonical bibliographic data.

## Consequences

The shared search core remains small and independent. CiteWeft and GROBID can evolve behind a consumer-driven compatibility suite. Searchright can use source-grounded full-text evidence for screening and study linkage without becoming a PDF parser or citation canonicaliser.
