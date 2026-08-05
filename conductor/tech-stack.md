# Technical Stack

## Stable core

- Rust 1.97.1, edition 2024, Cargo resolver 3.
- `serde`, `schemars` and checked-in JSON Schema 2020-12 contracts.
- `tokio` async runtime; `reqwest` 0.13 with rustls for opt-in network adapters.
- official `rmcp` 3.1 for MCP 2026-07-28, retaining compatibility planning for
  MCP 2025-11-25 clients.
- BLAKE3 canonical event hashes and append-only JSONL in the MVP.
- `clap` CLI over the same facade used by MCP.

## Experimental edge, feature-gated

- WASI component providers through the checked-in WIT contract and Wasmtime.
- Arrow/Parquet plus DuckDB/Polars for large review corpora.
- LanceDB or equivalent local hybrid retrieval for optional prioritisation.
- deterministic simulation, metamorphic query testing and model checking of
  workflow invariants.
- MCP tasks, subscriptions, response caching, discovery negotiation and
  Streamable HTTP after stdio conformance.
- active-learning and agent-team features only after human calibration and
  leakage-safe benchmarks.

Experimental dependencies must remain behind features, be pinned or range-bounded,
have an ADR and exit strategy, and never become required for the deterministic
core.

## Interfaces

- Rust facade crate `searchright`.
- CLI binary `searchright`.
- local stdio MCP binary `searchright-mcp`.
- future authenticated Streamable HTTP service.
- cross-agent skill package under `skills/systematic-search/`.

## Quality and supply chain

- `cargo fmt`, Clippy, rustdoc, nextest, llvm-cov, cargo-mutants, proptest,
  fuzzing, cargo-deny, cargo-audit, cargo-semver-checks and cargo-machete.
- CodeQL, Scorecard, zizmor, actionlint, Renovate and Dependabot alerts.
- signed release artefacts, checksums, CycloneDX/SPDX SBOM, SLSA provenance and
  OCI attestations.
- repository-level deterministic gate plus opt-in live smoke suite.
