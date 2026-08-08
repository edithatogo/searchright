# GitHub estate integration and migration

## Repository discovery

The requested name `edithatogo/sourcerightlibrary` was not present during the
audit. The active repository is `edithatogo/sourceright`. The migration packet
is pinned to the inspected `src/live_providers.rs` blob
`57bc071c6afc7d5a4cb8ead12112919a446ebd24`.

## Shared-core decision

A shared core is warranted, but only for product-neutral behaviour:

- query representation and compilation;
- provider registration and capability description;
- host, timeout, rate, retry, pagination and result budgets;
- cache/replay interfaces;
- normalised pages, receipts and audit events.

CiteWeft-specific document/reference extraction, spans and routing diagnostics stay in CiteWeft. Sourceright-specific CSL candidate comparison, citation reconciliation and reference verification stay in Sourceright. Searchright-specific protocols, screening and
PRISMA outputs stay in Searchright.

## Estate matrix

| Repository | Integration decision |
| --- | --- |
| `sourceright` | Adopt `evidence-search-core` behind compatibility feature; dual-run fixtures; delete generic runtime only after parity and rollback evidence. |
| `citeweft` | Reuse optional scholarly extraction evidence, source spans, uncertainty and routing diagnostics through `searchright-citeweft`; canonicalisation and reference verification remain Sourceright responsibilities. |
| `academic-research-skills` | Replace embedded search execution with Searchright CLI/MCP calls; retain higher-level research orchestration and integrity gates. |
| `research-skills` / `scholarly-publishing-agents` | Thin skills over Searchright; no provider implementation in prompts. |
| `PRISMA.jl` | Parity comparator and migration source for checklist/flow behaviour; avoid two canonical flow models after verified parity. |
| `synergy-dataset` | Rights-aware benchmark and human-calibration corpus. |
| `standards_check` | Upstream provenance source for versioned reporting checklists. |
| `repository-standards` | Register Searchright as research software with elevated security/release controls. |
| `api-standards` / conformance repositories | Reuse API, compatibility and verification-receipt conventions. |
| `osf-mcp-server` | Protocol registration and artefact deposit adapter after contract stability. |
| `mcp-registry`, `awesome-mcp-servers`, `awesome-agent-skills` | Distribution targets, not duplicated implementation sources. |
| research project repositories | Replace one-off PubMed/registry/import/dedup scripts only after the estate scanner classifies and a repository-specific migration issue is approved. |

The machine-readable inventory and patterns live under `migration/estate/`.

## Safe migration protocol

1. Inventory the downstream code and pin the exact source revision.
2. Map every legacy symbol and behaviour to a shared-core owner or explicit
   retention decision.
3. Import rights-cleared fixtures and preserve their provenance.
4. Add a compatibility adapter and feature-gated rollback path.
5. Dual-run old and new implementations over deterministic fixtures.
6. Compare identifiers, fields, errors, retries, cache behaviour, redaction,
   receipts and ordering.
7. Record and approve any intended difference.
8. Run the downstream repository’s compiler, tests, security and release gates.
9. Switch the default while retaining rollback for one compatible release.
10. Delete superseded code only after the evidence receipt is complete.

## Current state

Searchright contains the mapping, parity cases, compatibility helpers and estate
scanner. No downstream repository was modified, no dual-run was executed and no
custom code was deleted in this environment. Those are integration-evidence
tasks, not source tasks that can be closed by changing metadata.
