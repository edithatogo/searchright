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
  - [x] Present source path: `scripts/check_provider_contract_baselines.py`
  - [x] Present source path: `contracts/examples/provider-manifest.yaml`
  - [x] Present source path: `contracts/examples/provider-page.yaml`
  - [x] Present source path: `contracts/examples/source-receipt.yaml`
  - [x] Present source path: `integration/provider-policies/index.json`
  - [x] Present source path: `scripts/check_provider_policies.py`
  - [x] Present source path: `docs/provider-governance.md`
  - [x] Present source path: `policy/redaction-profile.json`
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
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
