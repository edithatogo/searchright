# Project status

- **Status date:** 10 September 2026
- **Observed default branch:** `main`
- **Observed main revision:** `9b4a4a4dba818d75f9dbb9ee6be458871b09a878`
- **Last fully admitted revision:** `10f983d44e67c08be44131ea6e2b7cf75dc147f6`
- **Evidence ceiling:** source-verified technical alpha with historical hosted
  admission and bounded exact-head clean-room evidence
- **Maturity decision:** not ready

The machine-readable source for this bounded observation is
`verification/receipts/project-status-snapshot.json`.

## Headline

Searchright is a substantial technical alpha. Its contract catalogue, shared
execution core, product facade, CLI, MCP server, agent skill and review-domain
services have working source implementations. A committed `Cargo.lock`,
historical cross-platform compiler evidence and an exact-head clean-room run
exist.

The observed main revision has not completed a full exact-head admission matrix.
**No complete exact-head admission matrix is claimed.** Historical workflow
results remain bound to their exact revision and do not automatically promote
later commits.

Searchright is not yet a validated systematic-review product, authenticated
hosted service, release candidate or published package. Live provider behaviour,
methodological performance, downstream cutover, usability, operational recovery,
release signing and registry acceptance remain separate evidence gates.

## Evidence boundary

The status model distinguishes three evidence surfaces:

1. **Observed source:** repository content and source-bound counts at
   `9b4a4a4dba818d75f9dbb9ee6be458871b09a878`.
2. **Bounded exact-head evidence:** clean-room reproducibility workflow run
   `34338222604`, completed successfully for that exact revision.
3. **Historical full admission:** 17 successful hosted checks for
   `10f983d44e67c08be44131ea6e2b7cf75dc147f6`, with no failed or pending result
   in the committed historical record.

The exact-head clean-room observation does not establish the current state of
CI, security, formal assurance, release readiness, live providers, downstream
consumers or external methodological evaluation.

## Remote-state reporting policy

Committed documentation does not assert volatile local or GitHub state such as:

- whether a local working tree is clean;
- whether local and remote branches are identical;
- whether pull requests or delivery branches are open;
- whether a later workflow or registry operation has completed.

Those facts must be read from GitHub or a timestamped, exact-revision receipt.
This prevents static prose from becoming an unlabelled live dashboard.

## Required environment and bootstrap disclosures

- **Rust compilation:** historical hosted compilation is evidenced for
  `10f983d44e67c08be44131ea6e2b7cf75dc147f6`; a complete compiler and admission
  matrix is not claimed for the observed main revision.
- **Live provider calls:** none is claimed by this status snapshot. Provider
  support remains bounded to fixtures and separately authorised, redacted
  canaries.
- **GitHub repository creation/push:** the public repository and its control-plane
  artefacts exist. This static document does not claim the current state of
  branches, pull requests, projects or later pushes.
- **Conductor plugin installation:** prior host receipts reported an installed
  Conductor version. This snapshot validates repository contracts only and does
  not claim current host installation or general host-version compatibility.

## Current source surface

At the observed revision, the repository contains:

- a 30-crate Rust 2024 workspace, with publication disabled by default;
- 74 JSON Schema 2020-12 contracts and canonical examples;
- 35 catalogue-backed product operations exposed through the shared interface
  model;
- neutral `evidence-search-contracts` and shared `evidence-search-core` layers;
- review planning, eligibility, query compilation, provider execution, receipts,
  audit, storage, import, deduplication, record/report/study linkage, screening,
  PRISMA/PRESS reporting, living updates and provenance foundations;
- deterministic open-provider fixtures and bounded connector implementations;
- a shared Rust facade, CLI and local stdio MCP server;
- default-deny external writes, human-only final screening authority, receipt
  redaction, package-publication gates and release/maturity blockers;
- CiteWeft and Sourceright integration passports, consumer contracts, migration
  packets, parity machinery and rollback boundaries.

Recent source work includes PubMed EFetch orchestration and the Sourceright
parity execution matrix v2. Their presence is source evidence, not proof of a
completed downstream cutover or production live-provider operation.

## Current source-bound Conductor position

The canonical source reports:

- 38 tracks: 31 active and 7 archived;
- 404 top-level tasks: 209 completed and 195 open evidence tasks;
- 93 requirements checked by the roadmap-coverage validator.

These are source-bound planning and evidence-debt counts. They are not a quality
score, release decision or claim that every completed checklist item has higher
order compiler, live, human or external evidence.

## Historical hosted admission

The last fully admitted revision in the committed record is
`10f983d44e67c08be44131ea6e2b7cf75dc147f6`. Its 17 successful hosted checks
covered:

- Rust 1.97.1 on Ubuntu, Windows and macOS;
- the declared Rust-version gate;
- formatting, Clippy, tests and documentation;
- static contracts and roadmap evidence;
- coverage admission;
- CodeQL and full-history secret scanning;
- Rust dependency, advisory, unused-dependency and cargo-vet policy;
- clean-room vendored build and install smoke;
- public API and SemVer checks;
- Kani, Miri, Loom and standard-library precondition suites;
- OpenSSF Scorecard and workflow policy.

Those checks establish compiler and repository-admission evidence for that exact
historical revision only. They do not establish current-head equivalence, live
provider correctness, methodological validity, production security, usability,
operational recovery or downstream adoption.

## Exact-head observation

GitHub Actions run `34338222604`, **Clean-room reproducibility**, completed
successfully for `9b4a4a4dba818d75f9dbb9ee6be458871b09a878` on 9 September 2026.
Its claim scope is limited to clean-room reproducibility. It does not substitute
for a full exact-head admission matrix.

## Open critical evidence domains

### Providers and methodology

- Authorised, redacted live canaries for each claimed provider.
- Completed provider terms, licence and data-handling review.
- DNS resolution and connection-pinning evidence for live endpoint security.
- Reviewed PRESS strategies and rights-cleared gold corpora.
- Sealed retrieval, translation, deduplication and prioritisation evaluation.
- Methodology, usability and information-specialist calibration.

### Interfaces, access and operations

- Complete exact-head CI, security, formal and release-candidate admission.
- Third-party MCP interoperability beyond the pinned official-client evidence.
- Authenticated Streamable HTTP with issuer, principal, tenant, region, scope,
  rate, replay, isolation and rollback evidence.
- Durable, resumable and multi-replica task state with lossless subscription
  delivery and production load/cache evidence.
- Successful encrypted backup restore, incident and cancellation rehearsals.

### Ecosystem and distribution

- CiteWeft and Sourceright producer/consumer canaries, dual-run parity, cutover
  and rollback rehearsal.
- Removal of duplicated provider implementations from downstream repositories.
- Representative persisted-data migration and backward-reader evidence.
- Generated SDK compilation, install smoke and downstream adoption evidence.
- Bounded institutional pilots and operational service-level evidence.
- Release signing, attestations, package publication and registry acceptance.

## Permitted description

Searchright may be described as a **source-verified technical alpha whose last
fully admitted revision has cross-platform compiler and repository-admission
evidence, while the observed main revision has bounded exact-head clean-room
evidence**.

It must not be described as current-head fully admitted, fully roadmap-complete,
live-provider-proven, methodologically validated, authenticated-hosted,
production-ready, restore-proven, independently evaluated, published or
registry-accepted.

## Next evidence sequence

1. Run and preserve the complete admission matrix for the exact current head.
2. Execute authorised provider canaries and provider-policy review.
3. Complete the Sourceright dual-run, parity, cutover and rollback path.
4. Replace one downstream repository's direct provider clients and retain
   regression fixtures.
5. Reconcile methodology and agent-review governance, then run sealed external
   evaluation and usability calibration.
6. Complete recovery, pilot, release, publication and registry evidence before
   reconsidering maturity.
