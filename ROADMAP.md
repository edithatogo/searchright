# Roadmap

## Horizon 0 — Foundation and executable contracts

- Establish shared-core architecture, Conductor context and repository standards.
- Stabilise review-plan/query/receipt/audit/screening/PRISMA schemas.
- Compile and pass strict deterministic gates.

## Horizon 1 — MVP

- PubMed, Europe PMC, Crossref, OpenAlex and import-only RIS/CSL/CSV connectors.
- Query translation, execution receipts, local append-only store and deduplication.
- Dual-reviewer screening, exclusion reasons and conflict resolution.
- CLI, MCP stdio server and systematic-search agent skill.
- PRISMA-S appendix and PRISMA 2020 flow outputs.
- Sourceright adoption of `evidence-search-core`.

## Horizon 2 — Public beta

- Trial registries, repositories, grey-literature and citation-chaining adapters.
- PRESS workflow, gold-set recall testing and strategy regression snapshots.
- MCP Streamable HTTP, OAuth, tasks, subscriptions and response caching.
- Signed binaries, OCI image, SBOM/provenance and registry submissions.
- SYNERGY-based screening benchmark and human calibration pilot.

## Horizon 3 — Product maturity

- Licensed BYO-access adapters for Embase, Scopus and Web of Science.
- Search-dialect conformance corpus and cross-platform translation testing.
- Record-to-report-to-study graph, active-learning prioritisation and living-search
  updates with amendment tracking.
- WASI component provider SDK and signed capability manifests.
- Review-system and citation-manager adapters.

## Horizon 4 — Bleeding-edge research platform

- Agent teams for strategy generation, adversarial PRESS review and update
  surveillance with calibrated uncertainty.
- Retrieval simulation, metamorphic search tests and sensitivity/precision
  estimation against benchmark corpora.
- Privacy-preserving institutional deployment and federated review artefacts.
- Formal contract refinement and model checking for workflow invariants.
- External methodological evaluation and software paper.

Detailed sequencing, acceptance criteria and evidence gates are in
`conductor/tracks.md` and each track's `spec.md`/`plan.md`.
