# Plan: 02 Portable query AST and dialect compilers

Current status: **external_evidence_required**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.

GitHub issue key: `track-02`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-02-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/evidence-search-contracts/src/query.rs`
  - [x] Present source path: `crates/evidence-search-core/src/compiler.rs`
  - [x] Present source path: `crates/evidence-search-core/src/native.rs`
  - [x] Present source path: `crates/evidence-search-core/tests/property_tests.rs`
  - [x] Present source path: `contracts/query-corpus/index.json`
  - [x] Present source path: `contracts/query-corpus/loss-matrix.json`
  - [x] Present source path: `contracts/json-schema/native-search-strategy.v1.schema.json`
  - [x] Present source path: `contracts/examples/native-search-strategy.json`
  - [x] Present source path: `scripts/check_native_query_corpus.py`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Present source path: `crates/evidence-search-core/tests/native_adversarial_tests.rs`
  - [x] Present source path: `docs/query-normalization.md`
  - [x] Present source path: `tests/test_native_query_corpus.py`
  - [x] Assertion ledger: `conductor/tracks/02-query-ast-dialects/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-02-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_native_query_corpus.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-02-phase-3 -->

- [x] Validate semantic AST parsing and compile/parse/compile stability for the declared bounded seven-dialect subsets.
- [ ] Obtain rights-clear real named-filter expressions, exact versions and provider-current evidence; an isolated agent panel reviews them and the accountable owner records the decision. The checked-in pack remains synthetic.
- [ ] Obtain the accountable owner's disposition of the exact digest-bound corpus/loss-matrix agent-panel findings; topic-specific PRESS adequacy and empirical retrieval claims require separate protocol and outcome evidence.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-02-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `6f86326`: Expose the current semantic-conformance receipt through canonical Track 02 evidence.
  - [x] Review fix: Preserve the semantic-parser phase task identity while named-filter and owner-decision gates remain open.
  - [x] Review fix: Reject foreign dialect syntax, unknown fields, undefined references and unmodeled limits; preserve Unicode clauses and correct native heading/proximity semantics.
  - [x] Review fix: Bound parser metadata, recursion, tokens and set expansion before expensive work; add independent adversarial and recursive Unicode properties.
  - [x] Review fix: Bind each loss-matrix dialect to exactly one matching fixture; retain agent-panel and owner-decision gates without requiring a second person.
- [ ] Close only after required evidence is present and the accountable owner records the agent-panel decisions.
