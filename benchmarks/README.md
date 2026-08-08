# Benchmark programme

Searchright separates deterministic regression fixtures from external performance
claims. `synthetic/` is rights-clear and suitable for CI. SYNERGY and other
external corpora are opt-in, version-pinned inputs with leakage controls and
licence receipts; their absence must produce a skipped receipt rather than a
fabricated metric.

Every benchmark output validates against
`contracts/json-schema/benchmark-report.v1.schema.json` and records the corpus,
version/digest, rights basis, configuration digest, environment and claim
boundary.
