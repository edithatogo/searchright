# Project status

**Status date:** 8 August 2026

**Evidence level:** source-verified alpha

## Headline status

Searchright has a broad contract-first source implementation covering the entire
roadmap. It is managed in Git and organised into 38 evidence-aware Conductor
tracks through final version 1.0 decision. The network-free harness passes its individual source gates; compiler,
live, downstream and external-evaluation gates remain separate and open where
this environment could not execute them.

## Source implementation completed

- 29-crate Rust 2024 workspace with a shared application facade, CLI and stdio
  MCP server.
- CiteWeft isolated behind a one-way, non-publishable leaf adapter and a neutral
  `DocumentEvidence` contract.
- 49 Draft 2020-12 schemas, 49 canonical examples and a complete schema
  catalogue.
- 31 mapped operations across Rust facade, CLI and MCP.
- Review planning, protocol amendments, standards packs and evidence-linked
  assessments.
- Portable query AST, source dialects, explicit translation fidelity and loss
  approval.
- Bounded fixture/live/replay provider architecture with host, budget, retry,
  cache and receipt contracts.
- Append-only hash-linked audit events and crash-conscious single-writer
  snapshots.
- Deterministic import/export, conservative deduplication and record–report–study
  linkage.
- Human-governed title/abstract and full-text screening with conflicts and
  explicit exclusion reasons.
- PRISMA arithmetic, Mermaid flow rendering, PRISMA-S reporting evidence, PRESS
  and seed-set validation.
- Living-review lineage, supplementary discovery, explainable ranking and
  calibration contracts.
- RO-Crate/W3C PROV-style exports, accessible diagnostics and institutional
  governance decisions.
- Kani proof harnesses, Loom concurrency models, Miri/`cargo-careful` jobs and
  three cargo-fuzz targets.
- Eight exact-revision integration passports, eight consumer-contract
  interactions and read-only drift surveillance.
- Sourceright symbol/parity/cutover packet and a 15-repository estate migration
  inventory.
- One roadmap epic, 38 track issues, 152 phase subissues and 373 task
  subissues, with dry-run-first idempotent issue and Project synchronisation.
- Declarative GitHub repository settings, main ruleset, Project v2 fields/views
  and one-command remote bootstrap controller.
- Authenticated remote-MCP, tenancy, component-health, telemetry, backup and
  incident contracts with default-deny authority.
- Cross-repository release train, release-candidate rehearsal, pilot protocol
  and final maturity decision track.
- CI, CodeQL, dependency review, cargo-deny/audit/machete, Scorecard, Gitleaks,
  Codecov, coverage, mutation, SBOM, clean-room builds, reproducible packaging
  and build attestations.
- Registry/JOSS packets held at prepared-not-submitted status.

## Current source evidence

The source tree currently contains and checks:

- 49 schemas and 49 schema examples;
- 38 Conductor tracks and 92 MoSCoW requirements;
- 190 checked plan tasks and 183 explicit open evidence tasks;
- 29 crate manifests and 62 Rust source files;
- 31 CLI/MCP/facade operations;
- 8 integration passports and 8 consumer-contract interactions;
- 564 prepared GitHub hierarchy nodes: one epic, 38 tracks, 152 phases and
  373 tasks;
- 24 mapped Sourceright symbols, 7 parity cases and 10 parity dimensions;
- 58 immutable GitHub Action references;
- a 28-dimension assurance matrix and 30-command aggregate static harness;
- source-component SBOM and deterministic source-package consistency.

These checks are lexical, structural and contract-semantic. They are not a Rust
compiler substitute.

## Open evidence gates

### Local environment blockers

- Rust compilation has not yet been evidenced in this session.
- `Cargo.lock` has not yet been generated and committed.
- Formatting, Clippy, compilation, unit/integration/property/metamorphic tests,
  documentation, coverage, mutation, fuzzing, Kani, Loom, Miri and
  `cargo-careful` have source/configuration but no local execution receipt.

### Integration blockers

- No producer and consumer repository have both executed the prepared
  consumer-contract suite.
- No dual-run was executed in a checked-out Sourceright repository.
- No custom search code was removed from downstream repositories.
- **Live provider calls** were not executed; no PubMed, Europe PMC, Crossref,
  OpenAlex or ClinicalTrials.gov receipt was produced.
- Licensed platforms require user-provided access, terms review and redacted live
  evidence.
- Authenticated remote HTTP MCP, tenancy and WASI runtime conformance remain
  higher-evidence work; only source contracts and policies are present.
- No operational health/SLO run, backup restore drill, incident exercise,
  downstream release-train canary, release rehearsal or institutional pilot has
  been executed.
- Conductor plugin installation was not performed because no compatible host
  executable was present; host-aware installers and setup artefacts are checked
  in.

### Human and external blockers

- PRESS review and usability testing by information specialists.
- Human calibration of advisory ranking and agent recommendations.
- Independent methodological, security and privacy evaluation.
- Remote GitHub repository creation/push, issue/Project/control-plane apply, signed release,
  crates.io publication and registry submission were not performed.
- Acceptance by the official MCP Registry, Glama, Smithery, JOSS or any journal.

## Git and claim boundary

Git is the local source of truth. Generated Conductor evidence, context locks and
GitHub issue bodies are reproducible projections from committed source data.
The source may be described as **source-implemented and statically validated**.
It must not yet be described as compiler-verified, production-ready,
live-provider proven, downstream-compatible, registry-listed, independently
validated or as having replaced downstream code.

The next safe gate is to generate and commit `Cargo.lock`, run the full
cross-platform compiler/test/security harness, then execute producer–consumer
fixtures and Sourceright dual-run parity before any deletion or public release
claim.

## Delivery handoff

`CODEX_HANDOFF.md` is the single execution contract for compiler verification,
remote creation, Project synchronisation and CI closure.
`scripts/package_complete_delivery.py` creates and verifies the full Git ZIP,
source archives, Git bundle, checksums and delivery receipt from a clean,
remote-free repository.
