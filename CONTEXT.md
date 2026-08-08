# Searchright context spine

This file is the entry point for human and agent work. Load only the context
needed for the active task, but never bypass the non-negotiable policy layer.

## Required load order

1. `AGENTS.md` — authority, safety and evidence rules.
2. `context/manifest.json` — canonical context inventory and evidence ceiling.
3. `context/claim-boundaries.json`, `context/capability-matrix.json` and
   `context/hazard-log.json` — what can be claimed or done.
4. `conductor/requirements.md` and `conductor/roadmap-coverage.json` — complete
   product and evidence contract.
5. The active `conductor/tracks/NN-*/spec.md`, `plan.md`, `metadata.json` and
   `evidence.json`.
6. Applicable schemas, ADRs, integration passports and implementation paths.
7. For delivery work, load `CODEX_HANDOFF.md`,
   `conductor/github/issue-hierarchy.json`,
   `conductor/github/project.json` and
   `conductor/github/repository-settings.json`.
8. For release or maturity work, load `integration/release-train.json`,
   `release/rehearsal.json` and `conductor/maturity-dossier.json`.

## Architectural boundaries

- CiteWeft owns backend-neutral scholarly document extraction evidence.
- `evidence-search-core` owns query compilation and bounded provider execution.
- Searchright owns review planning, search orchestration, deduplication,
  screening, study/report linkage and reporting.
- Sourceright owns citation/reference canonicalisation and verification.
- Cross-repository integration occurs through pinned passports, consumer
  contracts and release-train receipts, never through copied private logic,
  submodules or implicit shared state.
- Local stdio MCP is distinct from authenticated remote MCP. Remote identity,
  tenancy, data residency and abuse budgets have separate contracts and gates.
- Conductor is canonical planning state. GitHub issues, subissues and Project
  fields are generated coordination projections and cannot promote evidence.

## Current evidence ceiling

The repository is source-verified. It is not compiler-verified, fixture-proven,
live-provider-proven, downstream-compatible, operationally rehearsed,
externally validated, remotely bootstrapped or registry-accepted until matching
receipts exist.

## Remote-effect rule

Network, external writes, telemetry, registry submission, downstream migration,
GitHub repository/issue/Project mutation, release promotion and final screening
exclusion are denied or dry-run by default. Apply requires explicit authority,
a clean state, least privilege, preview and an audit receipt. No script may
delete remote issues, Projects, fields, releases or repositories automatically.

## Maturity rule

Source completeness is not a version 1.0 decision. The final maturity track may
record `ready` only when the dossier's compiler, fixture, live, downstream,
security, operational, usability, methodological and publication thresholds are
satisfied and no critical hazard remains unaccepted.
