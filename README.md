# Searchright

<!-- mcp-name: io.github.edithatogo/searchright -->

**Searchright** is contract-first Rust infrastructure for planning, executing,
screening, updating, auditing and reporting systematic, scoping, rapid and
living literature searches. One product facade exposes the same governed
operations through a Rust API, CLI, Model Context Protocol (MCP) server and
agent skill.

> **Current evidence level — source-verified alpha, assertion-rebaselined.**
> The repository has a broad source surface and a passing network-free harness,
> but roadmap scope is no longer inferred from file presence. Each track is
> decomposed into acceptance assertions classified as contracted, scaffolded or
> partially implemented unless stronger evidence exists. This environment has
> not produced Rust compilation, a committed `Cargo.lock`, live-provider
> receipts, downstream cutover, registry acceptance or external methodological
> validation.

## Why Searchright

Systematic-search intent is usually fragmented across protocol prose, database
interfaces, spreadsheets, reference managers and one-off scripts. Searchright
turns that intent into versioned, inspectable contracts and retains evidence from
question formulation through search translation, retrieval, deduplication,
screening, study linkage, amendments and reporting.

The differentiator is not a long connector list. It is a reusable evidence-search
kernel with explicit authority, translation-loss reporting, bounded execution,
immutable lineage and claims that cannot outrun their evidence.

## Architecture boundary

The repository estate now has three distinct scholarly domains:

- **CiteWeft** owns backend-neutral scholarly-document extraction evidence,
  including source spans, uncertainty, diagnostics and optional backend routing.
  CiteWeft is GROBID-inspired; it is not represented as a Rust port, fork,
  reimplementation or compatibility replacement for GROBID.
- **Searchright** owns review planning, source selection, search validation,
  execution orchestration, import/export, deduplication, record–report–study
  linkage, screening, living updates and standards-aware reporting.
- **Sourceright** owns canonical CSL, provider-backed citation verification,
  citation reconciliation and reference-integrity workflows.

The product-neutral **`evidence-search-contracts`** crate owns portable query,
provider, receipt, record and audit contracts. **`evidence-search-core`** sits
above it and owns dialect compilation, provider execution, rate/retry/budget
policy, cache/replay interfaces and audit-ledger behaviour. Searchright and
Sourceright are intended to consume those neutral layers without importing
review-specific contracts or copying runtime code.

Only the non-publishable leaf crate `searchright-citeweft` depends on CiteWeft.
The Searchright facade, shared kernel, CLI and MCP server remain extraction-
backend neutral. CLI and MCP are adapters over `SearchrightEngine`; they do not
maintain separate methodological logic.

## Implemented source surface

### Methodological contracts

- Review questions and frameworks including PICO, PCC, SPIDER and PEO.
- Explicit eligibility criteria, governance, protocol registration and
  amendments.
- Portable Boolean query AST, source-specific dialect compilation, fidelity
  classes, loss codes and mandatory review markers.
- Search runs, source receipts, provider pages, audit events and execution
  envelopes.
- Records, reports, studies, retrieval attempts and evidence-bearing linkage.
- Dual-phase screening, reviewer authority, conflicts and full-text exclusion
  reasons.
- PRISMA 2020 flow arithmetic, PRISMA-S ledgers, PRESS findings and seed-set
  validation.
- Living-review lineage, ranking calibration, supplementary discovery,
  interchange, diagnostics, governance, benchmark and provider-component
  contracts.
- Neutral document evidence and consumer-driven cross-repository integration
  contracts.

### Executable Rust components

The workspace currently contains **30 internal crates**. All packages are non-publishable by default; only three neutral/SDK candidates may be promoted after compiler, SemVer, consumer and supply-chain evidence:

```text
crates/evidence-search-contracts/      Neutral query/provider/receipt/audit contracts
crates/evidence-search-core/           Shared query/provider/audit kernel
crates/searchright-contracts/          Canonical Rust contract types
crates/searchright-citeweft/           Optional one-way CiteWeft evidence adapter
crates/searchright-connectors/         Open-provider fixtures and opt-in adapters
crates/searchright-store/              Single-writer audit and snapshot store
crates/searchright-dedup/              Conservative duplicate clustering
crates/searchright-study/              Record–report–study graph operations
crates/searchright-screening/          Human-governed screening and reconciliation
crates/searchright-validation/         PRESS, seed recall and translation gates
crates/searchright-prisma/             PRISMA flow and PRISMA-S outputs
crates/searchright-interchange/        Searchright JSON, JSONL, CSL, RIS, NBIB, CSV
crates/searchright-living/             Immutable update lineage and record diffs
crates/searchright-provenance/         RO-Crate 1.3 and W3C PROV-style exports
crates/searchright-ranking/            Explainable advisory ranking/calibration
crates/searchright-discovery/          Bounded citation/grey-literature discovery
crates/searchright-policy/             Host, capability and hostile-content policy
crates/searchright-governance/         Institutional data-handling decisions
crates/searchright-assurance/          Finite lifecycle and formal authority model
crates/searchright-plugin-sdk/         WASI component manifest verification
crates/searchright-licensed/           Redacted BYO-access request planning
crates/searchright-bench/              Retrieval/ranking/dedup regression metrics
crates/searchright-diagnostics/        Accessible stable diagnostics
crates/searchright-sourceright-compat/ Sourceright parity/cutover helpers
crates/searchright/                    Shared application facade
crates/searchright-cli/                `searchright` CLI
crates/searchright-mcp/                `searchright-mcp` stdio server
crates/searchright-agent/              Governed agent workflow
crates/searchright-access/             Authentication, tenancy and authority policy
crates/searchright-ops/                Health, telemetry, backup and incident contracts
```

### Contracts, integration and standards

- **52** Draft 2020-12 JSON Schemas with 52 conforming examples.
- A machine-readable schema catalogue and a **31-operation**
  CLI–MCP–facade interface catalogue.
- OpenAPI and WIT boundary contracts.
- Eight exact-revision integration passports and eight matching
  producer–consumer interactions.
- A read-only integration-drift workflow that cannot update revisions or claims.
- Versioned standards packs for PRISMA 2020, PRISMA-S, PRISMA-ScR, PRISMA-LSR,
  PRISMA-P, PRESS, Cochrane, MECIR, JBI and Campbell guidance.
- Licensed-source profiles for Embase, Scopus and Web of Science that contain no
  credentials and do not imply bundled access.

Cross-repository integration uses exact pins, neutral schemas, golden fixtures,
consumer-driven contracts and optional leaf adapters. It deliberately avoids Git
submodules, copied implementation code and automatic dependency promotion.

## Implementation truth model

Every one of the 38 Conductor tracks now has assertion-level traceability. The
current 198 acceptance assertions include 65 with explicit symbol/test/gate
mappings; the remainder retain conservative track-level status. A track cannot
be promoted merely because a named file exists, an issue is closed or a remote
Project field changes.

The repository distinguishes:

- **contracted** — requirement exists;
- **scaffolded** — interface or placeholder exists, without behaviour claim;
- **partially implemented** — some behaviour is mapped, while scope remains open;
- **source implemented** — every assertion maps to symbols and deterministic
  tests, while compiler/runtime evidence remains separate;
- **higher evidence** — compiler, fixture, live, downstream, external and public
  acceptance receipts, each independently earned.

Additional local controls now freeze the exact JSON Schema, WIT, OpenAPI and MCP
surface; prohibit publishing any workspace crate by default; pin rights-clear
provider response baselines; distinguish canonical upstreams from personal
forks; and reject unclear-licence integrations from redistributed code/content.

A deterministic `.srpack` review bundle can package and verify review artefacts,
while a derived review-state reducer binds to an externally verified audit head
and rejects non-human attempts to exercise final screening authority. These are
local source/fixture proofs, not claims of a completed systematic review.

## Agentic workflow

The `systematic-search` skill and role cards support planning, search design,
PRESS review, bounded execution, screening assistance, reporting and updates.
Agents may propose, translate, critique, prioritise and explain. They cannot
silently amend a protocol, release a consequential write, make a final exclusion
or promote a public readiness claim.

## Conductor and GitHub hierarchy

Conductor context is fully scaffolded with vision, strategy, architecture,
MoSCoW requirements and **38 ordered tracks** spanning foundation through a
version 1.0 maturity dossier. Every track has `spec.md`, a four-phase `plan.md`,
`metadata.json`, `evidence.json`, source work and higher-evidence blockers.

The canonical roadmap deterministically renders **564 GitHub work items** in a four-level hierarchy:

- one roadmap epic;
- 38 track issues;
- 152 phase subissues, four under each track;
- 373 task subissues corresponding to every top-level Conductor task.

Remote mutation is dry-run first and requires an explicit workflow input, a
protected write environment, issue/project scopes, a clean Git tree and a
second environment opt-in. A declarative GitHub Project v2 manifest owns 12
custom fields and six views, including a separate implementation-gap view; an additive synchroniser creates or updates the
Project and places all 564 issue nodes into it without deleting or archiving
remote work. A one-command bootstrap controller can create the remote repository,
apply settings and the main-branch ruleset, synchronise the issue hierarchy, and
create/populate the Project. No remote Searchright repository, issue or Project
is claimed by the local source artefacts until an observed apply receipt exists.

The repository prepares host-aware Conductor installation scripts and records the
observed upstream version. A compatible Gemini, Antigravity or Claude host was
not present in this runtime, so installation is not falsely claimed.

## Operational and maturity control plane

Searchright now includes source contracts and policy crates for authenticated
remote MCP, tenant-scoped authorisation, bounded long-running tasks, component
health, opt-in telemetry, backup manifests, incident records and restore/rollback
planning. A cross-repository release train orders CiteWeft → Searchright →
Sourceright promotion through explicit consumer and downstream-canary evidence.
A release-candidate rehearsal and three pilot profiles are prepared; neither is
represented as executed. The final maturity track records a version 1.0 decision
only when every required evidence domain passes.

## Maximal engineering harness

The source configures a 28-dimension assurance matrix covering schema and
semantic contracts, unit/integration/doctests, property and metamorphic tests,
coverage, mutation, fuzzing, Kani proofs, Loom concurrency exploration, Miri,
`cargo-careful`, SSRF/hostile-content controls, supply-chain review, clean-room
offline builds, reproducible archives and binaries, attestations, MCP transcripts
and human methodological evaluation.

The network-free aggregate harness currently runs **44 static gates**, including assertion traceability, package-surface policy, contract-surface freezing, provider baselines, a rights-clear end-to-end contract reference slice, review-bundle and review-state self-tests, benchmark leakage controls, licence firewalls, portfolio consistency and exact CI/developer-tool pin parity. Configured
compiler, runtime, live and external gates do not become passing evidence until
their receipts are observed.

## Intended local use

Once Rust 1.97.1 and dependency access are available:

```bash
./scripts/bootstrap.sh
cargo generate-lockfile
./scripts/verify.sh

cargo run -p searchright-cli -- validate-plan contracts/examples/review-plan.yaml
cargo run -p searchright-cli -- compile contracts/examples/search-strategy.yaml --dialect pubmed
cargo run -p searchright-cli -- prisma contracts/examples/prisma-flow.json --format mermaid
cargo run -p searchright-mcp
```

The committed lockfile is deliberately a release gate. Internal path dependencies
carry exact workspace versions; unpublished Git-only adapters remain
`publish = false` until their registry dependency exists.

## Evidence and claims

Searchright uses this evidence ladder:

1. contracted;
2. source-verified;
3. compiler-verified;
4. fixture-proven;
5. opt-in live proven;
6. externally validated;
7. publicly accepted.

Prepared registry metadata is not acceptance. A provider fixture is not live
proof. Internal benchmarks are not methodological validation. A migration map is
not downstream cutover. A consumer-contract declaration is not downstream
compatibility.

## Safety and non-goals

Searchright does not certify comprehensiveness, clinical truth, legal compliance
or journal acceptance. It does not circumvent licensed platforms, execute
provider text as instructions, permit agent-only final exclusions or claim
one-click autonomous systematic reviews.

## Licence

Dual licensed under the MIT or Apache-2.0 licence at your option.
