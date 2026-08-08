# Searchright

<!-- mcp-name: io.github.edithatogo/searchright -->

**Searchright** is contract-first Rust infrastructure for planning, executing,
screening, updating, auditing and reporting systematic, scoping, rapid and
living literature searches. One product facade exposes the same governed
operations through a Rust API, CLI, Model Context Protocol (MCP) server and
agent skill.

> **Current evidence level — source-verified alpha.** The repository contains a
> broad implementation and passes its deterministic static harness. This
> execution environment has not yet produced Rust compilation, a committed
> `Cargo.lock`, live-provider receipts, downstream cutover, registry acceptance
> or external methodological validation.

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

The product-neutral **`evidence-search-core`** owns the portable query AST,
dialect compiler, provider runtime, rate/retry/budget policy, cache/replay
interfaces, source receipts and hash-linked audit events. Searchright and
Sourceright are intended to consume that kernel without copying it.

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

The workspace currently contains **27 crates**:

```text
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
```

### Contracts, integration and standards

- **37** Draft 2020-12 JSON Schemas with 37 conforming examples.
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

## Agentic workflow

The `systematic-search` skill and role cards support planning, search design,
PRESS review, bounded execution, screening assistance, reporting and updates.
Agents may propose, translate, critique, prioritise and explain. They cannot
silently amend a protocol, release a consequential write, make a final exclusion
or promote a public readiness claim.

## Conductor and GitHub hierarchy

Conductor context is fully scaffolded with vision, strategy, architecture,
MoSCoW requirements and **31 ordered tracks** spanning foundation through a
version 1.0 maturity dossier. Every track has `spec.md`, a four-phase `plan.md`,
`metadata.json`, `evidence.json`, source work and higher-evidence blockers.

The canonical roadmap deterministically renders **156 GitHub work items**:

- one roadmap epic;
- 31 track issues;
- 124 phase subissues, four under each track.

Remote mutation is dry-run first and requires an explicit workflow input, a
protected write environment, issue-write permission, a clean Git tree and a
second environment opt-in. No remote Searchright repository or issues are
claimed by the local source artefacts.

The repository prepares host-aware Conductor installation scripts and records the
observed upstream version. A compatible Gemini, Antigravity or Claude host was
not present in this runtime, so installation is not falsely claimed.

## Maximal engineering harness

The source configures a 20-dimension assurance matrix covering schema and
semantic contracts, unit/integration/doctests, property and metamorphic tests,
coverage, mutation, fuzzing, Kani proofs, Loom concurrency exploration, Miri,
`cargo-careful`, SSRF/hostile-content controls, supply-chain review, clean-room
offline builds, reproducible archives and binaries, attestations, MCP transcripts
and human methodological evaluation.

The network-free aggregate harness currently runs 22 static gates. Configured
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
