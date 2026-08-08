# ADR 0015: Pinned integration passports

## Status

Accepted.

## Decision

Every active cross-repository integration uses an exact Git revision and a
versioned passport declaring contract inputs/outputs, dependency direction,
feature or capability boundary, verification gates, default effects, rollback
and claim boundary.

Git submodules and copied implementation code are not the default integration
mechanism. Prefer, in order: neutral schemas and fixtures, optional Rust
features, CLI JSON/JSONL, MCP, capability-bounded WASI components, and generated
adapters. Scheduled drift checks may report change but cannot update pins.

## Consequences

Repositories retain independent release histories. Compatibility becomes a
consumer-driven evidence question rather than an assumption based on a shared
owner or matching type name.
