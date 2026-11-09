# Plan: 39 Formal BDD/SMT query equivalence and semantic loss calculus

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-39`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-39-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `conductor/tracks/39-smt-query-equivalence-calculus/spec.md`
  - [x] Present source path: `crates/evidence-search-contracts/src/query.rs`
  - [x] Present source path: `crates/evidence-search-core/src/compiler.rs`
  - [x] Present source path: `crates/evidence-search-core/src/native.rs`
  - [x] Present source path: `crates/evidence-search-core/src/lib.rs`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Assertion ledger: `conductor/tracks/39-smt-query-equivalence-calculus/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-39-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/check_traceability.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-39-phase-3 -->

- [ ] Define SemanticRelation, CounterexampleWitness, and QueryEquivalenceProof contracts in crates/evidence-search-contracts/src/query.rs and crates/searchright-contracts/src/lib.rs.
- [ ] Implement canonical Reduced Ordered Binary Decision Diagram (ROBDD) arena with Shannon reduction and field/explosion/proximity domain subsumption axioms in crates/evidence-search-core/src/lib.rs.
- [ ] Implement prove_translation_equivalence and lexicographically minimal counterexample witness document synthesis across PubMed, Ovid MEDLINE, and Scopus in crates/evidence-search-core/src/compiler.rs.
- [ ] Enforce fail-closed recall narrowing validation in crates/searchright-validation/src/lib.rs and crates/searchright/src/engine.rs with differential compiler test suites.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-39-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
