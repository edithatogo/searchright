# Changelog

All notable changes are recorded here. Semantic-version compatibility claims
begin only after the public API, compiler and contract-migration policies have
matching receipts.

## [Unreleased]

### Added

- Added a product-neutral `evidence-search-contracts` crate beneath
  `evidence-search-core`, with Searchright-specific review contracts kept in
  `searchright-contracts`.
- Added assertion-level roadmap traceability across 38 Conductor tracks and 199
  acceptance assertions, including explicit scaffolded, partial, source and
  external-evidence states and permitted claims.
- Added a default-deny package-publication policy: all 30 workspace crates are
  non-publishable, with only three future public-package candidates and none
  marked ready.
- Added a frozen alpha contract surface covering 60 JSON Schemas, WIT, OpenAPI
  and MCP metadata, plus configured Rust public-API and SemVer gates.
- Added source implementations and rights-clear response baselines for PubMed
  ESearch/ESummary, Europe PMC, Crossref and OpenAlex, including bounded runtime,
  retryability, total/request budgets, response-size limits and raw digests.
- Added source-preserving native strategy contracts and a seven-dialect lexical
  corpus for PubMed, Ovid MEDLINE, Embase, CINAHL, PsycINFO, Scopus and Web of
  Science, with explicit translation-loss semantics.
- Added deterministic `.srpack` review-bundle creation and verification with
  path, symlink, size, likely-secret, SHA-256, Merkle-root and tamper controls.
- Added a deterministic, noncanonical review-state reducer that binds to a
  caller-verified audit head and rejects non-human final screening authority.
- Added a rights-clear end-to-end contract reference slice spanning planning,
  native strategy, source receipts, deduplication, human screening, PRISMA
  arithmetic and deterministic bundle generation.
- Added sealed-label methodological benchmark fixtures, provider contract-drift
  checks, canonical-upstream/fork classification, licence and redistribution
  firewalls, and eleven companion-repository change packets.
- Added a cross-repository ecosystem lock, release-train promotion controls and
  a separate evidence-infrastructure portfolio Project manifest.
- Added configured `cargo-vet`, `cargo-semver-checks` and `cargo-public-api`
  assurance surfaces and expanded the network-free aggregate harness to 51
  gates, the registered gate catalogue to 53 commands and the assurance matrix
  to 42 dimensions.
- Added explicit gate evidence ceilings and a deterministic evidence-debt
  register so static, compiler, live, downstream, legal and external evidence
  cannot be silently conflated.
- Added executable architecture-fitness policy, including neutral-layer,
  network-dependency, provider-endpoint, final-authority and external-write
  boundaries. The connector crate now consumes the neutral contract layer
  directly rather than Searchright review contracts.
- Added provider terms/data-handling manifests, deterministic receipt
  minimisation tests, versioned schema migration/rollback plans and a
  network-free recovery reference rehearsal with tamper and idempotency checks.
- Added a tracked Sourceright interoperability proposal for provenance-bearing
  retraction, correction, expression-of-concern, duplicate-publication and
  version signals, with a Searchright advisory-only/human-authority boundary.

### Changed

- Replaced path-presence completion with assertion-to-symbol/test/evidence
  traceability. Track and GitHub status can no longer promote implementation or
  evidence by themselves.
- Reduced the intended public crate surface and separated the current
  development toolchain from lower shared-contract/shared-core MSRV targets.
- Reframed query translation as native source plus semantic representation and
  fidelity diagnostics rather than presumed cross-database equivalence.
- Reclassified forked integrations against canonical upstreams and made unclear
  content or code licences reference-only or review-required.
- Expanded the generated GitHub delivery Project to 13 fields and six views,
  including a dedicated implementation-gap view, while preserving the 568-node
  epic → track → phase → task hierarchy.
- Updated project status, strategy, architecture, context, hazards, decisions and
  claim boundaries to distinguish scaffolding, source behaviour, compiler proof,
  live behaviour, downstream compatibility, external validation and public
  acceptance.

### Evidence limitations

- Rust 1.97.1 and Cargo remain unavailable in the implementation environment;
  `Cargo.lock`, compilation, rustfmt, Clippy and Rust tests have not run.
- Configured coverage, mutation, fuzz, Kani, Loom, Miri, `cargo-careful`,
  `cargo-vet`, public-API and SemVer jobs have not produced execution receipts.
- No provider API, licensed source, downstream repository, GitHub remote,
  registry, pilot or external methodological evaluator was invoked.
- Provider baselines prove local fixture and expected-shape integrity only; the
  vertical slice proves contract coherence only; neither establishes search
  recall, screening validity or methodological adequacy.

## [0.1.0-alpha.1] - unpublished

- Initial technical-preview source scaffold.
