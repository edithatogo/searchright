# Plan: 04 Open provider connectors MVP

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-04`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-04-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-connectors/src/lib.rs`
  - [x] Present source path: `integration/provider-contract-baselines.json`
  - [x] Present source path: `provider-fixtures/mvp/pubmed-esearch.json`
  - [x] Present source path: `provider-fixtures/mvp/pubmed-esummary.json`
  - [x] Present source path: `provider-fixtures/mvp/europe-pmc.json`
  - [x] Present source path: `provider-fixtures/mvp/crossref.json`
  - [x] Present source path: `provider-fixtures/mvp/openalex.json`
  - [x] Present source path: `crates/searchright-connectors/src/efetch.rs`
  - [x] Present source path: `crates/searchright-connectors/tests/efetch_offline.rs`
  - [x] Present source path: `crates/searchright-connectors/tests/runtime_receipts.rs`
  - [x] Present source path: `crates/searchright-connectors/tests/fixtures/pubmed-efetch.xml`
  - [x] Present source path: `crates/searchright-connectors/tests/fixtures/pubmed-efetch-page.json`
  - [x] Present source path: `tests/test_provider_contract_baselines.py`
  - [x] Present source path: `docs/pubmed-efetch-mapping.md`
  - [x] Present source path: `crates/searchright-connectors/tests/parser_integrity.rs`
  - [x] Present source path: `crates/searchright-connectors/tests/canonical_pages.rs`
  - [x] Present source path: `crates/searchright-connectors/tests/fixtures`
  - [x] Present source path: `docs/provider-parser-integrity.md`
  - [x] Present source path: `scripts/check_provider_contract_baselines.py`
  - [x] Present source path: `contracts/examples/provider-manifest.yaml`
  - [x] Present source path: `contracts/examples/provider-page.yaml`
  - [x] Present source path: `contracts/examples/source-receipt.yaml`
  - [x] Present source path: `integration/provider-policies/index.json`
  - [x] Present source path: `scripts/check_provider_policies.py`
  - [x] Present source path: `docs/provider-governance.md`
  - [x] Present source path: `policy/redaction-profile.json`
  - [x] Present source path: `crates/searchright-connectors/tests/cache_parser_version.rs`
  - [x] Assertion ledger: `conductor/tracks/04-open-connectors-mvp/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-04-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_provider_contract_baselines.py`
  - [x] `python scripts/check_provider_policies.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-04-phase-3 -->

- [ ] Run authorised, redacted live smokes for each advertised provider.
- [ ] Verify upstream terms, rate limits and response changes at release time.
- [ ] Review current provider terms, privacy, authentication, redistribution and rate policy before any live-support promotion.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-04-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `3343d6a86b500e993f34ebe5e6518172ad7876b5`: Parser integrity: preserve stable PubMed/Crossref IDs, correct Europe PMC/OpenAlex identity forward-only, reject incomplete/malformed pages and compare complete synthetic goldens.
  - Review fix `d9d1da3603da8ddc3c0c75f755fa6179f25ad89b`: Bounded offline EFetch citation/abstract parser and request builder, exact PMID reconciliation, structured metadata, complete XML golden and static baseline regressions; no live switch.
  - Review fix `d13cbca321d4372b3a52ecef3f8c8e801472e864`: Actual fixture-runtime receipt binding, memory-cache replay/corruption checks and budget visibility; seven runtime tests pass after the separately delivered Track03 budget fix.
  - Review fix `d7854b0956eed7ec3730ab3219b358d5f1964e77`: Partition all four JSON adapters' normalized cache entries with PROVIDER_PARSER_VERSION; retain legacy FixtureProvider construction and add explicit with_version declaration. Preserve historical entries without re-keying or migration; observed old-cache bypass, then 61 focused Homebrew tests/Clippy and full pinned 1.97.1 snapshot 6d995108106db62474e2746a2021f29e89603b95 passed.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
