# Roadmap

The roadmap is implementation- and evidence-driven. Every track has source
artefacts, a machine-readable `evidence.json`, explicit higher-evidence blockers
and an ordered dependency graph in `conductor/roadmap-coverage.json`. Each track
maps to a GitHub issue, each four-phase plan maps to phase subissues, and every
top-level plan task maps to a task subissue. All nodes are projected into the
manifest-owned GitHub Project v2.

## Horizon 0 — Foundation (`00`–`03`)

- Git, Conductor, toolchain and repository-standard bootstrap.
- Versioned contract and standards catalogues.
- Portable query AST with deterministic, review-required loss reporting.
- Shared bounded provider runtime and reversible Sourceright compatibility.

**Exit gate:** source-verified architecture plus compiler/fixture evidence for the
stable kernel; downstream migration remains separately gated.

## Horizon 1 — MVP (`04`–`11`)

- Open provider adapters and replay fixtures.
- Audit, receipts, crash-conscious local storage and deterministic import/export.
- Conservative deduplication and record-report-study linkage.
- Governed dual-phase screening and exclusion reasons.
- PRISMA/PRISMA-S, PRESS and complete search-strategy outputs.
- Shared Rust facade, CLI, MCP stdio server and systematic-search agent skill.

**Exit gate:** cross-platform compilation, complete deterministic tests, CLI
install smoke and MCP protocol transcripts.

## Horizon 2 — Alpha integration and release (`12`–`19`)

- CiteWeft document-evidence adapter with source-span and uncertainty preservation.
- Pinned integration passports, consumer-contract suites and canonical context.
- Sourceright dual-run adoption and rollback.
- GitHub-estate code inventory and repository-specific replacements.
- Maximal quality/security harness, source SBOM and supply-chain policy.
- Search, deduplication and prioritisation benchmark infrastructure.
- Signed reproducible alpha release and truthful registry/JOSS packets.

**Exit gate:** committed lockfile, evidenced coverage/mutation/security receipts,
signed artefacts, producer-consumer receipts and downstream migration proof.
Prepared packets are not submission or acceptance.

## Horizon 3 — Public beta and experimental edge (`20`–`24`)

- Grey literature, registers, repositories, citation chaining and contacts.
- Licensed BYO-access profiles and user-authorised transports.
- Living-review updates, amendments and prior-run lineage.
- Transparent calibrated prioritisation with no autonomous exclusion.
- Sandboxed WASI providers and remote-MCP boundary design.

**Exit gate:** source-specific live receipts, human calibration, component
conformance, transport threat model and backward-compatible MCP evidence.

## Horizon 4 — Mature research infrastructure (`25`–`30`)

- RO-Crate and W3C PROV research-object exports.
- Formal workflow assurance and contract-evolution policy.
- Accessible diagnostics, internationalisation readiness and usability testing.
- Institutional data governance, least privilege and collaboration controls.
- Independent methodological evaluation and sustainability evidence.
- Explicit maturity gap register rather than premature 1.0 promotion.

**Exit gate:** compiler, fixture, live-provider, interoperability, migration,
security, usability and external-evaluation evidence with no critical blocker.

## Horizon 5 — Operational maturity and version 1.0 decision (`31`–`37`)

- **31 GitHub control plane:** create/verify the remote, apply repository
  settings and protections, synchronise all native issue/subissue relationships,
  and create/populate the Project v2 through additive, receipt-bearing commands.
- **32 Cross-repository release train:** order CiteWeft, Searchright and
  Sourceright contract, fixture, compiler, downstream-canary, RC and promotion
  gates with rollback.
- **33 Operational reliability:** health/readiness, SLO, telemetry, backup,
  restore and incident contracts and exercises.
- **34 Authenticated remote MCP:** tenant isolation, data residency, scoped
  authorisation, long-running-task budgets, replay/abuse resistance and audit.
- **35 SDK, documentation and adoption:** generated/thin client surfaces,
  examples, migration guides, compatibility policy and adoption telemetry that
  remains opt-in.
- **36 Release candidate and pilots:** clean-room RC rehearsal, local,
  institutional self-hosted and remote single-tenant pilots, rollback exercise
  and registry submission rehearsal.
- **37 Version 1.0 decision:** aggregate every required receipt, resolve or
  explicitly accept residual risk, and record `ready` or `not_ready`; source
  completeness cannot decide this track.

**Exit gate:** observed remote control-plane receipt, operational drills,
authenticated transport and tenancy tests, downstream canaries, successful RC
and pilots, external methods/usability/security evidence, and a final maturity
decision with no unaccepted critical hazard.

## Current position

All 38 maturity tracks remain present, but their status has been rebaselined
against 199 acceptance assertions. Seventy-two assertions currently have
individual symbol/test/gate mappings; other assertions remain conservatively
track-level. No track is complete merely because its source paths, GitHub issues
or Project items exist.

The planning projection contains 583 nodes: one epic, 38 track issues, 152
phase subissues and 392 task subissues. The delivery Project separates
implementation state from evidence level and has 13 fields and six views. A
second, non-mutating portfolio projection tracks cross-repository contracts,
licence decisions, migrations and the release train without importing all 583
work items.

The local source has added neutral contracts, four provider baselines, a
seven-dialect native-query corpus, a deterministic review-state reducer, a
portable review bundle, benchmark leakage controls, licence-aware integration
passports, companion change packets, a frozen contract surface and public-package
policy. These improvements advance multiple tracks but do not erase their open
compiler, fixture, live, downstream and external gates.

## Executable launch-preparation order

The canonical machine-readable launch path is
`conductor/launch-preparation-roadmap.json`. It contains 16 dependency-ordered
work packages. Each package has exactly one owning track, named commands,
required receipts and a fail-closed exit criterion:

1. MCP schemas and current/previous-client transcripts (Track 10).
2. MCP resources, prompts, tasks, MRTR, pagination and cancellation (Track 24).
3. Authenticated Streamable HTTP and tenancy controls (Track 34).
4. Provider transport security, policy review and authorised live canaries (Track 04).
5. Query fidelity, named filters and reviewed strategies (Track 02).
6. Audit/store recovery, retention, export and deletion (Track 05).
7. Import, deduplication, linkage and screening fixtures (Track 06).
8. PRISMA/PRESS and accessible reporting polish (Track 08).
9. CiteWeft/Sourceright canaries, dual-run parity and rollback (Track 32).
10. Coverage, mutation, fuzz, formal and dependency-trust maturity gates (Track 16).
11. Privacy, accessibility and usability evaluation (Track 28).
12. Preregistered sealed methodological evaluation (Track 29).
13. Encrypted restore, incident and SLO rehearsals (Track 33).
14. Generated SDKs and fixture-backed launch documentation (Track 35).
15. Reproducible release candidate, pilots and rollback (Track 36).
16. Accountable maturity decision and separately approved submissions (Track 37).

Execution still uses one track per PR. Cross-track dependencies coordinate the
order; they are not permission for omnibus pull requests.

`PROJECT_STATUS.md`, assertion-level traceability and machine-readable receipts
are the claim authority.
