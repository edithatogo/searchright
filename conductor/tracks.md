# Tracks

Track status is evidence-scaled. `source_implemented_unverified` means the
contracts and source exist and pass the static harness; it does not mean Rust
compiled, providers ran, users evaluated the workflow or a registry accepted it.

Each track maps to a GitHub issue key `track-NN`; each numbered plan phase maps
to a native GitHub subissue key `track-NN-phase-M`. The generated hierarchy is
stored under `conductor/github/` and remains prepared-not-synced until an
explicit, approval-gated apply run succeeds.

| ID | Track | Horizon | Status | Evidence | Outcome |
| --- | --- | --- | --- | --- | --- |
| 00 | [Foundation, Conductor and toolchain](tracks/00-foundation-conductor-toolchain/spec.md) | foundation | source_implemented | source_verified | Establish the Git repository, Conductor context, pinned toolchain, standards inheritance and reproducible bootstrap. |
| 01 | [Contract catalogue and code generation](tracks/01-contract-catalog/spec.md) | foundation | source_implemented | source_verified | Maintain versioned schemas, examples, standards packs and Rust wire types from one catalogue. |
| 02 | [Portable query AST and dialect compilers](tracks/02-query-ast-dialects/spec.md) | foundation | source_implemented_unverified | source_verified | Deliver deterministic, reviewable query translation with explicit fidelity and loss warnings. |
| 03 | [Shared provider runtime and Sourceright extraction](tracks/03-shared-provider-runtime/spec.md) | foundation | integration_prepared | source_verified | Centralise bounded provider execution, caching, receipts and policy while preparing reversible Sourceright adoption. |
| 04 | [Open provider connectors MVP](tracks/04-open-connectors-mvp/spec.md) | mvp | source_implemented_unverified | source_verified | Provide deterministic open-source adapters and opt-in live execution for major discovery sources. |
| 05 | [Execution, audit and local storage](tracks/05-execution-audit-store/spec.md) | mvp | source_implemented_unverified | source_verified | Create replayable runs, content-addressed receipts, tamper-evident events and crash-conscious local state. |
| 06 | [Imports, deduplication and study linkage](tracks/06-imports-dedup-linkage/spec.md) | mvp | source_implemented_unverified | source_verified | Import/export common bibliographic formats, propose conservative duplicate clusters and distinguish records, reports and studies. |
| 07 | [Governed screening workflow](tracks/07-screening-workflow/spec.md) | mvp | source_implemented_unverified | source_verified | Support independent title/abstract and full-text decisions, conflicts, adjudication, roles and conservative agent authority. |
| 08 | [PRISMA, PRESS and reporting](tracks/08-prisma-press-reporting/spec.md) | mvp | source_implemented_unverified | source_verified | Render PRISMA flow/appendix outputs and PRESS/standards assessments directly from evidence without conflating reporting and conduct. |
| 09 | [CLI MVP](tracks/09-cli-mvp/spec.md) | mvp | source_implemented_unverified | source_verified | Expose stable scriptable operations through the shared application facade. |
| 10 | [MCP stdio server MVP](tracks/10-mcp-mvp/spec.md) | mvp | source_implemented_unverified | source_verified | Expose typed, authority-annotated MCP tools over the same facade with stable structured outputs. |
| 11 | [Systematic-search agent skill and workflows](tracks/11-agentic-skill/spec.md) | mvp | source_implemented_unverified | source_verified | Package planning, PRESS, execution, deduplication, screening and reporting workflows with explicit human checkpoints. |
| 12 | [CiteWeft scholarly extraction and document evidence](tracks/12-citeweft-document-evidence/spec.md) | alpha | integration_prepared | source_verified | Integrate CiteWeft through a pinned optional adapter while preserving spans, uncertainty, provenance and the no-canonical-write boundary. |
| 13 | [Integration passports, GitHub issue hierarchy and context spine](tracks/13-integration-passports-github-context/spec.md) | alpha | integration_prepared | source_verified | Make repository boundaries, pinned compatibility, Conductor-to-GitHub hierarchy and agent context machine-readable and drift-checked. |
| 14 | [Sourceright migration and shared releases](tracks/14-sourceright-migration/spec.md) | alpha | integration_prepared | source_verified | Adopt the shared runtime in Sourceright and coordinate compatible releases without losing citation semantics. |
| 15 | [GitHub estate audit and custom-code replacement](tracks/15-estate-migration/spec.md) | alpha | integration_prepared | source_verified | Identify and safely replace duplicate systematic-search code across the GitHub estate. |
| 16 | [Maximal quality, context and security harness](tracks/16-quality-security-harness/spec.md) | alpha | source_implemented_unverified | source_verified | Provide compiler, test, coverage, mutation, supply-chain, workflow, secret, fuzz and release evidence gates. |
| 17 | [Benchmarks, search validation and human calibration](tracks/17-benchmarks-calibration/spec.md) | alpha | source_implemented_unverified | source_verified | Evaluate translation, retrieval, deduplication and prioritisation with leakage controls and human calibration. |
| 18 | [Alpha release and distribution](tracks/18-alpha-release/spec.md) | alpha | release_prepared | source_verified | Produce locked, signed, attestable cross-platform technical-preview releases with reproducible source archives. |
| 19 | [Registries and scholarly publication](tracks/19-registries-publication/spec.md) | alpha | submission_prepared | source_verified | Prepare and submit truthful registry packets and a software paper only after a verified release. |
| 20 | [Grey literature, registers and supplementary discovery](tracks/20-broader-discovery/spec.md) | beta | source_implemented_unverified | source_verified | Represent bounded trial-register, repository, website, citation-chaining and contact-search methods with explicit limits. |
| 21 | [Licensed BYO-access adapters](tracks/21-licensed-adapters/spec.md) | beta | source_implemented_unverified | source_verified | Provide credential-free request planning and explicit licence/capability profiles for Embase, Scopus and Web of Science. |
| 22 | [Living reviews, amendments and update lineage](tracks/22-living-updates/spec.md) | beta | source_implemented_unverified | source_verified | Make updates, protocol changes, prior-run deduplication and cadence explicit and immutable. |
| 23 | [Active-learning prioritisation and calibrated agents](tracks/23-active-learning-agents/spec.md) | beta | source_implemented_unverified | source_verified | Provide transparent advisory ranking with uncertainty and no default autonomous exclusion. |
| 24 | [WASI components, HTTP MCP and scalable execution](tracks/24-wasi-http-scale/spec.md) | beta | source_implemented_unverified | source_verified | Define sandboxed provider components, integrity/capability checks and future remote MCP boundaries. |
| 25 | [Provenance and research-object interoperability](tracks/25-provenance-research-objects/spec.md) | mature | source_implemented_unverified | source_verified | Export immutable review lineage as deterministic RO-Crate and W3C PROV research objects. |
| 26 | [Formal assurance and contract evolution](tracks/26-formal-assurance-contract-evolution/spec.md) | mature | source_implemented_unverified | source_verified | Model workflow/authority invariants and govern compatibility, migration, deprecation and rollback. |
| 27 | [Accessibility, internationalisation and usability](tracks/27-accessibility-internationalisation-usability/spec.md) | mature | source_implemented_unverified | source_verified | Provide stable accessible diagnostics and locale-neutral contracts, then validate them with users and assistive technology. |
| 28 | [Institutional governance, privacy and collaboration](tracks/28-institutional-governance-privacy-collaboration/spec.md) | mature | source_implemented_unverified | source_verified | Evaluate data handling and least-privilege collaboration policy before sensitive or cross-institution operations. |
| 29 | [External methodological evaluation and sustainability](tracks/29-external-evaluation-sustainability/spec.md) | mature | external_evidence_required | source_verified | Complete independent methodological/usability evaluation and establish durable governance, publication and succession evidence. |
| 30 | [Mature 1.0 product, community and evaluation](tracks/30-maturity-1-0/spec.md) | mature | external_evidence_required | source_verified | Reach a stable externally evaluated 1.0 infrastructure product only after every evidence domain passes. |

## Evidence ladder

Contracted → source-verified → compiler-verified → fixture-proven →
opt-in live proven → externally validated → publicly accepted.

The canonical machine-readable mapping is `roadmap-coverage.json`; each track
contains `evidence.json`, `spec.md` and an evidence-aware `plan.md`.
