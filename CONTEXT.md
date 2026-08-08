# Searchright context spine

This file is the entry point for human and agent work. Load only the context
needed for the active task, but never bypass the non-negotiable policy and claim
layers.

## Required load order

1. `AGENTS.md` — authority, safety and evidence rules.
2. `context/manifest.json` — canonical inventory and evidence ceiling.
3. `context/claim-boundaries.json`, `context/capability-matrix.json` and
   `context/hazard-log.json` — permitted actions and claims.
4. `conductor/requirements.md`, `conductor/roadmap-coverage.json` and the active
   track's `traceability.json` — requirements, state and assertion evidence.
5. `release/public-packages.json` and
   `contracts/compatibility/schema-surface-0.1.0-alpha.1.json` for package or
   contract changes.
6. Applicable schemas, ADRs, provider baselines, integration passports and
   implementation paths.
7. For delivery work, load `CODEX_HANDOFF.md`, the GitHub issue hierarchy,
   delivery Project and repository settings.
8. For cross-repository work, load `integration/ecosystem-lock.json`,
   `integration/release-train.json`, the companion change packet and the
   strategic portfolio projection.
9. For release or maturity work, load `release/rehearsal.json` and
   `conductor/maturity-dossier.json`.

## Architectural boundaries

- `evidence-search-contracts` owns neutral query/provider/record/receipt/audit
  wire contracts.
- `evidence-search-core` owns neutral compilation, bounded provider execution,
  replay/cache and audit-ledger behaviour.
- CiteWeft owns backend-neutral document extraction evidence.
- Searchright owns review planning, search orchestration, deduplication,
  screening, study/report linkage, living updates and reporting.
- Sourceright owns citation/reference canonicalisation and verification.
- Cross-repository integration occurs through pins, passports, consumer
  contracts and receipts—not copied private logic, submodules or implicit state.
- Local stdio MCP is distinct from authenticated remote MCP.
- Conductor is canonical planning state. GitHub issues and Projects are
  coordination projections and cannot promote implementation or evidence.

## Implementation truth rule

The roadmap contains 198 acceptance assertions. A track status is bounded by its
assertions; path existence is not behaviour proof. Use
`conductor/tracks/NN-*/traceability.json` and the latest receipt to determine what
may be claimed. The vertical-slice completion rule is in
`docs/vertical-slice-definition-of-done.md`.

## Current evidence ceiling

The repository is source-verified and assertion-rebaselined. It is not
compiler-verified, end-to-end fixture-proven, live-provider-proven,
downstream-compatible, operationally rehearsed, externally validated, remotely
bootstrapped or registry-accepted until matching receipts exist.

All packages are non-publishable and zero packages are marked publish-ready.
Provider baselines are local drift controls, not live API evidence. The derived
review-state snapshot is disposable and requires an externally verified audit
head. The `.srpack` bundle proves declared byte integrity, not methodological
adequacy.

## Remote-effect rule

Network, external writes, telemetry, registry submission, downstream migration,
GitHub mutation, release promotion and final screening exclusion are denied or
dry-run by default. Apply requires explicit authority, clean state, least
privilege, preview, bounded effects and an audit receipt. No script may delete
remote issues, Projects, fields, releases or repositories automatically.

## Licence and upstream rule

An integration must identify original versus forked origin, canonical upstream,
local-fork role, code/content/model licences and redistribution constraints.
Reference-only or review-required sources cannot enter redistributed packages or
standard packs without a recorded rights decision.

## Maturity rule

Source breadth is not a version 1.0 decision. The final maturity track may record
`ready` only when compiler, fixture, live, downstream, security, operational,
usability, methodological, licence and publication thresholds are satisfied and
no critical hazard remains unaccepted.
