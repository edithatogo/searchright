# Benchmark and calibration protocol

## Purpose

Evaluate search compilation, duplicate clustering, supplementary discovery,
screening prioritisation and reporting without allowing benchmark labels to leak
into generation or tuning decisions.

## Evaluation strata

1. **Contract conformance:** schemas, identifiers, invariants and deterministic
   serialisation.
2. **Dialect fidelity:** exact and lossy translation against curated query pairs.
3. **Retrieval sensitivity:** recall of preregistered seed records and known
   relevant records, reported by source and topic.
4. **Deduplication:** pairwise and cluster precision/recall with ambiguity review.
5. **Prioritisation:** work-saved-over-sampling, recall at screening fractions,
   calibration error and subgroup performance; ranking cannot discard records.
6. **Human calibration:** information-specialist review of strategies, warnings,
   receipts, diagnostics and generated appendices.
7. **Operational robustness:** malformed responses, retries, replay, cache,
   cancellation, hostile metadata and resource budgets.

## Leakage controls

- Keep hidden evaluation reviews separate from development fixtures.
- Hash and version every split.
- Record all model, prompt, feature and threshold changes before evaluation.
- Report failed and null runs.
- Do not use test labels for prompt construction, query expansion or stopping.

## Baselines

Deterministic no-ranking order, identifier-only deduplication, human-authored
source strategy, and the previous tagged Searchright version are mandatory
comparators. External systems are named only when their licence permits a fair,
reproducible comparison.

## Release gate

Internal benchmark success supports a fixture-proven claim only after the Rust
suite runs. It never substitutes for independent information-specialist review
or live database evidence.
