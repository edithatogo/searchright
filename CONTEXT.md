# Searchright context spine

This file is the entry point for human and agent work. Load only the context
needed for the active task, but never bypass the non-negotiable policy layer.

## Required load order

1. `AGENTS.md` — authority, safety and evidence rules.
2. `context/manifest.json` — canonical context inventory and evidence ceiling.
3. `conductor/requirements.md` — MoSCoW product contract.
4. The active `conductor/tracks/NN-*/spec.md`, `plan.md`, `metadata.json` and
   `evidence.json`.
5. Applicable schemas, ADRs and implementation paths.
6. `context/hazard-log.json`, `context/capability-matrix.json` and
   `context/claim-boundaries.json` before consequential changes.

## Architectural boundaries

- CiteWeft owns backend-neutral scholarly document extraction evidence.
- `evidence-search-core` owns query compilation and bounded provider execution.
- Searchright owns review planning, search orchestration, deduplication,
  screening, study/report linkage and reporting.
- Sourceright owns citation/reference canonicalisation and verification.
- Cross-repository integration occurs through pinned passports and neutral
  contracts, never through copied private logic or implicit shared state.

## Current evidence ceiling

The repository is source-verified. It is not compiler-verified, fixture-proven,
live-provider-proven, externally validated, remotely issue-synchronised or
registry-accepted until corresponding receipts exist.

## Remote-effect rule

Network, external writes, telemetry, registry submission, downstream migration,
GitHub issue synchronisation and final screening exclusion are denied or dry-run
by default and require explicit authority plus audit evidence.
