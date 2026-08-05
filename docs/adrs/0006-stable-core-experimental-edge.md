# ADR 0006: Stable core, experimental edge

Status: accepted

The default build uses stable Rust and released dependencies. Experimental MCP,
WASI, simulation and agent features are isolated behind explicit feature flags,
revision pins and fallback paths. “Bleeding edge” does not justify making the
core unreproducible.
