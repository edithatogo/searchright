# Methodological benchmark contract

This directory contains rights-clear synthetic validation fixtures and a sealed
case manifest for Searchright's methodological evaluation surfaces. It is a
benchmark *contract*, not a performance result.

Development-visible fixtures exercise expected data shapes for deduplication,
report-to-study linkage, screening prioritisation and living-review change
detection. The sealed manifest contains identifiers only. Final evaluation
labels, model outputs and metric results must be mounted independently at run
time and recorded in a benchmark receipt so that agents and development prompts
cannot train on or inspect the final test answers.

The canonical SYNERGY source is the upstream ASReview project. A personal fork
may be used only as a pinned mirror or patch carrier and must never silently
replace the canonical upstream identity in a benchmark receipt.

Run the network-free policy check with:

```text
python scripts/check_methodology_benchmarks.py
```

This check proves fixture integrity, provenance declarations and anti-leakage
policy only. It does not execute Searchright, compare models, or establish
screening, deduplication or retrieval performance.
