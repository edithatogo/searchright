# Rust Style Guide

- Forbid unsafe code in first-party crates. Exceptions require a dedicated ADR,
  isolated crate and Miri/sanitiser evidence.
- Prefer explicit domain types over stringly-typed maps at public boundaries.
- Derive `serde` and `schemars` together for contract types.
- Validate semantic and cross-field invariants beyond JSON Schema.
- No `unwrap`, `expect`, `todo`, `unimplemented`, debug prints or silent default
  recovery in production code.
- All allow attributes require a reason.
- Preserve deterministic iteration order and canonical serialisation.
- Network adapters are async, bounded, allowlisted, redacted and disabled by
  default.
- Errors identify provider/contract/stage without leaking credentials or full
  sensitive payloads.
- CLI and MCP are thin adapters; business logic belongs in core/product crates.
- Public items are documented; examples use fixtures and no live credentials.
- Tests cover normal, error, boundary, metamorphic and adversarial cases.
