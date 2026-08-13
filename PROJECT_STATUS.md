# Project status

**Status date:** 13 August 2026

**Verified revision:** `10f983d44e67c08be44131ea6e2b7cf75dc147f6` (`main`)

**Evidence ceiling:** source-verified alpha with hosted compiler and admission evidence
**Maturity decision:** not ready

## Headline

Searchright is a clean, protected and cross-platform-compiled Rust alpha. Its
neutral contracts, shared execution core, application facade, CLI and local
MCP server have substantial working implementations. The repository has strong
static, compiler, security, supply-chain, packaging and formal-assurance gates.

It is not yet a validated systematic-review product, authenticated hosted
service, release candidate or publishable package. Live provider behavior,
methodological performance, downstream adoption, usability, operational
recovery and external acceptance remain separate evidence gates.

## Repository and delivery state

- Local `main` and `origin/main` are identical at the verified revision.
- The working tree is clean; there are no open pull requests or delivery
  branches.
- The Track 05 closeout PR is merged; no pull request is open at this
  observation.
- The active main ruleset requires strict, linear admission and the PR scope
  policy. Future PRs declare one Conductor track; a technically inseparable
  multi-track exception requires its explicit label and rationale.
- The public repository, native issue/subissue hierarchy and delivery Project
  exist. The latest committed source-bound audit observed all 583 canonical
  issues, all 582 relationships and zero content, label, task-state or
  recognised Project-field drift. Later Track 05 archival changes retain the
  same additive identities; an exact-current-source audit remains a separate
  closeout check whenever the projection changes.

## Historical bootstrap disclosures

- **Rust compilation:** it is now evidenced on merged main across Ubuntu,
  Windows and macOS; the original generation-environment receipt remains a
  historical record and is not rewritten.
- **Live provider calls:** none is claimed. Provider support remains limited to
  deterministic fixtures until authorised redacted canaries exist.
- **GitHub repository creation/push:** completed. The 583-node projection has a
  committed source-bound zero-drift audit; subsequent projection changes still
  require their own exact-source audit.
- **Conductor plugin installation:** the host previously reported Conductor
  0.4.1 installed, while repository contracts retain their pinned baseline;
  this status does not assert general host-version compatibility.

## Hosted evidence on merged main

The exact verified revision completed 17 hosted checks successfully, with no
failure or pending result:

- Rust 1.97.1 on Ubuntu, Windows and macOS;
- declared Rust-version admission;
- repository-wide formatting, Clippy, tests and documentation through CI;
- static contracts and roadmap evidence;
- LLVM coverage admission;
- CodeQL and full-history Gitleaks scanning;
- Rust dependency, advisory, unused-dependency and cargo-vet policy;
- clean-room vendored build and install smoke;
- public API and SemVer checks;
- Kani, Miri, Loom and standard-library precondition suites;
- OpenSSF Scorecard and workflow policy.

These checks establish compiler and repository-admission evidence for the exact
revision. They do not establish live-provider correctness, methodological
validity, production security, usability or operational recovery.

## Implemented surfaces

- 30-crate Rust 2024 workspace; every crate remains non-publishable by default.
- 60 JSON Schema 2020-12 contracts and canonical examples.
- Neutral `evidence-search-contracts` and shared `evidence-search-core` layers.
- Review planning, eligibility, query compilation, provider execution,
  receipts, audit, storage, import, deduplication, record/report/study linkage,
  screening, PRISMA/PRESS reporting, living updates and provenance foundations.
- Deterministic provider fixtures for open connectors, with live support still
  unclaimed.
- Shared Rust facade and CLI operation hierarchy.
- Local stdio MCP server targeting MCP 2026-07-28 with 31 tools, structured
  content, field-level catalogue-derived output schemas, explicit
  read-only/non-destructive effect annotations and pinned 2026-07-28 plus
  2025-11-25 stdio transcripts.
- Default-deny external writes, human-only final screening authority, receipt
  redaction, package publication gates and release/maturity blockers.
- Prepared CiteWeft and Sourceright integration passports, consumer contracts,
  migration packets and rollback boundaries.

## Current Conductor position

The roadmap contains 38 tracks and 200 acceptance assertions. Canonical state:

- 3 tracks are source implemented;
- 25 tracks are partially implemented;
- 7 tracks are scaffolded;
- 3 tracks require external evidence;
- Tracks 05 and 31 are semantically archived in place; their canonical paths
  and stable GitHub identities are retained.

At assertion level, 19 are source implemented, 124 partially implemented, 47
scaffolded and 10 external-evidence-required. 116 assertions still have only
track-level mappings and 437 open gate entries remain. These counts describe
evidence debt, not a quality score.

## Coverage and dependency trust

- Coverage admission uses a zero-regression ratchet with 61.02% line and
  83.70% patch baselines. Merged main currently measures 62.02% line coverage,
  so hosted admission is green. The greater-than-90% maturity target remains
  open.
- Cargo-vet admission is green with 41 dependencies fully audited through
  approved peer imports and 242 exact temporary exemptions. Searchright has no
  local audit entries. The exemptions are owned, issue-linked and scheduled for
  review by 10 November 2026. They are risk acceptances, not audits or safety
  certification.

## Open critical evidence domains

### MCP and hosted access

- Third-party MCP client interoperability beyond the pinned official `rmcp`
  client remains unclaimed; current and previous-era official-client stdio
  conformance now covers all 31 tools and 32 successful output shapes.
- Bounded local-stdio resources, prompts, completion, MRTR, pagination,
  subscriptions, task completion and cooperative cancellation are now
  compiler-evidenced. Durable, resumable, multi-replica and tenant-bound task
  state, lossless subscription delivery, production load/cache evidence and
  remote advanced capabilities remain open.
- Authenticated Streamable HTTP with verified issuer, principal, tenant,
  region, scope, rate, replay, isolation and rollback evidence.

### Providers and methodology

- Authorized redacted live canaries for each claimed provider.
- Completed provider terms/licence/data-handling review.
- DNS resolution and connection-pinning evidence for live endpoint security.
- Independently reviewed PRESS strategies and rights-cleared gold corpora.
- Sealed retrieval, translation, deduplication and prioritisation evaluation.
- Information-specialist and usability calibration.

### Ecosystem and operations

- CiteWeft and Sourceright producer/consumer canaries, dual-run parity and
  rollback rehearsal.
- Representative persisted-data migration and backward-reader evidence.
- Authenticated multi-tenant deployment and incident exercises.
- Successful encrypted backup restore drill.
- Bounded institutional pilots and operational SLO evidence.
- Generated SDK compilation, downstream adoption and install-smoke evidence.
- Release signing, attestations, package publication and registry acceptance.

## Permitted description

Searchright may be described as a **source-verified, cross-platform-compiled
alpha with strong fail-closed governance, hosted admission evidence and a
functional local CLI/MCP surface**.

It must not be described as fully roadmap-complete, live-provider-proven,
methodologically validated, authenticated-hosted, production-ready,
restore-proven, independently evaluated, published or registry-accepted.

## Next sequence

1. Complete Track 24's remaining WASI, remote, durable/distributed,
   lossless-delivery, production-load and production-cache evidence without
   broadening local MCP authority.
2. Implement and adversarially test Track 34 authenticated Streamable HTTP.
3. Run authorized provider canaries and provider-policy review.
4. Execute CiteWeft/Sourceright consumer canaries and rollback rehearsals.
5. Replace track-level mappings with assertion-specific symbols, tests and
   current receipts.
6. Run sealed methodological, usability and information-specialist evaluation.
7. Complete restore, pilot, release, publication and registry evidence before
   reconsidering maturity.

The executable launch roadmap currently records 13 packages as `not_started`,
LP-002 and LP-006 as `partially_evidenced`, and LP-001 as completed. LP-002 is
bounded to the local stdio receipt and remains incomplete while its
remote/distributed/load gates are open. Track 05 provides exact hosted and sealed review receipts for
crash recovery, replay and durable deletion. LP-006 remains open because
durable retention/export effects are unavailable unless an accountable
launch-profile decision explicitly keeps them fail closed.
