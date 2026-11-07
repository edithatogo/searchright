# Product context

## Vision

Make every systematic-search decision inspectable, reproducible and reusable by
humans, software and agents without surrendering methodological authority.

## Mission

Provide open, contract-first Rust infrastructure that converts review intent into
portable search strategies, bounded execution, auditable evidence, governed
screening, immutable updates and standards-aware reports.

## Purpose

Reduce methodological drift, opaque custom code and after-the-fact reporting in
systematic, scoping, rapid and living reviews.

## Users

- information specialists and health librarians;
- systematic/scoping/living review teams;
- clinical, policy, guideline and health-economic researchers;
- HTA and research-integrity teams;
- developers integrating evidence retrieval into repositories and products;
- bounded agents that need inspectable tools rather than browser automation.

## Core problem

Research questions, eligibility rules, source choices, translations, retrieval
logs, deduplication and screening decisions are fragmented across prose and
platforms. Their versions and authority are difficult to reconstruct, while each
project reimplements provider and provenance code.

## Product promise

One review contract, one shared provider kernel, one product facade and multiple
interfaces, with reproducible outputs and explicit decision authority. The
[active programme](frontier-plan.md) plans autonomous, solo and team operation;
machine-assessed and human-confirmed outputs remain distinguishable.

## Boundary

- Searchright owns planning, search validation/execution, import/export,
  deduplication, study linkage, screening, updates and reporting.
- `evidence-search-core` owns reusable query/provider/receipt/audit behaviour.
- Sourceright owns citation extraction, CSL normalisation and reference
  verification.
- Searchright is not a comprehensiveness certificate, truth engine or licensed
  scraper. The current runtime does not implement an unrestricted autonomous
  final reviewer or hosted multi-tenant product. Explicit autonomous machine
  decision policies are a planned capability, not a claim about current output.

## Strategic capabilities

1. Versioned methodological and technical contracts.
2. Portable query compilation with explicit semantic loss.
3. Bounded fixture/replay/live provider execution.
4. Source receipts, audit chains and research-object lineage.
5. Conservative deduplication and record–report–study modelling.
6. Configurable autonomous, solo and team screening with actor provenance and
   explicit policy; current protected defaults remain until implemented/tested.
7. PRISMA/PRISMA-S/PRESS and versioned standards packs.
8. CLI, MCP, Rust and skill surfaces over one facade.
9. Institutional diagnostics, privacy and data-handling policy.
10. Evidence-calibrated release, migration and public claims.

## Success measures

1. A complete deterministic review workflow replays from contracts and fixtures.
2. Every record and material decision resolves to provenance and authority.
3. Dialect translations disclose and test all material loss.
4. CLI, MCP and Rust behaviour remain in parity.
5. Sourceright adopts the shared kernel after dual-run evidence.
6. Information specialists can reproduce and critique outputs.
7. Relevant tests and independent comparisons support the features and claims
   actually shipped; unused deployment profiles do not delay the local product.
8. No public claim exceeds its evidence level.
