# Strategy

## Strategic thesis

Searchright's defensible advantage is not the number of database integrations.
It is a portable evidence-search contract, an audit-preserving runtime and a
shared application facade that downstream repositories, interfaces and agents
reuse without reimplementing policy, authority or provenance.

The compounding asset is the evidence graph: questions, strategy translations,
provider receipts, records, reports, studies, screening decisions, amendments
and reporting artefacts remain linked and reproducible across review lifecycles.

## Strategic architecture

The product is organised into three rates of change:

1. **Stable auditable kernel** — versioned contracts, portable query AST,
   deterministic compilation, execution policy, receipts, audit events and
   immutable identifiers.
2. **Product services** — study linkage, deduplication, screening, reporting,
   living updates, provenance, diagnostics and governance, all reached through
   `SearchrightEngine`.
3. **Experimental edge** — live providers, licensed adapters, ranking, agents,
   WASI components and authenticated remote MCP transports, each capability-
   gated and unable to silently alter canonical evidence.
4. **Operational control plane** — GitHub issues/Project, release trains, health,
   telemetry, backup, incident, pilot and maturity evidence as declarative,
   additive and receipt-bearing contracts.

This structure contains experimental risk without freezing innovation.

## Scholarly product boundary

- **CiteWeft** is the extraction-evidence layer. It preserves source spans,
  uncertainty and diagnostics and may route to GROBID or other backends.
- **Searchright** is the review-methodology and search-workflow layer.
- **Sourceright** is the canonical citation/reference-verification layer.
- **`evidence-search-core`** is the product-neutral query/provider/audit kernel
  intended for reuse by Searchright and Sourceright.

Extraction evidence is not canonical bibliography. Screening state is not
citation truth. Citation verification is not review eligibility. Keeping those
boundaries explicit reduces coupling and prevents one system from silently
writing another system's canonical state.

## Strategic pillars

1. **Contract first**
   Review intent, query semantics, provenance, authority, screening and reports
   are versioned before adapters or interfaces.
2. **Shared kernel and facade**
   Product-neutral provider behaviour belongs in `evidence-search-core`; CLI,
   MCP and library interfaces delegate to one application facade.
3. **Deterministic before probabilistic**
   Fixtures, replay, canonical serialisation, conservative baselines and audit
   chains precede live APIs and agentic optimisation.
4. **Standards as versioned evidence maps**
   PRISMA, PRISMA-S, PRESS, JBI, Campbell, Cochrane and related guidance are
   data packs with provenance and gaps, not compliance badges.
5. **Human-governed agents**
   Agents may frame, translate, critique, rank and explain. Protocol amendments,
   irreversible writes and final exclusions require explicit authority.
6. **Provenance and living lineage**
   Reviews, strategies, runs, decisions and amendments form an immutable graph
   exportable as RO-Crate and W3C PROV artefacts.
7. **Quality and security by construction**
   Host allowlists, budget limits, hostile-content handling, least privilege,
   dependency policy, source inventories and evidence receipts are default.
8. **Institutional and accessible operation**
   Stable diagnostic codes, plain/JSON/JSONL output, data-handling decisions and
   retention/export policy are shared across hosts.
9. **Evidence before claims**
   Contracted → source-verified → compiler-verified → fixture-proven → opt-in
   live proven → externally validated → publicly accepted.
10. **Federated repositories without hidden coupling**
    Independent repositories integrate through exact pins, neutral contracts,
    consumer fixtures, compatibility windows and rollback—not submodules,
    copied implementations or unreviewed automatic upgrades.
11. **Declarative delivery control plane**
    Conductor remains canonical; GitHub issues, nested subissues and Project
    fields/views are generated projections with idempotent, non-destructive sync.
12. **Operational maturity before hosted claims**
    Remote identity, tenancy, SLOs, telemetry, backup, incident response and
    restore drills are separate evidence domains, not implicit consequences of
    an MCP server compiling.
13. **Release and adoption as experiments**
    Cross-repository canaries, release rehearsals and bounded pilots precede
    registry promotion and any version 1.0 decision.

## Federated repository strategy

Searchright should not force the entire repository estate into a monorepo. It
uses a federation protocol instead:

1. **Integration passport:** repository, exact revision, dependency direction,
   contracts, capabilities, verification, rollback and claim boundary.
2. **Consumer-driven interaction:** producer and consumer contracts, fixtures,
   gates and fail-closed behaviour.
3. **Leaf adapter:** optional Rust, CLI, MCP or WASI adapter; shared kernels stay
   independent of extraction or host-specific packages.
4. **Read-only drift surveillance:** detect upstream change, open review work,
   never mutate the pin or public claim automatically.
5. **Dual execution before cutover:** old and new paths run against golden and
   adversarial fixtures before downstream code is removed.
6. **Compatibility window:** schema and package versions remain available long
   enough for independent repository release cycles.

`evidence-search-core` remains a separately publishable workspace package while
its API is stabilised. It should move to an independent repository only when two
or more consumers require a genuinely independent lifecycle and release process;
premature extraction would increase coordination overhead without reducing
coupling.

## Delivery and operational strategy

The repository is intended to be bootstrapped into GitHub from committed source,
not configured by undocumented clicks. A manifest owns repository features,
merge policy, security controls, protected environments and the main ruleset.
Conductor renders the complete epic → track → phase → task issue graph, while a
Project v2 manifest owns custom fields and views. Apply is additive,
dry-run-first, clean-tree-gated and receipt-bearing; remote work state can never
promote implementation evidence.

Operational maturity is pursued in layers: local stdio MCP first, then
single-tenant authenticated remote MCP, then only evidence-backed institutional
or broader deployment. Telemetry remains disabled by default. Backup
recoverability requires restore drills. Cross-repository releases move through
contract, fixture, compiler, downstream-canary and RC gates with explicit human
promotion and rollback.

## Product positioning

Searchright is infrastructure for evidence retrieval and selection. It should
integrate with reference managers, review systems, repositories and research
pipelines rather than replace every user interface. CiteWeft supplies extraction
evidence, Searchright governs the review workflow, and Sourceright verifies
citation/reference integrity.

## Initial wedge

The MVP supports a bounded, high-value sequence:

- encode a review question and eligibility criteria;
- construct and independently review a portable strategy;
- compile it into source-specific syntax with explicit loss warnings;
- execute fixture/replay sources and authorised open providers;
- import, deduplicate and link records, reports and studies conservatively;
- record dual screening decisions and full-text exclusion reasons;
- render PRISMA-S evidence and PRISMA flow arithmetic;
- expose identical operations through Rust, CLI and MCP.

## Adoption sequence

1. **Developer and methods alpha:** contracts, fixtures, CLI/MCP, CiteWeft
   evidence and Sourceright compatibility.
2. **Information-specialist beta:** PRESS review, seed-set validation, real
   provider receipts and usability calibration.
3. **Institutional pilot:** policy packs, least privilege, retention profiles,
   signed releases and repository/review-system handoffs.
4. **Operational pilot:** authenticated single-tenant MCP, repository/Project
   control plane, health, restore and incident exercises, and downstream
   release-train canaries.
5. **Mature infrastructure:** licensed BYO adapters, living updates, component
   ecosystem, external methodological evaluation and an evidence-based 1.0
   decision.

## Evaluation strategy

Internal benchmarks establish regression protection, not methodological
validity. Mature claims require preregistered external evaluation across topics,
sources and review types, with information specialists, blinded adjudication,
raw receipts and a response-to-findings matrix.

## Non-goals before 1.0

- autonomous final inclusion or exclusion;
- scraping or circumventing licensed databases;
- truth, clinical recommendation or legal-compliance claims;
- hosted multi-tenant SaaS without deployed tenancy isolation, data-residency,
  abuse-budget and adversarial-test receipts;
- opaque “one-click systematic review” positioning;
- deleting downstream custom code before fixture parity and rollback evidence;
- describing CiteWeft as a GROBID port, fork or drop-in replacement;
- treating prepared consumer contracts as downstream compatibility.
- remote GitHub repository, issue or Project claims without an observed bootstrap receipt;
- backup recoverability, telemetry safety or operational SLO claims without exercises;
- version 1.0 promotion from source completeness alone;
