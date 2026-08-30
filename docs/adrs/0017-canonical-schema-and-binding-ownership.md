# ADR 0017: Canonical schema and generated binding ownership

- Status: accepted
- Date: 2026-08-29

## Context

Searchright has canonical JSON Schema 2020-12 documents and Rust types derived
with `serde` and `schemars`. Schemars cannot express every canonical validation
keyword or Searchright cross-field invariant without duplicating the canonical
contract inside Rust-specific transforms. Treating generated and canonical
documents as exactly equivalent would therefore conceal constraint loss.

Track 01 also requires Python and TypeScript representations, while Track 35
owns generated clients, installation, adoption and downstream evidence.

## Decision

- Checked-in JSON Schemas remain the canonical machine contracts.
- Rust-generated schemas are compiler-backed drift diagnostics. Their complete
  validation-shape difference set is recorded in
  `contracts/compatibility/rust-schema-parity.json` and exact semantic parity is
  fail-closed whenever any difference remains.
- Contract-only Python and TypeScript types are generated deterministically from
  the canonical schema catalogue. They contain no transport or domain logic and
  remain private and non-publishable.
- Track 35 separately owns generated clients, package installation, consumer
  conformance, adoption and any publication decision.

## Consequences

- Constraint loss cannot be hidden behind root-field equality.
- Canonical schema evolution deterministically invalidates stale bindings and
  the recorded Rust parity report.
- Generated contract packages may be compiler-checked locally without implying
  client support, downstream compatibility or publication readiness.
