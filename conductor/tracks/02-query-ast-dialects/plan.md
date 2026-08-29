# Plan: 02 Portable query AST and dialect compilers

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-02`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-02-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/evidence-search-contracts/src/query.rs`
  - [x] Present source path: `crates/evidence-search-core/src/compiler.rs`
  - [x] Present source path: `crates/evidence-search-core/src/native.rs`
  - [x] Present source path: `contracts/query-corpus/index.json`
  - [x] Present source path: `contracts/json-schema/native-search-strategy.v1.schema.json`
  - [x] Present source path: `contracts/examples/native-search-strategy.json`
  - [x] Present source path: `scripts/check_native_query_corpus.py`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Assertion ledger: `conductor/tracks/02-query-ast-dialects/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-02-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-02-phase-3 -->

- [ ] Complete semantic AST parsing and parse/compile/parse property coverage beyond the declared seven-dialect lexical subsets.
- [ ] Obtain accountable methodological and provider-currency review of real named-filter packs; the checked-in pack is structural and synthetic only.
- [ ] Obtain an accountable independent information-specialist PRESS review of the digest-bound, rights-clear conformance corpus via structured agent-panel protocol.
  - [ ] Convene a multi-agent panel simulating information specialists and PRESS methodologists.
  - [ ] Panel produces: (1) structured PRESS findings with severity-ranked observations, (2) conformance corpus coverage assessment, (3) disposition record per finding.
  - [ ] Record panel output as fixture-proven verification receipt under `verification/receipts/`.
  - [ ] Bounded claim: panel output is a simulation instrument, not a substitute for live human PRESS review; evidence level remains fixture-proven.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-02-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
