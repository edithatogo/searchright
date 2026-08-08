# Project status

**Status date:** 9 August 2026

**Evidence ceiling:** source-verified alpha

**Implementation model:** assertion-level, evidence-separated

## Headline status

Searchright is a clean Git-managed Rust workspace with the complete maturity
roadmap represented in 38 Conductor tracks and a deterministic GitHub
issue/subissue projection. The repository has been deliberately rebaselined:
roadmap scope is no longer considered implemented because a source path exists.
Each scope statement is now an acceptance assertion with an implementation
state, symbol/path mapping, deterministic gate, open evidence requirements and
permitted claim.

The network-free local harness covers 51 gates, backed by a 53-command gate
catalogue with explicit evidence ceilings. Rust compilation, live provider
execution, downstream cutover and external methodological evaluation remain
open, separately named evidence levels.

## Implemented local source and policy surfaces

- 30-crate Rust 2024 workspace, with all crates `publish = false` by default.
- Neutral `evidence-search-contracts` below the shared
  `evidence-search-core`, avoiding a Searchright-specific dependency inversion.
- Only three future public package candidates, all explicitly `publish_ready:
  false` pending compiler, SemVer, consumer and supply-chain evidence.
- 60 Draft 2020-12 schemas and 60 canonical examples.
- Exact contract-surface baseline for JSON Schema, WIT, OpenAPI and MCP server
  metadata; any byte drift requires an explicit compatibility update.
- 31 mapped operations across the shared Rust facade, CLI and MCP adapters.
- Loss-preserving native strategy representation and a seven-dialect lexical
  corpus for PubMed, Ovid MEDLINE, Embase, CINAHL, PsycINFO, Scopus and Web of
  Science. Semantic equivalence is not claimed.
- Bounded provider runtime source with typed retryability, total/per-request
  budgets, cache fingerprints and rate controls.
- Source adapters and rights-clear response baselines for PubMed ESearch and
  ESummary, Europe PMC, Crossref and OpenAlex.
- Provider-policy manifests that separately track technical endpoint identity,
  credential handling, raw-response retention, data classification and the
  absence of legal/terms approval evidence.
- Executable architecture-fitness checks that keep neutral contracts/core below
  product services, confine network dependencies and provider endpoints to the
  connector boundary, and default external writes to explicit dual opt-in.
- Deterministic receipt-redaction tests, declared schema migration/rollback
  plans, and a local recovery reference rehearsal with tamper and idempotency
  checks. None is represented as legal approval or production recoverability.
- Append-only audit ledger, single-writer filesystem store and deterministic
  derived review-state reducer. The reducer requires an externally verified
  BLAKE3 head and rejects non-human final screening authority.
- Deterministic `.srpack` review bundles with path, symlink, size, likely-secret,
  hash, Merkle-root and tamper checks.
- Review planning, eligibility, amendments, deduplication, report/study linkage,
  screening, PRISMA/PRISMA-S, living updates, provenance and governance source
  surfaces.
- Rights-clear methodological validation fixtures with sealed-label policy and
  no checked-in final test labels or performance result.
- Eight exact-revision integration passports with canonical-upstream, local-fork
  role, code/content/model licence, redistribution and drift fields.
- Eleven non-mutating companion-repository change packets and an estate scanner
  covering known direct-provider, insecure endpoint, direct-writeback and
  title-only deduplication patterns.
- The Sourceright packet now includes a provenance-bearing scholarly-integrity
  signal boundary: retractions, corrections, expressions of concern and version
  relationships remain advisory and cannot cause automatic study exclusion.
- Separate Searchright delivery Project and strategic evidence-infrastructure
  portfolio projection.
- `cargo-vet`, cargo-semver-checks and cargo-public-api policy/workflows added;
  these tools are pinned but not represented as locally executed.
- Ecosystem lock and default-deny CiteWeft → Searchright → Sourceright release
  train linked to the contract surface and package policy.

## Current measured source evidence

| Surface | Current value |
| --- | ---: |
| Conductor tracks | 38 |
| Acceptance assertions | 199 |
| Individually mapped assertions | 72 |
| MoSCoW requirements | 92 |
| Checked source tasks | 154 |
| Open higher-evidence tasks | 223 |
| Rust crates | 30 |
| Rust source files | 64 |
| Rust test functions in source | 42 |
| JSON Schemas / examples | 60 / 60 |
| CLI/MCP/facade operations | 31 |
| GitHub hierarchy nodes | 568 |
| Project fields / views | 13 / 6 |
| Integration passports / consumer interactions | 8 / 8 |
| Companion change packets / planned changes | 11 / 55 |
| Network-free aggregate gates | 51 |
| Registered gate commands | 53 |
| Assurance dimensions | 42 |
| Provider policies with approval evidence | 0 / 5 |
| Public packages marked ready | 0 |

These are source, contract and static-policy measurements. They are not a
compiler or methodological-performance result.


## Explicit non-executed disclosures

- **Live provider calls:** not executed; all provider baselines are rights-clear local fixtures.
- **GitHub repository creation/push:** not executed; remote repository, issues and Projects remain prepared projections.
- **Conductor plugin installation:** not executed because no compatible Gemini, Antigravity or Claude host executable is available locally; the checked-in installer and lifecycle artefacts remain prepared.

## Open evidence gates

### Compiler and executable evidence

- Rust 1.97.1 is not installed in this environment.
- `Cargo.lock` has not been generated or committed.
- rustfmt, Clippy, Cargo metadata/check/build/test/doc, cross-platform and MSRV
  jobs have not run locally.
- Coverage, mutation, fuzzing, Kani, Loom, Miri and `cargo-careful` remain
  configured rather than executed.
- cargo-vet, cargo-semver-checks and cargo-public-api require compiler/tool
  bootstrap and baseline receipts.

### Provider and methodological evidence

- Rust parser output has not been compared with the new provider fixtures.
- No upstream API was contacted; the response baselines detect local and
  expected-shape drift only.
- No live pagination, rate-limit, retry, cancellation or source-policy receipt
  exists.
- No independently PRESS-reviewed query corpus or gold parse has been supplied.
- No sealed-label benchmark, information-specialist calibration or external
  methods evaluation has run.

### Cross-repository and remote evidence

- CiteWeft and Sourceright have not compiled against the neutral contracts/core.
- The prepared dual-run and consumer-contract suites have not executed in the
  companion repositories.
- No custom code has been deleted from UOGTO, VOIAGE or other downstream repos.
- No Searchright remote, issue hierarchy, Project, portfolio Project, release,
  package or registry entry was created in this local pass.
- Licence review remains required for `standards_check`; `api-standards` and
  `academic-research-skills` are reference-only until reuse rights are clear.

## Claim boundary

The repository may be described as **source-verified, assertion-rebaselined and
statically validated**. It must not be described as fully implemented across its
roadmap, compiler-verified, fixture-proven end to end, live-provider compatible,
downstream-integrated, production-ready, independently validated, published or
registry-accepted.

The next safe sequence is:

1. install Rust 1.97.1 and the declared MSRV toolchains;
2. generate and commit `Cargo.lock`;
3. compile and repair the 30-crate workspace;
4. run Rust parser/runtime tests against the provider and workflow fixtures;
5. capture SemVer/public-API/supply-chain receipts;
6. execute CiteWeft and Sourceright consumer canaries;
7. only then migrate downstream custom code or make public readiness claims.

`CODEX_HANDOFF.md` remains the remote/bootstrap contract. The portable review
bundle and complete Git delivery are separate artefacts: `.srpack` packages a
review; the delivery ZIP packages this repository and its history.
