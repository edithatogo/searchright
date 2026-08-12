# GitHub estate integration and migration

## Architectural rule

Searchright must replace duplicated systematic-search behaviour without becoming
a monorepo or absorbing unrelated product responsibilities. Integration is by
versioned contracts, exact pins, consumer fixtures, thin adapters and explicit
rollback. Git submodules and copied implementation code are not the default.

The earlier name `edithatogo/sourcerightlibrary` was not found. The active
bibliographic product is `edithatogo/sourceright`; CiteWeft is a separate Rust
extraction repository.

## Shared ownership boundary

- `evidence-search-contracts`: neutral query, provider, record, receipt and audit
  wire types.
- `evidence-search-core`: neutral compilation, bounded provider execution,
  replay/cache and audit-ledger behaviour.
- Searchright: review planning, search orchestration, deduplication,
  report/study linkage, screening, living updates and reporting.
- CiteWeft: document extraction evidence, spans, uncertainty and diagnostics.
- Sourceright: CSL canonicalisation, citation reconciliation and bibliographic
  verification.

The neutral layers remain internal and non-publishable until Searchright and
Sourceright both pass compiler and consumer-contract evidence.

## Active integration classifications

| Repository | Origin/licence posture | Intended integration |
| --- | --- | --- |
| `sourceright` | Original; MIT OR Apache-2.0 | Adopt the neutral contracts/core behind a feature flag; dual-run all parity dimensions before cutover. |
| `citeweft` | Original; MIT OR Apache-2.0 | One-way document-evidence producer; protect `main`, add signed releases and backend conformance fixtures. |
| `repository-standards` | Original; Apache-2.0 | Add a `rust-research-infrastructure` profile and reusable evidence workflows. |
| `standards_check` | Original, licence not asserted in the observed repository | Provenance/reference source only until a rights inventory permits signed standard packs. |
| `PRISMA.jl` | Fork of `cecoeco/PRISMA.jl`; upstream MIT | Comparator and migration source, not a second canonical PRISMA engine. |
| `synergy-dataset` | Fork of `asreview/synergy-dataset`; upstream CC0 | Pin canonical upstream release/DOI for benchmark runs; local fork is mirror/patch carrier only. |
| `api-standards` | Fork of Te Whatu Ora repository; GitHub licence `NOASSERTION` | External policy reference, not redistributed source, until reuse rights are clear. |
| `academic-research-skills` | Fork with unclear observed licence and substantial upstream drift | Reference-only orchestration integration until licence/upstream decisions are resolved. |

Every passport records canonical upstream, local-fork role, code/content/model
licences, redistribution policy, drift policy and exact revision. The licence
firewall is a static governance control, not legal advice.

## Concrete custom-code migrations

The local estate scan and companion packets identify the following immediate
migrations:

| Repository | Existing duplication | Target owner |
| --- | --- | --- |
| `UOGTO` | Direct OpenAlex, Europe PMC, arXiv and Crossref clients; insecure HTTP arXiv URL; hard-coded limits | Searchright connectors and execution receipts |
| `UOGTO` | First-two-record snowballing, silent errors and title-only deduplication | Searchright supplementary discovery, deduplication and study linkage |
| `voiage` | Direct Crossref DOI validation/enrichment and regex BibTeX writeback | Sourceright verification and preview/apply writeback |
| `scholarly-publishing-agents` | Procedural search, screening, PRISMA and citation logic in prompts | Thin Searchright and Sourceright MCP/skill orchestration |
| `academic-research-skills` | Embedded review-method instructions and contracts | Versioned skill wrapper over Searchright, subject to upstream/licence resolution |
| other research repos | Direct provider endpoints, local PRISMA arithmetic, bespoke dedup/screening | Searchright or a documented exception after regression fixture capture |

Eleven machine-readable change packets under
`migration/companion-repositories/` specify observed revisions, target paths,
owners, required evidence and non-automatic application. No companion repository
was changed in this local pass.

## Safe migration protocol

1. Refresh the producer repository and exact revision.
2. Capture legacy success and failure outputs as rights-clear regression
   fixtures.
3. Map every legacy behaviour to Searchright, Sourceright, CiteWeft or an explicit
   retention decision.
4. Add a compatibility adapter and a reversible feature flag.
5. Run old/new paths over success, empty, malformed, pagination, retry,
   cancellation and redaction cases.
6. Compare identifiers, fields, ordering, errors, cache semantics, raw digests
   and receipts.
7. Record and approve intended differences.
8. Run the downstream repository's native compiler, quality, security and
   release gates.
9. Switch the default while retaining rollback for at least one compatible
   release.
10. Delete superseded code only after the consumer and rollback receipts pass.

## Portfolio and release governance

The 583-item Searchright delivery Project remains separate from the strategic
Evidence Infrastructure Portfolio. The portfolio contains only cross-repository
contracts, migration, licence and release-train items. The ecosystem lock fixes
observed component identities and local contract/package digests; it does not
update pins automatically or claim downstream compatibility.

## Remaining improvements in companion repositories

- **CiteWeft:** branch protection, personal/signed release identity, JSON Schema
  interchange, rights-clear extraction benchmarks, backend conformance kit and
  explicit text-layer/OCR/inference provenance.
- **Sourceright:** neutral-core feature flag, public-API/SemVer gates, old/new
  runtime parity, one-release rollback and deletion receipt.
- **repository-standards:** dedicated evidence-infrastructure profile, canary
  rollout and immutable reusable workflow versions.
- **standards_check:** stable item URIs, source checksums, effective dates,
  supersession and rights-aware signed pack generation.
- **forks:** automated upstream-drift reports, explicit patch inventory and
  archive/mirror decisions where no local delta is intended.
- **research agents:** capability manifests, tool allowlists, contract versions,
  prompt-injection boundaries and no alternate provider/PRISMA/final-screening
  implementation.
