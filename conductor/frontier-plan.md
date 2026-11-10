# Searchright — CSL-first systematic-search implementation

Updated 10 November 2026. This is an **implementation queue**, not a declaration of completed work. The former active tracks' screening/entailment/VOI algorithms are superseded by their documented search-specific contracts. Archived track history remains intact.

## Scope

Searchright owns systematic search and provenance-rich CSL-JSON corpus. Sourceright reconciles references; CiteWeft extracts; ASReview screens; pybibx/bibliometrix/VOSviewer/tobac/voiage analyse.

**Canonical reference format:** CSL-JSON. Stable reference IDs are reconciled by Sourceright. Source observations, metadata lineage, citation edges, authorship and affiliation details, rights and consumer-loss descriptions use separate revision-bound sidecars. CSV/RIS/BibTeX and native bibliometric formats are derived projections, not alternate authorities.

## Dependency-aware sequence

| Order | Tasks | Completion gate |
| --- | --- | --- |
| 0 | Security and active PR reconciliation | Exact-head CI/security, no fabricated cargo-vet approvals |
| 1 | 40.1 | Canonical CSL-JSON normalisation and BLAKE3 digest using existing serde_json/blake3 |
| 2 | 38.1–38.6 | Authorised resumable two-source search and independent retained-response replay |
| 3 | 40.2–40.6 | Source/identity/rights sidecars, ASReview, pybibx and bibliometric consumer acceptance |
| 4 | 39.1–39.5; 42.1–42.5 | Semantic loss, faithful provenance and verification negative controls |
| 5 | 43.1–43.5 | Published-strategy SEARCH-REPLAY, reproducible interpretation uncertainty, expert checking |
| 6 | 41.1–41.5 | Temporal source observations, versioned deltas and full replay equivalence |
| 7 | 44.1–44.5 | Generic full/delta corpus handoffs and independently checked consumer profiles |

Independent tasks may run in parallel when declared prerequisites exist; no mandatory replacement of useful existing code. DVC, Salsa, OxiDD, TLSNotary, Snakemake, ASReview extensions, local agent auditing and OpenLineage are optional only after compatibility/value/security qualification. Searchright does not implement the ASReview screening/active-learning core.

## Decision identity and operational modes

Preserve actor, mode and policy version across autonomous, solo and team workflows; machine recommendations are not independent human reviews and cannot autonomously amend a search strategy. Retain immutable original source observations, individual interventions and actual human approvals. The existing screening owner remains ASReview.

## Conductor tracks

- [Track 38: Bounded live search, recovery and source receipts](tracks/38-zktls-verifiable-live-execution/plan.md) — 6 individually executable gated tasks
- [Track 39: Conditional dialect semantics and translation fidelity](tracks/39-smt-query-equivalence-calculus/plan.md) — 5 individually executable gated tasks
- [Track 40: Canonical CSL-JSON and downstream consumer interoperability](tracks/40-martingale-conformal-stopping/plan.md) — 6 individually executable gated tasks
- [Track 41: Temporal acquisition and versioned living-search corpora](tracks/41-salsa-incremental-living-reviews/plan.md) — 5 individually executable gated tasks
- [Track 42: Search provenance, integrity notices and independent evidence packages](tracks/42-subdocument-entailment-retraction/plan.md) — 5 individually executable gated tasks
- [Track 43: Search strategy auditing and SEARCH-REPLAY evaluation](tracks/43-adversarial-multi-agent-deliberation/plan.md) — 5 individually executable gated tasks
- [Track 44: Generic versioned corpus and downstream-research handoff](tracks/44-tobac-living-evidence-transport/plan.md) — 5 individually executable gated tasks

## Review/archival contract

Use `conductor/refinements/38-44-search-scope-20261110.json` as the deterministic action ledger and stable track IDs 38–44. Perform a real `$conductor-review` only where its runtime is actually available. Otherwise perform repository-native static/traceability checks, Rust CI and external/consumer checks, explicitly recording which Conductor checks could not be invoked. Do not tick unobserved tests or archive merely because source documents have been revised. After the implementation evidence is present, reconcile plan, metadata, coverage, traceability, PR and README. Only then use the existing archival validator.

## Research

Evaluate search translation fidelity, deduplication loss, bibliometric corpus sensitivity, downstream ASReview handoff, temporal search drift, independent package reproduction and SEARCH-REPLAY published-strategy variability. Formal research contracts and publication-ready claims require dated source-linked experiments and independent comparison.
