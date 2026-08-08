# ADR 0008: Versioned standards packs with a conduct/reporting boundary

- Status: accepted
- Date: 2026-08-06

## Context

PRISMA, PRISMA-S, PRESS, JBI, Campbell, Cochrane and related guidance overlap but
are not interchangeable. A checklist badge can easily be mistaken for evidence
that a search was comprehensive or methodologically sound.

## Decision

Standards are versioned data packs with stable item identifiers, provenance,
applicability and evidence mappings. Reporting standards and methodological
conduct checks remain separate assessment dimensions. Generated assessments
record met, partial, unmet and not-applicable states with evidence locations and
unresolved gaps.

## Consequences

- Standards can evolve without hard-coding prose into orchestration logic.
- PRISMA-S support cannot be marketed as search-quality certification.
- Standards surveillance, licensing and provenance become explicit maintenance
  work.
