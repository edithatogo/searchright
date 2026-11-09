# 39: Formal BDD/SMT query equivalence and semantic loss calculus

## Objective

Replace rule-based heuristic search translation warnings with a canonical Reduced Ordered Binary Decision Diagram (ROBDD) and bounded SMT equivalence engine that mathematically proves whether a cross-database query translation (e.g. PubMed to Ovid MEDLINE or Scopus) is semantically equivalent ($\equiv$), a safe recall-preserving superset ($\supset$), an unsafe recall-losing narrowing ($\subset$), or incomparable ($\bowtie$), and synthesizes concrete counterexample documents whenever semantic drift occurs.

## Scope

- Canonical Reduced Ordered Binary Decision Diagram (ROBDD) builder over atomic search predicates, field scopes and MeSH explosion trees
- Bounded relational constraint encoding for proximity operators (`NEAR/k`, `ADJ/k`) and date/publication-type filters
- Directional semantic subsumption solver classifying translations as equivalent, safe broadening, unsafe narrowing or incomparable
- Concrete witness counterexample document synthesizer generating minimal pseudo-records that match the source query but fail the target query
- Compiler integration attaching machine-checkable semantic equivalence proofs and counterexample witnesses to every cross-provider translation receipt

## Requirements owned

`SR-095`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

This playbook provides exact, mechanical instructions for an autonomous coding agent to implement formal query verification without external solver binaries (pure deterministic Rust).

### Task 1: Define Semantic Equivalence Proof & Counterexample Contracts (`crates/evidence-search-contracts` & `crates/searchright-contracts`)

1. **Target Files**:
   - `crates/evidence-search-contracts/src/query.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - `SemanticRelation`:
     - Enum variants: `Equivalent`, `BroadenedSuperset`, `NarrowedSubset`, `Incomparable`
   - `CounterexampleWitness`:
     - `witness_id: String`
     - `matched_source: bool`
     - `matched_target: bool`
     - `fields: BTreeMap<String, Vec<String>>` (e.g. `{"tiab": ["randomized"], "mesh_exploded": ["neoplasms/drug_therapy"]}`)
     - `explanation: String`
   - `QueryEquivalenceProof`:
     - `source_dialect: String`
     - `target_dialect: String`
     - `relation: SemanticRelation`
     - `source_robdd_root_hash: String`
     - `target_robdd_root_hash: String`
     - `lost_recall_witness: Option<CounterexampleWitness>` (present whenever `source && !target` is satisfiable)
     - `added_noise_witness: Option<CounterexampleWitness>` (present whenever `!source && target` is satisfiable)
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Canonical ROBDD Engine (`crates/evidence-search-core`)

1. **Target Files**:
   - `crates/evidence-search-core/src/bdd.rs` (new module, exported in `crates/evidence-search-core/src/lib.rs`)
2. **Data Structures & Algorithms to Implement**:
   - Define `AtomPredicate`:
     - `TermInField { field: CanonicalField, normalized_term: String, prefix_wildcard: bool }`
     - `ControlledConcept { vocabulary: String, concept_id: String, exploded: bool }`
     - `ProximityWindow { field: CanonicalField, term_a: String, term_b: String, ordered: bool, max_distance: u32 }`
     - `PublicationYearRange { min_year: Option<i32>, max_year: Option<i32> }`
   - Maintain a total deterministic variable ordering `AtomOrdering` (`BTreeMap<AtomPredicate, u32>`).
   - Implement `RobddArena`:
     - Nodes: `TerminalFalse (0)`, `TerminalTrue (1)`, and `Internal { var: u32, low: NodeId, high: NodeId }`.
     - Unique table (`HashMap<(u32, NodeId, NodeId), NodeId>`) and Shannon reduction rule: if `low == high`, return `low`.
     - Memoized `apply_and(a, b)`, `apply_or(a, b)`, `apply_not(a)`, `apply_diff(a, b)` (which computes $a \land \neg b$).
3. **Domain Subsumption Axioms (Bounded Theory Propagator)**:
   - Before evaluating satisfiability of a path assignment, enforce domain implications:
     - `TermInField { field: Title, term: X } == true` implies `TermInField { field: TitleAbstract, term: X } == true`.
     - `ControlledConcept { exploded: false, .. } == true` implies `ControlledConcept { exploded: true, .. } == true`.
     - `ProximityWindow { ordered: true, max_distance: k1, .. } == true` implies `ProximityWindow { ordered: false, max_distance: k2, .. } == true` for all $k_2 \ge k_1$.
     - Reject any path in `apply_diff` that violates these implications (pruning impossible truth assignments).
4. **Unit Tests to Add**:
   - Verify De Morgan's laws: $\neg(A \land B)$ produces the exact same `NodeId` as $(\neg A) \lor (\neg B)$.
   - Verify absorption and distributivity: $A \land (A \lor B) \equiv A$.
   - Verify domain implication: `Title("cancer")` $\subset$ `TitleAbstract("cancer")` yields `lost_recall_witness == None` when broadening from Title to TitleAbstract, and a concrete `lost_recall_witness` (document with `"cancer"` in `abstract` only) when narrowing from TitleAbstract to Title.
5. **Verification Command**:
   - `cargo test --locked -p evidence-search-core`

### Task 3: Implement Witness Counterexample Synthesizer & Dialect Classifier (`crates/evidence-search-core`)

1. **Target Files**:
   - `crates/evidence-search-core/src/compiler.rs`
   - `crates/evidence-search-core/src/native.rs`
2. **Functions to Implement**:
   - `pub fn prove_translation_equivalence(source_ir: &QueryIr, target_ir: &QueryIr, source_dialect: &str, target_dialect: &str) -> QueryEquivalenceProof`:
     - Lower both `source_ir` and `target_ir` into `RobddArena`.
     - Compute `diff_lost = arena.apply_diff(source_root, target_root)` ($S \land \neg T$).
     - Compute `diff_added = arena.apply_diff(target_root, source_root)` ($T \land \neg S$).
     - Classify `SemanticRelation`:
       - `diff_lost == False && diff_added == False` $\rightarrow$ `SemanticRelation::Equivalent`
       - `diff_lost == False && diff_added != False` $\rightarrow$ `SemanticRelation::BroadenedSuperset`
       - `diff_lost != False && diff_added == False` $\rightarrow$ `SemanticRelation::NarrowedSubset`
       - `diff_lost != False && diff_added != False` $\rightarrow$ `SemanticRelation::Incomparable`
     - When `diff_lost != False`, extract the lexicographically shortest satisfying path to `TerminalTrue` and materialize a `CounterexampleWitness` populating `fields` with every positive literal along that path.
3. **Unit Tests to Add**:
   - Test translating PubMed `"heart attack"[MeSH Terms]` (exploded by default in PubMed) to a target without explosion: prove `SemanticRelation::NarrowedSubset` and verify the synthesized witness contains a child MeSH descriptor that matches PubMed but fails the unexploded target.
   - Test translating Ovid `(covid adj3 vaccine).ti,ab.` to PubMed `"covid vaccine"[tiab]` (exact phrase = distance 0): prove `SemanticRelation::NarrowedSubset` and verify the witness has `"covid"` and `"vaccine"` separated by 2 intervening words.
4. **Verification Command**:
   - `cargo test --locked -p evidence-search-core`

### Task 4: Enforce Fail-Closed Recall Protection Policy in Validation & CLI (`crates/searchright-validation` & `crates/searchright`)

1. **Target Files**:
   - `crates/searchright-validation/src/lib.rs`
   - `crates/searchright/src/engine.rs`
2. **Behavior to Implement**:
   - Add `pub fn check_recall_safety(proof: &QueryEquivalenceProof, allow_narrowing: bool) -> Result<(), ValidationError>`:
     - If `proof.relation` is `NarrowedSubset` or `Incomparable` (meaning `proof.lost_recall_witness.is_some()`) and `!allow_narrowing`, return `Err(ValidationError::UnsafeRecallNarrowing)` with the formatted counterexample witness.
3. **Verification Command**:
   - `cargo test --locked -p searchright-validation -p searchright`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
