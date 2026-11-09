# Searchright documentation

Start with the [repository homepage](../README.md) for the project boundary, current evidence limits, canonical CSL-JSON references, downstream consumers and local usage.

## Implementation and methods

- [Architecture](architecture.md) — neutral Rust kernel, search orchestration and related repository boundaries.
- [Contracts](contracts.md) and [schema evolution](contracts/schema-evolution.md) — versioned interfaces, compatibility and migrations.
- [Provider model](provider-model.md) and [provider governance](provider-governance.md) — bounded execution, terms, access and source observations.
- [Standards and provenance](standards-and-provenance.md) — standards profiles, reporting and inspectable evidence.
- [Vertical-slice definition of done](vertical-slice-definition-of-done.md) — what is required before a real workflow is claimed.
- [Research programme](research/programme.md) and [versioned evidence packages](research/evidence-packages.md) — proposed evaluations and research outputs.
- [CSL-first boundary and SEARCH-REPLAY](../conductor/refinements/csl-first-20261109.md) — **draft work in PR #799**, not current `main` functionality; [view PR](https://github.com/edithatogo/searchright/pull/799).

## Governance and engineering

- [Conductor track inventory](../conductor/tracks.md) — canonical implementation/evidence ownership, including historical scopes.
- [Architecture fitness](quality/architecture-fitness.md) and [evidence debt](quality/evidence-debt.md) — executable architecture and remaining claims.
- [Security](security/threat-model.md), [recovery rehearsal](operations/recovery-reference-rehearsal.md) and [ADRs](adrs/) — threat boundaries, restore limits and decisions.
- [Estate integration](estate-integration.md) — downstream migration and compatibility planning.
- [Registry/publication rules](registry-publication.md) — external evidence and release gates.

**Scope caution:** older sections and track specifications may still describe Searchright-owned screening or novel downstream algorithms. The current product direction centres on systematic search and canonical corpus construction, with screening and analytics owned by external consumers. Do not interpret planning or archived evidence as implemented functionality.
