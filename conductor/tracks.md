# Tracks

The [agent-led frontier programme](frontier-plan.md) is the active forward plan.
It links existing ownership to four milestones, operating modes, reusable
dependencies, research studies and versioned outputs. Historical evidence and
archived acceptance below remain historical; new planned modes are not current
runtime claims. Do not generate a second phase/task hierarchy for this programme.

Track status, implementation completeness and evidence level are separate.
`scaffolded` and `partially_implemented` prevent path presence from being
misrepresented as completed behaviour; `traceability.json` owns assertion-level claims.

Each track maps to `track-NN`; each phase maps to `track-NN-phase-M`; and
each top-level plan task maps to `track-NN-phase-M-task-TT`. The generated
native issue hierarchy and Project projection remain prepared-not-synced until
an explicit, approval-gated apply receipt exists.

| ID | Track | Horizon | Status | Implementation | Evidence | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| 03 | [Shared provider runtime and Sourceright extraction](tracks/03-shared-provider-runtime/spec.md) | foundation | integration_prepared | partially_implemented | source_verified | Centralise bounded provider execution, caching, receipts and policy while preparing reversible Sourceright adoption. |
| 04 | [Open provider connectors MVP](tracks/04-open-connectors-mvp/spec.md) | mvp | partially_implemented | partially_implemented | source_verified | Provide deterministic open-source adapters and opt-in live execution for major discovery sources. |
| 08 | [PRISMA, PRESS and reporting](tracks/08-prisma-press-reporting/spec.md) | mvp | source_implemented | source_implemented | source_verified | Render PRISMA flow/appendix outputs and PRESS/standards assessments directly from evidence without conflating reporting and conduct. |
| 11 | [Systematic-search agent skill and workflows](tracks/11-agentic-skill/spec.md) | mvp | partially_implemented | partially_implemented | compiler_verified | Package planning, PRESS, execution, deduplication, screening and reporting workflows with explicit human checkpoints. |
| 12 | [CiteWeft scholarly extraction and document evidence](tracks/12-citeweft-document-evidence/spec.md) | alpha | integration_prepared | partially_implemented | source_verified | Integrate CiteWeft through a pinned optional adapter while preserving spans, uncertainty, provenance and the no-canonical-write boundary. |
| 13 | [Integration passports, GitHub issue hierarchy and context spine](tracks/13-integration-passports-github-context/spec.md) | alpha | source_implemented | source_implemented | source_verified | Make repository boundaries, pinned compatibility, Conductor-to-GitHub hierarchy and agent context machine-readable and drift-checked. |
| 14 | [Sourceright migration and shared releases](tracks/14-sourceright-migration/spec.md) | alpha | integration_prepared | partially_implemented | source_verified | Adopt the shared runtime in Sourceright and coordinate compatible releases without losing citation semantics. |
| 15 | [GitHub estate audit and custom-code replacement](tracks/15-estate-migration/spec.md) | alpha | partially_implemented | partially_implemented | source_verified | Identify and safely replace duplicate systematic-search code across the GitHub estate. |
| 16 | [Maximal quality, context and security harness](tracks/16-quality-security-harness/spec.md) | alpha | source_implemented | source_implemented | source_verified | Provide compiler, test, coverage, mutation, supply-chain, workflow, secret, fuzz and release evidence gates. |

## Archived tracks

Archived tracks retain their canonical paths, requirement ownership and stable
GitHub projection keys. Archival never deletes or automatically archives remote items.

| ID | Track | Archived | Evidence |
| --- | --- | --- | --- |
| 00 | [Foundation, Conductor and toolchain](tracks/00-foundation-conductor-toolchain/spec.md) | 2026-11-09 | fixture_proven |
| 01 | [Contract catalogue and code generation](tracks/01-contract-catalog/spec.md) | 2026-08-29 | compiler_verified |
| 02 | [Portable query AST and dialect compilers](tracks/02-query-ast-dialects/spec.md) | 2026-11-09 | fixture_proven |
| 05 | [Execution, audit and local storage](tracks/05-execution-audit-store/spec.md) | 2026-08-13 | compiler_verified |
| 06 | [Imports, deduplication and study linkage](tracks/06-imports-dedup-linkage/spec.md) | 2026-08-30 | compiler_verified |
| 07 | [Governed screening workflow](tracks/07-screening-workflow/spec.md) | 2026-08-31 | fixture_proven |
| 09 | [CLI MVP](tracks/09-cli-mvp/spec.md) | 2026-08-29 | compiler_verified |
| 10 | [MCP stdio server MVP](tracks/10-mcp-mvp/spec.md) | 2026-08-30 | compiler_verified |
| 17 | [Benchmarks, search validation and human calibration](tracks/17-benchmarks-calibration/spec.md) | 2026-11-08 | fixture_proven |
| 18 | [Alpha release and distribution](tracks/18-alpha-release/spec.md) | 2026-11-08 | fixture_proven |
| 19 | [Registries and scholarly publication](tracks/19-registries-publication/spec.md) | 2026-11-08 | fixture_proven |
| 20 | [Grey literature, registers and supplementary discovery](tracks/20-broader-discovery/spec.md) | 2026-08-29 | compiler_verified |
| 21 | [Licensed BYO-access adapters](tracks/21-licensed-adapters/spec.md) | 2026-11-08 | fixture_proven |
| 22 | [Living reviews, amendments and update lineage](tracks/22-living-updates/spec.md) | 2026-11-08 | fixture_proven |
| 23 | [Active-learning prioritisation and calibrated agents](tracks/23-active-learning-agents/spec.md) | 2026-11-08 | fixture_proven |
| 24 | [WASI components, HTTP MCP and scalable execution](tracks/24-wasi-http-scale/spec.md) | 2026-11-08 | fixture_proven |
| 25 | [Provenance and research-object interoperability](tracks/25-provenance-research-objects/spec.md) | 2026-11-08 | fixture_proven |
| 26 | [Formal assurance and contract evolution](tracks/26-formal-assurance-contract-evolution/spec.md) | 2026-11-09 | fixture_proven |
| 27 | [Accessibility, internationalisation and usability](tracks/27-accessibility-internationalisation-usability/spec.md) | 2026-11-09 | fixture_proven |
| 28 | [Institutional governance, privacy and collaboration](tracks/28-institutional-governance-privacy-collaboration/spec.md) | 2026-11-09 | fixture_proven |
| 29 | [External methodological evaluation and sustainability](tracks/29-external-evaluation-sustainability/spec.md) | 2026-11-09 | fixture_proven |
| 30 | [Maturity gate and gap closure](tracks/30-maturity-gap-closure/spec.md) | 2026-11-09 | fixture_proven |
| 31 | [GitHub remote, nested issues and Project v2 control plane](tracks/31-github-control-plane/spec.md) | 2026-08-12 | live_proven |
| 32 | [Cross-repository contract release train and downstream canaries](tracks/32-cross-repository-release-train/spec.md) | 2026-11-09 | fixture_proven |
| 33 | [Operational observability, backup, restore and incident response](tracks/33-operational-reliability/spec.md) | 2026-11-09 | fixture_proven |
| 34 | [Authenticated remote MCP, tenancy and data residency](tracks/34-authenticated-remote-mcp/spec.md) | 2026-11-09 | fixture_proven |
| 35 | [Generated SDKs, fixture-backed documentation and adoption operations](tracks/35-sdk-docs-adoption/spec.md) | 2026-11-09 | fixture_proven |
| 36 | [Release-candidate rehearsal, staged pilots and ecosystem rehearsal](tracks/36-release-candidate-pilots/spec.md) | 2026-11-09 | fixture_proven |
| 37 | [Final mature 1.0 release and long-term operations](tracks/37-maturity-1-0/spec.md) | 2026-11-09 | fixture_proven |

## Evidence ladder

Contracted → source-verified → compiler-verified → fixture-proven →
opt-in live proven → externally validated → publicly accepted.

The canonical machine-readable mapping is `roadmap-coverage.json`; each track
contains `evidence.json`, `spec.md` and an evidence-aware `plan.md`.
