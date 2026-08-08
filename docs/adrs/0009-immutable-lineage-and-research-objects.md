# ADR 0009: Immutable review lineage and research-object exports

- Status: accepted
- Date: 2026-08-06

## Context

Living reviews, protocol amendments and reruns need to show what changed without
rewriting prior searches. A folder of current-state JSON is insufficient for
external audit or durable deposit.

## Decision

Review plans, strategies, runs, amendments, decisions and reports use immutable
identifiers and explicit predecessor links. Searchright emits deterministic
RO-Crate and W3C PROV representations from the same contracts and audit events.
Exports are descriptive evidence packages, not truth certificates.

## Consequences

- Living updates can be compared and deposited reproducibly.
- Provenance is a first-class product output rather than release decoration.
- Schema evolution must retain resolvable historical identifiers.
