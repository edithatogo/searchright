# ADR 0007: One application facade for every public interface

- Status: accepted
- Date: 2026-08-06

## Context

Independent CLI, MCP and library implementations make authority, validation,
error and output behavior drift. That is especially unsafe for screening,
protocol amendments and network execution.

## Decision

`searchright::SearchrightEngine` is the application boundary. CLI commands, MCP
tools and embedders delegate to it. `contracts/interface-catalog.json` records
each operation, interface name, facade method, authority and stability. Source
parity is a required static gate; executable CLI and MCP transcript parity remain
compiler-backed gates.

## Consequences

- Core behavior has one implementation and one error taxonomy.
- Interface-specific code is limited to parsing, transport and presentation.
- A public operation cannot be added to one interface without an explicit parity
  decision.
- Static parity does not replace binary or protocol conformance testing.
