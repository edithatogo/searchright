# Rust dependency rationalisation — searchright (2026-11-13)

**Status:** scoped Conductor adoption contract; code substitutions not yet implemented or certified by this file.

## Repository-specific decisions

- Reconcile existing frontier plan and tracks 38–44 without replacing their ownership or implying scientific maturity.
- **Verified:** existing workspace already declares Anyhow, Clap, Proptest, Reqwest, Serde, Thiserror, Tokio and Tracing; Citeweft is pinned and Noyalib has a local patch.
- **First:** audit direct/transitive feature duplication and package boundaries; no blanket adding these already-adopted crates.
- **Evaluate:** Criterion for query translation/dedup/indexing, Proptest for canonical query and provider-schema invariants, Insta for bounded scrubbed CLI/MCP fixtures.
- **Conditional:** Regex for bounded text patterns (not query language parsers); Rayon and Polars for independently benchmarked deterministic analytics.
- **Retain:** evidence-span, access/licence, provider provenance and privacy contracts. Do not add SQLx, Ratatui, PyO3 or macros without an owned use case.

## Delivery and acceptance contract
- Reconcile existing Conductor tracks and ownership before editing code; do not duplicate completed tasks.
- Write a failing behavioural/regression test first for each runtime change; preserve APIs, schema versions, stdout/stderr, exit codes, error codes and MSRV or explicitly version breaking changes.
- Run the existing repo gates: targeted tests, full workspace tests, fmt, clippy and contract/interop checks. Record actual passing CI SHA; this document is planning, not validation.
- Inventory duplicate crate versions/features with `cargo tree -d` and `cargo tree -e features`; review unused dependencies, `cargo audit`, `cargo deny`, licence, MSRV, SBOM and release implications where applicable.
- Benchmark performance proposals with fixed fixtures and baseline/candidate receipts, including correctness, memory, CPU and determinism.
- Decision for each candidate must be recorded as adopt, retain, reject or defer with evidence, risks and rollback.
- Do not expose identifiable, secret, copyrighted or otherwise restricted raw data in tracing, snapshots, benchmarks or CI artifacts.

<!-- Scope: existing track 38 and shared frontier track assurance. -->
