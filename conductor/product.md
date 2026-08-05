# Product Context

## Product

Searchright is open infrastructure for systematic, scoping, rapid and living
literature searches. It converts a review protocol into versioned contracts,
compiles portable queries into source-specific syntax, executes authorised
providers, preserves a complete audit trail, supports governed screening and
renders standards-aware reports.

## Users

- information specialists and health librarians;
- systematic and scoping review teams;
- clinical, policy and health-economic researchers;
- research-integrity, guideline and health-technology-assessment teams;
- developers embedding evidence search into repositories or applications;
- agents that require bounded, inspectable search tools rather than ungoverned
  browser automation.

## Core problem

Search intent is commonly fragmented across protocol prose, database interfaces,
spreadsheets, reference managers and custom scripts. Query translation loses
meaning; provider execution is hard to replay; screening decisions are detached
from eligibility versions; reporting is reconstructed after the fact; and each
repository reimplements network, provenance and audit code.

## Product promise

One review contract, one shared provider runtime, many interfaces, complete
receipts, explicit human authority and evidence-scaled claims.

## Product boundary

- Searchright owns review planning, source selection, query compilation, search
  execution, deduplication, screening and search reporting.
- `evidence-search-core` is product-neutral and is intended to serve both
  Searchright and Sourceright.
- Sourceright retains citation extraction, CSL normalisation, reference
  verification and citation-manager integrity.
- Searchright is infrastructure, not a journal-compliance certificate, clinical
  evidence oracle, licensed-database scraper or autonomous final reviewer.

## Success measures

1. A complete MVP review can be replayed from contracts and fixtures.
2. Every record is traceable to a source receipt and every material change to a
   hash-chained audit event.
3. Supported dialect translations pass a versioned conformance corpus and
   disclose all lossy transformations.
4. Sourceright removes its duplicate generic provider runtime after parity.
5. Independent search specialists can reproduce and critique generated outputs.
6. Public claims never exceed contracted, fixture-backed, live-proven or
   publicly-accepted evidence levels.
