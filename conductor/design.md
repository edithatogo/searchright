# Design

## Context

```mermaid
C4Context
  title Searchright context
  Person(reviewer, "Review team", "Researchers, information specialists and screeners")
  System(searchright, "Searchright", "Contract-first search, screening and reporting infrastructure")
  System_Ext(citeweft, "CiteWeft", "Scholarly-document extraction evidence")
  System_Ext(sourceright, "Sourceright", "Canonical citation and reference verification")
  System_Ext(sources, "Information sources", "Databases, registers, repositories and websites")
  System_Ext(reviewtools, "Review tools", "Reference managers and screening platforms")
  System_Ext(registries, "Distribution and research registries", "MCP, crates.io, OSF, JOSS and directories")
  Rel(reviewer, searchright, "Plans, approves, screens and audits")
  Rel(searchright, sources, "Executes authorised bounded searches")
  Rel(searchright, reviewtools, "Imports, exports and hands off evidence")
  Rel(citeweft, searchright, "Provides source-grounded DocumentEvidence")
  Rel(searchright, sourceright, "Provides reviewed records and shares evidence-search-core")
  Rel(searchright, registries, "Publishes evidence-gated artefacts")
```

## Containers

```mermaid
flowchart TB
  subgraph Hosts
    CLI[searchright CLI]
    MCP[searchright-mcp stdio]
    API[Rust API]
    SKILL[systematic-search skill]
  end
  subgraph Application
    ENGINE[SearchrightEngine]
    PLAN[Planning/amendments]
    VALIDATE[PRESS/seed validation]
    REVIEW[Dedup/study/screening]
    REPORT[PRISMA/standards/living]
    GOV[Diagnostics/governance/assurance]
  end
  subgraph Kernel[evidence-search-core]
    AST[Portable query AST]
    COMP[Dialect compiler]
    PROVIDER[Provider runtime]
    RECEIPT[Source receipt]
    AUDIT[Hash-linked events]
  end
  subgraph Adapters
    OPEN[Open provider adapters]
    IMPORT[Interchange adapters]
    LICENSED[BYO licensed adapters]
    WASI[Verified WASI components]
  end
  subgraph Evidence
    STORE[Audit and snapshots]
    PROV[RO-Crate / PROV]
    BENCH[Benchmark/calibration receipts]
  end

  SKILL --> MCP
  CLI --> ENGINE
  MCP --> ENGINE
  API --> ENGINE
  ENGINE --> PLAN
  ENGINE --> VALIDATE
  ENGINE --> REVIEW
  ENGINE --> REPORT
  ENGINE --> GOV
  Application --> Kernel
  PROVIDER --> Adapters
  Kernel --> Evidence
  Application --> Evidence
```

## Lifecycle assurance

```mermaid
stateDiagram-v2
  [*] --> Draft
  Draft --> PlanApproved: human approval
  PlanApproved --> StrategyValidated: PRESS/seed/translation approval
  StrategyValidated --> ExecutionApproved: human release
  ExecutionApproved --> SearchExecuted: bounded provider run
  SearchExecuted --> Deduplicated: reviewed clusters
  Deduplicated --> TitleAbstractComplete: dual screening/reconciliation
  TitleAbstractComplete --> FullTextComplete: human full-text closure
  FullTextComplete --> Reported: validated counts and artefacts
  Reported --> UpdatePlanned: living cadence/amendment
  UpdatePlanned --> StrategyValidated: revalidation
```

## Provider execution

```mermaid
sequenceDiagram
  participant U as Human or authorised agent
  participant H as CLI/MCP host
  participant E as SearchrightEngine
  participant K as evidence-search-core
  participant P as Provider adapter
  participant S as Evidence store
  U->>H: strategy + envelope + approval
  H->>E: canonical operation
  E->>K: validated request
  K->>K: check host, capability, budgets and cache mode
  loop bounded pages
    K->>P: page request
    P-->>K: normalised page and cursor
  end
  K->>K: digest query and evidence
  K->>S: append receipt and audit event
  K-->>E: records, receipt and warnings
  E-->>H: canonical result
  H-->>U: reviewable output
```

## Security and evidence boundary

```mermaid
flowchart LR
  INPUT[Untrusted plan, query, metadata or full text] --> SCHEMA[Schema and semantic validation]
  SCHEMA --> AUTH[Authority and institutional policy]
  AUTH --> CAP[Capability and endpoint allowlist]
  CAP --> LIMIT[Rate, page, record, size and duration budgets]
  LIMIT --> ADAPTER[Fixture/replay/live adapter]
  ADAPTER --> REDACT[Secret and hostile-content controls]
  REDACT --> RECEIPT[Digest, receipt and audit chain]
  RECEIPT --> HUMAN[Human gate for consequential transitions]
  RECEIPT --> CLAIM[Evidence-level claim gate]
```

## Scholarly-domain integration

```mermaid
flowchart LR
  DOC[Scholarly document] --> CW[CiteWeft]
  CW --> DE[DocumentEvidence]
  DE --> SRCH[Searchright]
  SRCH --> SR[Sourceright]
  CORE[evidence-search-core] --> SRCH
  CORE --> SR
  CW -. extraction evidence only .-> DE
```

## Conductor and GitHub control plane

```mermaid
flowchart TB
  COV[roadmap-coverage.json] --> TRACKS[38 Conductor tracks]
  TRACKS --> PLANS[Four phases per track]
  PLANS --> TASKS[392 top-level tasks]
  COV --> RENDER[Deterministic issue renderer]
  RENDER --> EPIC[One roadmap epic]
  EPIC --> ISSUES[38 track issues]
  ISSUES --> PHASES[152 phase subissues]
  PHASES --> TSUB[392 task subissues]
  TSUB --> PROJECT[Project v2: 13 custom fields / 6 views]
  SETTINGS[Repository settings + main ruleset] --> BOOT[Dry-run-first bootstrap]
  PROJECT --> BOOT
  TSUB --> BOOT
  BOOT --> RECEIPT[Observed remote receipt]
  RECEIPT -. cannot promote .-> COV
```

## Operational deployment boundary

```mermaid
flowchart LR
  LOCAL[Local stdio MCP] --> ENGINE[SearchrightEngine]
  REMOTE[Authenticated streamable HTTP MCP] --> AUTH[Identity + tenant + scope decision]
  AUTH --> BUDGET[Region, concurrency, rate and egress budgets]
  BUDGET --> ENGINE
  ENGINE --> HEALTH[Health/readiness]
  ENGINE --> AUDIT[Audit and incident evidence]
  ENGINE --> BACKUP[Backup manifest / restore drill]
  ENGINE -. explicit opt-in only .-> TELEMETRY[Allowlisted telemetry]
```

## Release and maturity promotion

```mermaid
flowchart LR
  CW[CiteWeft exact revision] --> CONTRACT[Contract gate]
  CONTRACT --> FIXTURE[Consumer fixture]
  FIXTURE --> BUILD[Compiler gate]
  BUILD --> CANARY[Downstream canary]
  CANARY --> RC[Release candidate rehearsal]
  RC --> PILOT[Bounded pilots + rollback]
  PILOT --> DOSSIER[Maturity dossier]
  DOSSIER -->|all thresholds met| READY[Version 1.0 ready decision]
  DOSSIER -->|blocker remains| NOTREADY[Not ready + gap track]
```

## Key decisions

- One application facade prevents CLI/MCP drift.
- Record, report and study are separate entities.
- Standards packs report evidence and gaps; they do not certify quality.
- Provider content is inert data, never agent instruction.
- Live and licensed access are explicit opt-ins.
- Ranking is advisory and requires calibration; no automatic final exclusion.
- Downstream migration is dual-run and reversible.
- Contract evolution and public claims are evidence-gated.
- GitHub state is a generated coordination projection, not evidence authority.
- Remote MCP authentication/tenancy and operational recovery are distinct maturity domains.
- Cross-repository promotion requires downstream canaries, RC rehearsal and rollback.
