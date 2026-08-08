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
against 198 acceptance assertions. Sixty-five assertions currently have
individual symbol/test/gate mappings; other assertions remain conservatively
track-level. No track is complete merely because its source paths, GitHub issues
or Project items exist.

The planning projection still contains 564 nodes: one epic, 38 track issues, 152
phase subissues and 373 task subissues. The delivery Project now separates
implementation state from evidence level and has 13 fields and six views. A
second, non-mutating portfolio projection tracks cross-repository contracts,
licence decisions, migrations and the release train without importing all 564
task items.

The local source has added neutral contracts, four provider baselines, a
seven-dialect native-query corpus, a deterministic review-state reducer, a
portable review bundle, benchmark leakage controls, licence-aware integration
passports, companion change packets, a frozen contract surface and public-package
policy. These improvements advance multiple tracks but do not erase their open
compiler, fixture, live, downstream and external gates.

## Near-term execution order

1. Generate `Cargo.lock`, compile and repair the complete workspace.
2. Run provider fixture golden tests and finish one complete PubMed vertical
   slice before broadening provider scope.
3. Add database-specific native parsers and independently reviewed query gold
   sets.
4. Prove the neutral contract/core API in Sourceright and CiteWeft consumer
   canaries.
5. Execute the prepared UOGTO, VOIAGE and agent-repository migrations without
   deleting rollback paths.
6. Run sealed methodological benchmarks and human information-specialist review.
7. Apply the GitHub control plane, release train and registry submissions only
   after the required receipts exist.

`PROJECT_STATUS.md`, assertion-level traceability and machine-readable receipts
are the claim authority.
