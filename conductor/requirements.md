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
| SR-009 | Must | Sourceright compatibility boundary | 03,14 | Sourceright fixtures pass through old and shared runtimes before cutover. | Parity report and rollback plan. | Do not claim replacement before remote integration. |
| SR-010 | Must | Network disabled by default | 03,16 | Default build and policy cannot issue live requests. | Negative integration tests. | A compiled live feature is not execution approval. |
| SR-011 | Must | Provider host/egress controls | 03,16 | Adapters declare hosts; runtime blocks undeclared, local, link-local and metadata endpoints. | SSRF/adversarial tests. | Allowlisting does not certify an upstream service. |
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
| SR-025 | Should | PRISMA-ScR and PRISMA-LSR profiles | 08,22 | Review-kind profiles add relevant reporting/update requirements. | Profile fixtures. | Profiles do not guarantee journal acceptance. |
| SR-026 | Must | PRESS peer-review workflow | 08,11 | Six PRESS domains, reviewer identity, findings, response and approval are versioned. | Contract and workflow tests. | Automated lint is not independent PRESS review. |
| SR-027 | Must | Full strategy appendix | 08 | At least one complete major-database strategy and all source-specific syntax/dates/limits are rendered. | Snapshot tests. | Appendix generation does not validate term choice. |
| SR-028 | Must | CLI parity | 09 | CLI delegates to facade and emits stable JSON/errors for core operations. | CLI snapshot and install tests. | Do not advertise unimplemented commands. |
| SR-029 | Must | MCP stdio parity | 10 | MCP tools delegate to facade, use typed schemas, expose effects/authority and pass protocol transcripts. | MCP conformance tests. | Local stdio is not hosted MCP. |
| SR-030 | Should | MCP 2026-07-28 advanced capabilities | 24 | Discovery, tasks, subscriptions, caching and Streamable HTTP are added with auth/threat model. | Protocol compatibility matrix. | Do not break 2025-11-25 clients silently. |
| SR-031 | Must | Systematic-search agent skill | 11 | Skill covers planning, PRESS, execution, dedup, screening and reporting with explicit checkpoints. | Skill lint and scenario tests. | Skill does not confer database access or reviewer authority. |
| SR-032 | Must | Estate replacement manifest | 15 | Every custom search implementation has owner, evidence, replacement target, status and deletion gate. | Code-search inventory and PR links. | Do not bulk-delete based on approximate matches. |
| SR-033 | Must | Strict quality harness | 16 | Formatting, lint, typing/schema, unit/integration/e2e/property/mutation/fuzz/metamorphic/DST/CDC and agent tests are tracked. | CI artefacts. | Coverage percentage alone is not correctness. |
| SR-034 | Must | Coverage above 90 percent | 16 | Line coverage gate is >90% with branch/function reporting and justified exclusions. | llvm-cov artefact/Codecov. | Do not report coverage until a current run exists. |
| SR-035 | Must | Security engineering | 16 | Threat model, CodeQL, cargo-deny/audit, zizmor, Scorecard, SBOM, provenance, secret and SSRF tests pass. | Security receipts. | A clean scan does not mean no vulnerabilities. |
| SR-036 | Must | Dependency automation | 16 | Renovate plus GitHub alerts use bounded, grouped updates and automerge only after stable gates. | Configuration and observed runs. | Configuration is not evidence of applied updates. |
| SR-037 | Must | Benchmark and calibration corpus | 17 | SYNERGY and synthetic dialect/audit fixtures produce versioned metrics with leakage controls. | Benchmark report schema and runs. | Do not claim SOTA without external comparison. |
| SR-038 | Should | Recall-oriented gold-set testing | 17 | Known relevant records are used to measure strategy sensitivity and regressions. | Gold-set protocol and metrics. | Gold sets remain topic-specific. |
| SR-039 | Must | Signed reproducible release | 18 | Locked builds, platform binaries, checksums, SBOM and provenance are generated and install-smoked. | Release workflow artefacts. | Prepared workflows are not published releases. |
| SR-040 | Should | Registry publication | 19 | Official MCP Registry, Glama, Smithery, GitHub Releases and crates.io packets are validated and submitted after approval. | Public listing URL/version/date. | Prepared metadata is not acceptance. |
| SR-041 | Could | JOSS/software paper | 19,29 | Rights, benchmark, statement of need, citations and archived release satisfy submission checks. | Submission packet and DOI. | Do not claim submitted or accepted prematurely. |
| SR-042 | Should | Grey literature and registers | 20 | Trial registers, repositories, websites, conference and organisational sources have explicit adapters or documented manual methods. | Source-specific fixtures. | No unauthorised scraping. |
| SR-043 | Should | Citation chaining and contacts | 20 | Backward/forward citation search and contact logs are reproducible and reported. | Audit/event fixtures. | Citation-network expansion is not exhaustive. |
| SR-044 | Could | Licensed BYO-access adapters | 21 | Embase, Scopus and Web of Science adapters separate credentials, licences, caches and terms. | Contract fixtures and user-run smokes. | No bundled or circumvented access. |
| SR-045 | Should | Living review updates | 22 | Search dates, prior work, amendments, dedup against prior runs and update cadence are first-class. | Update scenario tests. | Surveillance alerts are not automatic inclusion. |
| SR-046 | Could | Active-learning prioritisation | 23 | Ranking is calibration-tested, uncertainty-bearing and cannot discard unseen records by default. | SYNERGY/human calibration. | No autonomous exclusion claim. |
| SR-047 | Could | WASI provider ecosystem | 24 | Signed components obey WIT capabilities, resource budgets and sandbox policy. | Component conformance suite. | Third-party plugins are not endorsed by listing. |
| SR-048 | Must | Data minimisation and retention | 05,16 | Audit and screening stores avoid unnecessary full text/PII and enforce declared retention/export/deletion policy. | Privacy tests and policy. | Do not promise legal compliance without deployment review. |
| SR-049 | Must | Protocol amendments and deviations | 01,22 | Changes are versioned, explained, dated and linked to affected runs/decisions. | Amendment scenarios. | Never rewrite historical contracts in place. |
| SR-050 | Won’t now | One-click autonomous systematic review | 30 | Explicitly excluded before mature external validation and governance. | Non-goal tests/docs. | Never market end-to-end autonomy as current capability. |


| SR-051 | Must | Research-object provenance exports | 25 | RO-Crate and W3C PROV exports preserve identifiers, entities, activities, agents and source-receipt links deterministically. | Golden export fixtures and schema checks. | Provenance records process lineage; it does not establish scientific truth. |
| SR-052 | Must | Immutable review and update lineage | 22,25 | Reviews, runs, amendments and update deltas retain stable lineage identifiers and never rewrite prior evidence. | Living-update and provenance scenarios. | A lineage graph is not evidence that every source was searched. |
| SR-053 | Must | Formal workflow assurance | 26 | Consequential workflow transitions, authority limits and irreversible actions are represented as executable finite-state invariants. | Model-check traces and forbidden-transition tests. | Source-level state models are not formal proof until the checker runs. |
| SR-054 | Should | Contract evolution and compatibility | 26 | Every public schema/interface change declares compatibility, migration, deprecation and rollback behavior. | Compatibility catalogue and contract tests. | Version labels alone do not prove backward compatibility. |
| SR-055 | Must | Accessible deterministic diagnostics | 27 | CLI, MCP and library diagnostics support stable plain text, JSON and JSONL without relying on colour, animation or terminal width. | Snapshot/accessibility tests. | Accessible output formats do not certify every host UI. |
| SR-056 | Should | Internationalisation-ready contracts | 27 | User-facing messages use stable codes and separable locale text; identifiers and audit semantics remain locale-neutral. | Locale fixtures and message-catalogue checks. | Internationalisation readiness is not a claim of translated coverage. |
| SR-057 | Must | Institutional data-governance policy | 28 | Data classification, purpose, retention, storage, export, deletion and human approval are evaluated before sensitive operations. | Governance-decision scenarios and negative tests. | Policy evaluation does not constitute legal or privacy compliance advice. |
| SR-058 | Should | Multi-reviewer collaboration and least privilege | 07,28 | Reviewer roles, assignments, conflicts, adjudication and write authority are explicit, auditable and minimally privileged. | Role/authority scenario tests. | Collaboration support does not replace research-team governance. |
| SR-059 | Must | External methodological evaluation | 29 | Independent information specialists evaluate recall, translation fidelity, usability and reporting against preregistered protocols. | External protocol, blinded results and response matrix. | Internal benchmark results are not external validation. |
| SR-060 | Should | Sustainable ecosystem governance | 29 | Deprecation, contribution, maintenance, security response, standards surveillance and succession are documented and evidence-tracked. | Governance receipts and observed release history. | Documents alone do not prove a sustainable community. |
| SR-061 | Must | Mature 1.0 evidence gate | 30 | Version 1.0 requires compiler, fixture, live-provider, interoperability, security, migration, usability and external-evaluation evidence with no critical blocker. | Signed maturity dossier and release decision. | Source completeness must never be described as product maturity. |
| SR-062 | Could | Privacy-preserving federated review artefacts | 28,29 | Cross-institution artefact exchange minimises data, preserves provenance and separates local decisions from shared metadata. | Threat model and multi-site pilot. | No federated or privacy-preserving deployment claim before a real pilot. |

| SR-063 | Must | CiteWeft extraction boundary | 12 | CiteWeft output is adapted one-way into neutral document evidence with spans, uncertainty, diagnostics and pinned provenance. | Adapter fixtures, schema checks and dependency-boundary tests. | CiteWeft is GROBID-inspired, not a GROBID reimplementation or canonical citation source. |
| SR-064 | Should | Source-grounded document evidence | 12 | Reference fields, callouts and minimal source spans support full-text screening and study linkage without retaining whole documents by default. | Golden span/diagnostic fixtures and retention-negative tests. | Extracted fields remain review evidence, not bibliographic truth. |
| SR-065 | Must | Pinned integration passports | 13 | Each repository integration declares exact upstream revision, contracts, dependency direction, feature flags, verification, rollback and claim boundary. | Passport schema, lock manifest and consumer-driven contract receipts. | A pin and passport do not prove downstream compatibility until tests run. |
| SR-066 | Must | Conductor-to-GitHub issue hierarchy | 13 | The roadmap epic contains track issues and each Conductor plan phase maps deterministically to a native GitHub subissue with idempotent dry-run-first sync. | Rendered issue manifest, hierarchy checker and approved sync receipt. | Prepared issue bodies are not remote GitHub issues. |
| SR-067 | Must | Canonical context and claim spine | 13,16 | Agents and humans consume one versioned manifest of product context, decisions, hazards, capabilities, evidence levels and public claim boundaries. | Context-integrity and drift checks. | Context documents cannot promote their own evidence level. |
| SR-068 | Must | Clean-room build and artifact attestation | 16,18 | Fresh-clone builds, install smokes, MCP transcripts, deterministic archives, SBOMs and signed attestations are generated from locked sources. | Clean-room and release workflow receipts. | Configured workflows are not executed attestations. |
| SR-069 | Should | Maximal adversarial and formal harness | 16,26 | Miri, Kani, cargo-careful, Loom/model checks, fuzzing, mutation, fault injection, SSRF and hostile-content cases cover consequential boundaries. | Scheduled CI artefacts and triaged survivors/findings. | Tool configuration is not proof that all defects are absent. |
| SR-070 | Should | Integration drift surveillance | 13,15 | Scheduled read-only checks compare pinned upstream revisions and public contracts, opening review work without automatic migration. | Drift receipts and reviewed update issues. | Drift detection must not silently update dependencies or claims. |


## Evidence levels

| Level | Meaning | Minimum proof |
| --- | --- | --- |
| Contracted | Type/schema/spec exists. | Requirement row, schema and track. |
| Source-verified | Source, contracts, docs and static policy gates agree. | Reproducible static-harness receipt. |
| Compiler-verified | Rust compiles, lints and tests on supported targets. | Current Cargo/CI artefacts and committed lockfile. |
| Fixture-proven | Deterministic behavior runs against versioned fixtures. | Passing fixture/contract/interop receipt. |
| Opt-in live proven | Authorised external behavior runs. | Redacted timestamped smoke receipt. |
| Externally validated | Independent evaluators execute a preregistered protocol. | Public protocol, results and response matrix. |
| Publicly accepted | External system lists or accepts the artefact. | Public URL, version and date. |
