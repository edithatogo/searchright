# ADR 0011: Separate source implementation from executable and external evidence

- Status: accepted
- Date: 2026-08-06

## Context

A large source scaffold can appear complete while never having compiled or run.
Conversely, a passing unit test does not prove live database behavior, usability
or methodological validity.

## Decision

Tracks and public claims use an evidence ladder: contracted, source-verified,
compiler-verified, fixture-proven, opt-in live proven, externally validated and
publicly accepted. Workflow transitions and authority constraints are expressed
as executable finite-state invariants. Every track has a machine-readable source
evidence receipt and explicit blockers for higher levels.

## Consequences

- Roadmap completeness can be audited without false completion.
- External and licensed operations remain open tasks even when source support is
  implemented.
- Version 1.0 requires a multi-domain maturity dossier, not a source-code tally.
