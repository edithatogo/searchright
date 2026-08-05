# Architecture

Searchright uses a hexagonal architecture. Domain contracts and deterministic
logic do not depend on provider SDKs, MCP transports or storage engines.

```mermaid
flowchart TB
  subgraph Interfaces
    CLI[CLI]
    MCP[MCP 2026-07-28]
    RUST[Rust API]
    SKILL[Agent skill]
  end
  subgraph Product
    PLAN[Planning and protocol]
    EXEC[Execution orchestration]
    DEDUP[Deduplication]
    SCREEN[Screening]
    REPORT[PRISMA and appendices]
  end
  subgraph SharedCore[evidence-search-core]
    AST[Query AST and dialect compiler]
    PR[Provider runtime]
    RECEIPT[Evidence receipts]
    AUDIT[Hash-chained audit]
  end
  subgraph Adapters
    PUBMED[PubMed]
    EPMC[Europe PMC]
    OA[OpenAlex / Crossref]
    REG[Trial registries]
    IMPORT[RIS / CSL / CSV]
    LICENSED[Licensed BYO access]
  end
  subgraph Persistence
    JSONL[Append-only JSONL]
    SQLITE[SQLite / DuckDB feature]
    ROCRATE[RO-Crate export]
  end

  CLI --> Product
  MCP --> Product
  RUST --> Product
  SKILL --> MCP
  Product --> SharedCore
  PR --> Adapters
  AUDIT --> Persistence
  RECEIPT --> Persistence
```

## Dependency rule

Dependencies point inward. `evidence-search-core` may depend on contracts and
portable runtime primitives. It must not depend on Searchright screening,
Sourceright CSL logic, an MCP server, a CLI or a database-specific UI.
