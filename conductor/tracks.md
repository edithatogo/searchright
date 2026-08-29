# Tracks

Track status, implementation completeness and evidence level are separate.
`scaffolded` and `partially_implemented` prevent path presence from being
misrepresented as completed behaviour; `traceability.json` owns assertion-level claims.

Each track maps to `track-NN`; each phase maps to `track-NN-phase-M`; and
each top-level plan task maps to `track-NN-phase-M-task-TT`. The generated
native issue hierarchy and Project projection remain prepared-not-synced until
an explicit, approval-gated apply receipt exists.

| ID | Track | Horizon | Status | Implementation | Evidence | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| 00 | [Foundation, Conductor and toolchain](tracks/00-foundation-conductor-toolchain/spec.md) | foundation | external_evidence_required | external_evidence_required | source_verified | Establish the Git repository, Conductor context, pinned toolchain, standards inheritance and reproducible bootstrap. |
| 01 | [Contract catalogue and code generation](tracks/01-contract-catalog/spec.md) | foundation | partially_implemented | partially_implemented | source_verified | Maintain versioned schemas, examples, standards packs and Rust wire types from one catalogue. |
| 02 | [Portable query AST and dialect compilers](tracks/02-query-ast-dialects/spec.md) | foundation | partially_implemented | partially_implemented | source_verified | Deliver deterministic, reviewable query translation with explicit fidelity and loss warnings. |
| 03 | [Shared provider runtime and Sourceright extraction](tracks/03-shared-provider-runtime/spec.md) | foundation | integration_prepared | partially_implemented | source_verified | Centralise bounded provider execution, caching, receipts and policy while preparing reversible Sourceright adoption. |
| 04 | [Open provider connectors MVP](tracks/04-open-connectors-mvp/spec.md) | mvp | partially_implemented | partially_implemented | source_verified | Provide deterministic open-source adapters and opt-in live execution for major discovery sources. |
| 06 | [Imports, deduplication and study linkage](tracks/06-imports-dedup-linkage/spec.md) | mvp | source_implemented | source_implemented | compiler_verified | Import/export common bibliographic formats, propose conservative duplicate clusters and distinguish records, reports and studies. |
| 07 | [Governed screening workflow](tracks/07-screening-workflow/spec.md) | mvp | partially_implemented | partially_implemented | source_verified | Support independent title/abstract and full-text decisions, conflicts, adjudication, roles and conservative agent authority. |
| 08 | [PRISMA, PRESS and reporting](tracks/08-prisma-press-reporting/spec.md) | mvp | partially_implemented | partially_implemented | source_verified | Render PRISMA flow/appendix outputs and PRESS/standards assessments directly from evidence without conflating reporting and conduct. |
| 09 | [CLI MVP](tracks/09-cli-mvp/spec.md) | mvp | source_implemented | source_implemented | compiler_verified | Expose stable scriptable operations through the shared application facade. |
| 10 | [MCP stdio server MVP](tracks/10-mcp-mvp/spec.md) | mvp | partially_implemented | partially_implemented | compiler_verified | Expose typed, authority-annotated MCP tools over the same facade with stable structured outputs. |
| 11 | [Systematic-search agent skill and workflows](tracks/11-agentic-skill/spec.md) | mvp | partially_implemented | partially_implemented | source_verified | Package planning, PRESS, execution, deduplication, screening and reporting workflows with explicit human checkpoints. |
| 12 | [CiteWeft scholarly extraction and document evidence](tracks/12-citeweft-document-evidence/spec.md) | alpha | integration_prepared | partially_implemented | source_verified | Integrate CiteWeft through a pinned optional adapter while preserving spans, uncertainty, provenance and the no-canonical-write boundary. |
| 13 | [Integration passports, GitHub issue hierarchy and context spine](tracks/13-integration-passports-github-context/spec.md) | alpha | source_implemented | source_implemented | source_verified | Make repository boundaries, pinned compatibility, Conductor-to-GitHub hierarchy and agent context machine-readable and drift-checked. |
| 14 | [Sourceright migration and shared releases](tracks/14-sourceright-migration/spec.md) | alpha | integration_prepared | partially_implemented | source_verified | Adopt the shared runtime in Sourceright and coordinate compatible releases without losing citation semantics. |
| 15 | [GitHub estate audit and custom-code replacement](tracks/15-estate-migration/spec.md) | alpha | partially_implemented | partially_implemented | source_verified | Identify and safely replace duplicate systematic-search code across the GitHub estate. |
| 16 | [Maximal quality, context and security harness](tracks/16-quality-security-harness/spec.md) | alpha | partially_implemented | partially_implemented | source_verified | Provide compiler, test, coverage, mutation, supply-chain, workflow, secret, fuzz and release evidence gates. |
| 17 | [Benchmarks, search validation and human calibration](tracks/17-benchmarks-calibration/spec.md) | alpha | scaffolded | partially_implemented | source_verified | Evaluate translation, retrieval, deduplication and prioritisation with leakage controls and human calibration. |
| 18 | [Alpha release and distribution](tracks/18-alpha-release/spec.md) | alpha | release_prepared | scaffolded | source_verified | Produce locked, signed, attestable cross-platform technical-preview releases with reproducible source archives. |
| 19 | [Registries and scholarly publication](tracks/19-registries-publication/spec.md) | alpha | submission_prepared | scaffolded | source_verified | Prepare and submit truthful registry packets and a software paper only after a verified release. |
| 20 | [Grey literature, registers and supplementary discovery](tracks/20-broader-discovery/spec.md) | beta | partially_implemented | partially_implemented | source_verified | Represent bounded trial-register, repository, website, citation-chaining and contact-search methods with explicit limits. |
| 21 | [Licensed BYO-access adapters](tracks/21-licensed-adapters/spec.md) | beta | scaffolded | scaffolded | source_verified | Provide credential-free request planning and explicit licence/capability profiles for Embase, Scopus and Web of Science. |
| 22 | [Living reviews, amendments and update lineage](tracks/22-living-updates/spec.md) | beta | partially_implemented | partially_implemented | source_verified | Make updates, protocol changes, prior-run deduplication and cadence explicit and immutable. |
| 23 | [Active-learning prioritisation and calibrated agents](tracks/23-active-learning-agents/spec.md) | beta | scaffolded | scaffolded | source_verified | Provide transparent advisory ranking with uncertainty and no default autonomous exclusion. |
| 24 | [WASI components, HTTP MCP and scalable execution](tracks/24-wasi-http-scale/spec.md) | beta | partially_implemented | partially_implemented | source_verified | Define sandboxed provider components, integrity/capability checks and future remote MCP boundaries. |
| 25 | [Provenance and research-object interoperability](tracks/25-provenance-research-objects/spec.md) | mature | partially_implemented | partially_implemented | source_verified | Export immutable review lineage as deterministic RO-Crate and W3C PROV research objects. |
| 26 | [Formal assurance and contract evolution](tracks/26-formal-assurance-contract-evolution/spec.md) | mature | partially_implemented | partially_implemented | source_verified | Model workflow/authority invariants and govern compatibility, migration, deprecation and rollback. |
| 27 | [Accessibility, internationalisation and usability](tracks/27-accessibility-internationalisation-usability/spec.md) | mature | scaffolded | scaffolded | source_verified | Provide stable accessible diagnostics and locale-neutral contracts, then validate them with users and assistive technology. |
| 28 | [Institutional governance, privacy and collaboration](tracks/28-institutional-governance-privacy-collaboration/spec.md) | mature | partially_implemented | partially_implemented | source_verified | Evaluate data handling and least-privilege collaboration policy before sensitive or cross-institution operations. |
| 29 | [External methodological evaluation and sustainability](tracks/29-external-evaluation-sustainability/spec.md) | mature | external_evidence_required | external_evidence_required | source_verified | Complete independent methodological/usability evaluation and establish durable governance, publication and succession evidence. |
| 30 | [Maturity gate and gap closure](tracks/30-maturity-gap-closure/spec.md) | mature | partially_implemented | partially_implemented | source_verified | Maintain one evidence-scaled gap register and an executable hardened/polished launch-preparation dependency graph while blocking premature release claims. |
| 32 | [Cross-repository contract release train and downstream canaries](tracks/32-cross-repository-release-train/spec.md) | mature | integration_prepared | partially_implemented | source_verified | Coordinate CiteWeft, Searchright/shared core and Sourceright compatibility without coupling repositories or automatically promoting revisions. |
| 33 | [Operational observability, backup, restore and incident response](tracks/33-operational-reliability/spec.md) | mature | partially_implemented | partially_implemented | source_verified | Provide default-private health, telemetry, backup, restore, resilience and incident contracts for local and hosted deployments. |
| 34 | [Authenticated remote MCP, tenancy and data residency](tracks/34-authenticated-remote-mcp/spec.md) | mature | integration_prepared | partially_implemented | source_verified | Add default-deny principal, scope, tenant and region policy before any hosted Streamable HTTP MCP deployment. |
| 35 | [Generated SDKs, fixture-backed documentation and adoption operations](tracks/35-sdk-docs-adoption/spec.md) | mature | scaffolded | scaffolded | source_verified | Expose thin contract-generated clients and evidence-scaled tutorials without duplicating the Rust domain core. |
| 36 | [Release-candidate rehearsal, staged pilots and ecosystem rehearsal](tracks/36-release-candidate-pilots/spec.md) | mature | release_prepared | scaffolded | source_verified | Exercise the complete candidate in clean-room builds, downstream canaries, bounded pilots, rollback and registry submission rehearsals. |
| 37 | [Final mature 1.0 release and long-term operations](tracks/37-maturity-1-0/spec.md) | mature | external_evidence_required | external_evidence_required | source_verified | Release version 1.0 only after every maturity domain has current evidence, no critical blocker, accountable approval and support/rollback readiness. |

## Archived tracks

Archived tracks retain their canonical paths, requirement ownership and stable
GitHub projection keys. Archival never deletes or automatically archives remote items.

| ID | Track | Archived | Evidence |
| --- | --- | --- | --- |
| 05 | [Execution, audit and local storage](tracks/05-execution-audit-store/spec.md) | 2026-08-13 | compiler_verified |
| 31 | [GitHub remote, nested issues and Project v2 control plane](tracks/31-github-control-plane/spec.md) | 2026-08-12 | live_proven |

## Evidence ladder

Contracted → source-verified → compiler-verified → fixture-proven →
opt-in live proven → externally validated → publicly accepted.

The canonical machine-readable mapping is `roadmap-coverage.json`; each track
contains `evidence.json`, `spec.md` and an evidence-aware `plan.md`.
