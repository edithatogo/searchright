# Requirements Contract

## Contract rules

- Requirements are versioned and owned by Conductor tracks.
- The current repository is the source of truth for implementation status.
- A requirement advances only with named evidence; documentation cannot promote
  its own evidence level.
- Reporting-standard support is distinct from methodological conduct.
- Canonical records are never silently overwritten by provider or agent output.
- Live/network/write operations are disabled or previewed until explicitly
  approved and audit logged.
- External publication, registry acceptance and remote migration require public
  evidence; prepared packets are not acceptance.

## MoSCoW matrix

| ID | MoSCoW | Requirement | Track owner | Completion contract | Evidence | Overclaim guard |
| --- | --- | --- | --- | --- | --- | --- |
| SR-001 | Must | Canonical review plan | 01 | Review kind, question framework, objectives, eligibility, sources, protocol and governance validate against versioned schemas. | Schema examples plus semantic tests. | Do not call a prose protocol executable until it validates. |
| SR-002 | Must | PICO/PCC and extensible question frameworks | 01 | PICO, PECOS, PCC, SPIDER, PEO and named custom frameworks preserve labels and non-empty values. | Contract tests. | Framework structure does not prove a well-framed question. |
| SR-003 | Must | Explicit eligibility and versioning | 01 | Inclusion/exclusion rules have stable IDs, operational rules, rationale, stage, priority and amendment version. | Schema and workflow tests. | Do not let agents silently amend criteria. |
| SR-004 | Must | Information-source and platform identity | 01 | Database/resource and platform are distinct, versioned and reportable. | Plan examples and PRISMA-S checks. | Do not collapse MEDLINE, PubMed and Ovid into one label. |
| SR-005 | Must | Portable query AST | 02 | Boolean, NOT, proximity, fields, phrases, truncation and controlled vocabulary are represented losslessly where possible. | Unit/property/schema tests. | Portable AST is not universal source equivalence. |
| SR-006 | Must | Deterministic dialect compilation | 02 | Given the same contract/compiler version, output and hash are identical. | Snapshots and property tests. | Do not hide lossy translations. |
| SR-007 | Must | Translation warnings and review gate | 02 | Unsupported fields, proximity, filters and limits emit stable review-required warnings. | Dialect conformance corpus. | Warnings are not peer review. |
| SR-008 | Must | Shared provider runtime | 03 | Provider registration, bounded pagination, timeout, rate policy, mode and redacted receipts are product-neutral. | Core integration tests. | Do not duplicate the runtime downstream. |
| SR-009 | Must | Sourceright compatibility boundary | 03,12 | Sourceright fixtures pass through old and shared runtimes before cutover. | Parity report and rollback plan. | Do not claim replacement before remote integration. |
| SR-010 | Must | Network disabled by default | 03,14 | Default build and policy cannot issue live requests. | Negative integration tests. | A compiled live feature is not execution approval. |
| SR-011 | Must | Provider host/egress controls | 03,14 | Adapters declare hosts; runtime blocks undeclared, local, link-local and metadata endpoints. | SSRF/adversarial tests. | Allowlisting does not certify an upstream service. |
| SR-012 | Must | Source receipts | 05 | Every source execution records source/platform, query hash, time, mode, pagination, counts, policy and warnings without secrets. | Receipt schema and replay tests. | A receipt records execution, not comprehensiveness. |
| SR-013 | Must | Append-only tamper-evident audit | 05 | Material events form a verifiable BLAKE3 chain with actor/tool provenance. | Tamper tests and persistence round trip. | Hash chains do not prove external truth. |
| SR-014 | Must | Replay and fixture modes | 04,05 | Default tests reproduce provider pages without network access. | Checked-in fixtures. | Fixture support is not live-provider proof. |
| SR-015 | Must | PubMed and Europe PMC MVP | 04 | Both have source-specific compiler targets, fixtures and endpoint contracts; live status is separately evidenced. | Fixture tests and opt-in smoke receipts. | Do not call PubMed fully supported from ESearch-only work. |
| SR-016 | Should | Crossref and OpenAlex discovery adapters | 04 | Adapters identify discovery limitations and return normalised records. | Fixtures and optional live smokes. | Discovery APIs are not substitutes for all bibliographic databases. |
| SR-017 | Must | Record/report/study separation | 01,06 | Contracts distinguish provider records, reports and underlying studies and retain linkage evidence. | Graph invariants and fixtures. | Do not count reports as studies. |
| SR-018 | Must | Deterministic deduplication | 06 | Exact identifiers and conservative fuzzy matching create reviewable clusters without deleting source records. | Unit/property/metamorphic tests. | Proposed duplicates require review where ambiguous. |
| SR-019 | Must | Import/export interoperability | 06 | RIS, CSL JSON, PubMed XML/nbib and CSV imports preserve provenance; exports are deterministic. | Golden fixtures and round trips. | Import success is not semantic completeness. |
| SR-020 | Must | Dual screening phases | 07 | Title/abstract and full text support independent decisions, conflicts and adjudication. | State-machine tests. | Software does not replace reviewer judgement. |
| SR-021 | Must | Structured full-text exclusion reasons | 07 | Each excluded full text has one primary reason tied to the eligibility version and criterion. | Contract and PRISMA tests. | Reasons must reflect reviewer judgement, not inferred labels. |
| SR-022 | Must | Agent authority policy | 07,11 | Agent exclusions are denied by default; authority is role- and calibration-bound. | Negative tests. | No autonomous final exclusion claim. |
| SR-023 | Must | PRISMA 2020 flow arithmetic | 08 | Counts reconcile and invalid arithmetic blocks generation. | Invariant/property tests. | Valid arithmetic is not PRISMA endorsement. |
| SR-024 | Must | PRISMA-S 16-item ledger | 08 | All 16 reporting items map to evidence, status, gaps and locations. | Ledger fixture. | PRISMA-S is reporting, not conduct certification. |
| SR-025 | Should | PRISMA-ScR and PRISMA-LSR profiles | 08,20 | Review-kind profiles add relevant reporting/update requirements. | Profile fixtures. | Profiles do not guarantee journal acceptance. |
| SR-026 | Must | PRESS peer-review workflow | 08,11 | Six PRESS domains, reviewer identity, findings, response and approval are versioned. | Contract and workflow tests. | Automated lint is not independent PRESS review. |
| SR-027 | Must | Full strategy appendix | 08 | At least one complete major-database strategy and all source-specific syntax/dates/limits are rendered. | Snapshot tests. | Appendix generation does not validate term choice. |
| SR-028 | Must | CLI parity | 09 | CLI delegates to facade and emits stable JSON/errors for core operations. | CLI snapshot and install tests. | Do not advertise unimplemented commands. |
| SR-029 | Must | MCP stdio parity | 10 | MCP tools delegate to facade, use typed schemas, expose effects/authority and pass protocol transcripts. | MCP conformance tests. | Local stdio is not hosted MCP. |
| SR-030 | Should | MCP 2026-07-28 advanced capabilities | 22 | Discovery, tasks, subscriptions, caching and Streamable HTTP are added with auth/threat model. | Protocol compatibility matrix. | Do not break 2025-11-25 clients silently. |
| SR-031 | Must | Systematic-search agent skill | 11 | Skill covers planning, PRESS, execution, dedup, screening and reporting with explicit checkpoints. | Skill lint and scenario tests. | Skill does not confer database access or reviewer authority. |
| SR-032 | Must | Estate replacement manifest | 13 | Every custom search implementation has owner, evidence, replacement target, status and deletion gate. | Code-search inventory and PR links. | Do not bulk-delete based on approximate matches. |
| SR-033 | Must | Strict quality harness | 14 | Formatting, lint, typing/schema, unit/integration/e2e/property/mutation/fuzz/metamorphic/DST/CDC and agent tests are tracked. | CI artefacts. | Coverage percentage alone is not correctness. |
| SR-034 | Must | Coverage above 90 percent | 14 | Line coverage gate is >90% with branch/function reporting and justified exclusions. | llvm-cov artefact/Codecov. | Do not report coverage until a current run exists. |
| SR-035 | Must | Security engineering | 14 | Threat model, CodeQL, cargo-deny/audit, zizmor, Scorecard, SBOM, provenance, secret and SSRF tests pass. | Security receipts. | A clean scan does not mean no vulnerabilities. |
| SR-036 | Must | Dependency automation | 14 | Renovate plus GitHub alerts use bounded, grouped updates and automerge only after stable gates. | Configuration and observed runs. | Configuration is not evidence of applied updates. |
| SR-037 | Must | Benchmark and calibration corpus | 15 | SYNERGY and synthetic dialect/audit fixtures produce versioned metrics with leakage controls. | Benchmark report schema and runs. | Do not claim SOTA without external comparison. |
| SR-038 | Should | Recall-oriented gold-set testing | 15 | Known relevant records are used to measure strategy sensitivity and regressions. | Gold-set protocol and metrics. | Gold sets remain topic-specific. |
| SR-039 | Must | Signed reproducible release | 16 | Locked builds, platform binaries, checksums, SBOM and provenance are generated and install-smoked. | Release workflow artefacts. | Prepared workflows are not published releases. |
| SR-040 | Should | Registry publication | 17 | Official MCP Registry, Glama, Smithery, GitHub Releases and crates.io packets are validated and submitted after approval. | Public listing URL/version/date. | Prepared metadata is not acceptance. |
| SR-041 | Could | JOSS/software paper | 17,23 | Rights, benchmark, statement of need, citations and archived release satisfy submission checks. | Submission packet and DOI. | Do not claim submitted or accepted prematurely. |
| SR-042 | Should | Grey literature and registers | 18 | Trial registers, repositories, websites, conference and organisational sources have explicit adapters or documented manual methods. | Source-specific fixtures. | No unauthorised scraping. |
| SR-043 | Should | Citation chaining and contacts | 18 | Backward/forward citation search and contact logs are reproducible and reported. | Audit/event fixtures. | Citation-network expansion is not exhaustive. |
| SR-044 | Could | Licensed BYO-access adapters | 19 | Embase, Scopus and Web of Science adapters separate credentials, licences, caches and terms. | Contract fixtures and user-run smokes. | No bundled or circumvented access. |
| SR-045 | Should | Living review updates | 20 | Search dates, prior work, amendments, dedup against prior runs and update cadence are first-class. | Update scenario tests. | Surveillance alerts are not automatic inclusion. |
| SR-046 | Could | Active-learning prioritisation | 21 | Ranking is calibration-tested, uncertainty-bearing and cannot discard unseen records by default. | SYNERGY/human calibration. | No autonomous exclusion claim. |
| SR-047 | Could | WASI provider ecosystem | 22 | Signed components obey WIT capabilities, resource budgets and sandbox policy. | Component conformance suite. | Third-party plugins are not endorsed by listing. |
| SR-048 | Must | Data minimisation and retention | 05,14 | Audit and screening stores avoid unnecessary full text/PII and enforce declared retention/export/deletion policy. | Privacy tests and policy. | Do not promise legal compliance without deployment review. |
| SR-049 | Must | Protocol amendments and deviations | 01,20 | Changes are versioned, explained, dated and linked to affected runs/decisions. | Amendment scenarios. | Never rewrite historical contracts in place. |
| SR-050 | Won’t now | One-click autonomous systematic review | 23 | Explicitly excluded before mature external validation and governance. | Non-goal tests/docs. | Never market end-to-end autonomy as current capability. |

## Evidence levels

| Level | Meaning | Minimum proof |
| --- | --- | --- |
| Contracted | Type/schema/spec exists. | Requirement row, schema and track. |
| Scaffolded | Source/interface exists. | Static validation and explicit blocker. |
| Fixture-backed | Deterministic behavior runs. | Passing fixture/contract test receipt. |
| Opt-in live proven | Authorised external behavior runs. | Redacted timestamped smoke receipt. |
| Publicly accepted | External system lists the artefact. | Public URL, version and date. |
