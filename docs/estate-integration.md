# GitHub estate integration and migration

## Key finding

The requested `edithatogo/sourcerightlibrary` path was not found. The active
repository is `edithatogo/sourceright`. Its current `src/live_providers.rs`
contains provider configuration, retries, cache controls, endpoint construction,
HTTP execution and fixture parsing. These generic responsibilities are the first
shared-core extraction target.

## Repository integration matrix

| Repository | Reuse or replacement |
| --- | --- |
| `sourceright` | Replace generic provider runtime/query/receipt code with `evidence-search-core`; retain CSL/citation verification. |
| `citeweft` | Interoperate for bibliography parsing/canonicalisation; no duplicate citation parser. |
| `academic-research-skills` | Replace custom systematic-review search execution with Searchright MCP tools; retain high-level research orchestration and integrity gates. |
| `research-skills` / `scholarly-publishing-agents` | Publish thin skills/prompts that call Searchright rather than embedding provider code. |
| `PRISMA.jl` | Use as a parity comparator and migration source for flow/checklist behaviour; avoid two canonical flow models once Rust parity is proven. |
| `synergy-dataset` | Use as a screening benchmark and calibration corpus under its licence/provenance. |
| `standards_check` | Source versioned reporting checklists and provenance sidecars. |
| `repository-standards` | Register Searchright as high-risk research software and inherit CI/security/release controls. |
| `api-standards` / conformance repos | Reuse API, contract and receipt conventions. |
| `osf-mcp-server` | Integrate protocol registration and artefact deposit after Searchright contracts stabilise. |
| `mcp-registry`, `awesome-mcp-servers`, `awesome-agent-skills` | Submission and discovery targets, not sources of duplicated core code. |

## Migration sequence

1. Publish `evidence-search-core` inside this workspace with fixture parity.
2. Add a compatibility adapter in Sourceright behind a feature flag.
3. Run Sourceright's provider fixtures through both implementations.
4. Switch generic runtime ownership to the shared crate.
5. Delete superseded code only after parity, semver and rollback evidence.
6. Repeat the estate audit with GitHub code search and track each replacement in
   `migration/estate-migration-manifest.yaml`.

No remote repository has been modified by this scaffold.
