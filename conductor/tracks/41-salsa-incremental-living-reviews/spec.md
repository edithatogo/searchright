# 41: Demand-driven incremental living review computation

## Objective

Re-architect living systematic review updates (`crates/searchright-living`, `crates/searchright-dedup`, `crates/searchright-study`, and `crates/searchright-prisma`) around a demand-driven, Salsa-inspired memoized dependency DAG so that when new provider delta observations arrive or a single eligibility criterion is amended, only the exact downstream clusters, screening judgements, study aggregates and PRISMA cells that depend on the changed inputs are recomputed, with deterministic bisimulation equivalence to a full from-scratch replay.

## Scope

- Fine-grained dependency graph nodes and content-addressed revision fingerprints across observations, deduplication clusters, screening criteria, studies and PRISMA counts
- Demand-driven memoization and early-cutoff propagation skipping downstream recomputation when an intermediate node's output digest is unchanged
- Selective criterion-level invalidation on protocol amendments re-queueing only records whose decision depended on the modified eligibility criterion
- Incremental PRISMA 2020 and Living Systematic Review (LSR) delta ledger tracking added, merged, upgraded and invalidated records across update waves
- Deterministic bisimulation test harness proving bit-for-bit state identity between incremental update sequences and full from-scratch replay

## Requirements owned

`SR-097`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

Follow these step-by-step instructions to build the demand-driven incremental engine and bisimulation proof suite.

### Task 1: Define Incremental Dependency DAG Node & Revision Contracts (`crates/searchright-contracts`)

1. **Target Files**:
   - `crates/searchright-contracts/src/living.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - `DagNodeKey`:
     - Enum variants:
       - `SourceObservation { observation_id: String }`
       - `DedupCluster { cluster_id: String }`
       - `EligibilityCriterion { criterion_id: String }`
       - `ScreeningCell { cluster_id: String, criterion_id: String }`
       - `RecordDecision { cluster_id: String }`
       - `StudyGroup { study_id: String }`
       - `PrismaCounter { counter_name: String }`
   - `NodeMemoEntry`:
     - `node_key: DagNodeKey`
     - `input_dependencies: Vec<(DagNodeKey, String)>` (pairs of `(upstream_key, upstream_output_sha256)`)
     - `output_sha256: String`
     - `last_verified_revision: u64`
     - `last_changed_revision: u64`
   - `IncrementalUpdateReceipt`:
     - `revision: u64`
     - `dirty_input_keys: Vec<DagNodeKey>`
     - `recomputed_nodes: Vec<DagNodeKey>`
     - `early_cutoff_nodes: Vec<DagNodeKey>`
     - `reused_clean_nodes: usize`
     - `state_root_sha256: String`
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Demand-Driven Memoization Engine with Early Cutoff (`crates/searchright-living`)

1. **Target Files**:
   - `crates/searchright-living/src/dag.rs` (new module, exported in `crates/searchright-living/src/lib.rs`)
2. **Algorithms to Implement**:
   - Implement `IncrementalReviewDag`:
     - Stores `revision: u64` and `memos: BTreeMap<DagNodeKey, NodeMemoEntry>`.
     - **Early Cutoff Rule**: When an upstream input `SourceObservation` changes (e.g. a provider re-emits the same PMID with only a changed retrieval timestamp, leaving title, abstract, DOI and identifiers identical), `DedupCluster` recomputes its canonical content digest. Because its `output_sha256` matches the previous revision's `output_sha256`, `last_changed_revision` is NOT bumped, and downstream `ScreeningCell`, `RecordDecision`, `StudyGroup`, and `PrismaCounter` nodes are recorded in `early_cutoff_nodes` without re-execution.
     - **Selective Criterion Amendment Rule**: When `EligibilityCriterion { criterion_id: C }` is amended:
       1. Mark only `ScreeningCell { cluster_id: *, criterion_id: C }` as dirty.
       2. Further prune: if a `cluster_id` was already definitively excluded by an *unmodified* mandatory criterion $C' \neq C$ (`ScreeningCell { cluster_id, criterion_id: C' } == Excluded`), do not re-queue `cluster_id` for human re-screening unless $C'$ itself changes.
3. **Unit Tests to Add**:
   - Test metadata-only provenance refresh triggers early cutoff at `DedupCluster` and performs zero screening recomputations.
   - Test amending Criterion `C2` re-evaluates records included or pending on `C2`, while leaving records already excluded by untouched Criterion `C1` stable.
4. **Verification Command**:
   - `cargo test --locked -p searchright-living`

### Task 3: Incremental Deduplication & Study Linkage (`crates/searchright-dedup` & `crates/searchright-study`)

1. **Target Files**:
   - `crates/searchright-dedup/src/lib.rs`
   - `crates/searchright-study/src/lib.rs`
2. **Functions to Implement**:
   - In `crates/searchright-dedup/src/lib.rs`:
     - Implement `pub fn incremental_merge_observations(existing_clusters: &[DedupClusterRecord], delta_observations: &[SourceObservationRecord]) -> (Vec<DedupClusterRecord>, BTreeSet<String>)` returning the updated cluster list and the exact set of `changed_cluster_ids`.
   - In `crates/searchright-study/src/lib.rs`:
     - Implement `pub fn recompute_affected_studies(existing_studies: &[StudyAggregate], changed_cluster_ids: &BTreeSet<String>, all_clusters: &[DedupClusterRecord]) -> Vec<StudyAggregate>`.
3. **Verification Command**:
   - `cargo test --locked -p searchright-dedup -p searchright-study`

### Task 4: Bisimulation Equivalence Test Suite (`crates/searchright-living` & `crates/searchright-prisma`)

1. **Target Files**:
   - `crates/searchright-living/tests/bisimulation_replay.rs` (new integration test)
2. **Test Specification**:
   - Construct a 3-wave living review scenario:
     - **Wave 1**: Initial 25 observations across PubMed and Europe PMC, deduplicated and screened under Protocol v1.
     - **Wave 2**: 8 new observations (3 duplicates of Wave 1 records, 2 secondary reports of an existing RCT study, 3 brand-new studies).
     - **Wave 3**: Protocol v2 amends secondary outcome criterion `IC-03`.
   - Run **Path A (Incremental DAG)**: Apply Wave 1 $\rightarrow$ incremental Wave 2 $\rightarrow$ incremental Wave 3.
   - Run **Path B (From-Scratch Replay)**: Concatenate all observations through Wave 3 and execute a clean batch run under Protocol v2 with the same recorded human decisions.
   - Assert `state_root_sha256`, cluster sets, study groupings, and PRISMA flow counts are **bit-for-bit identical** between Path A and Path B, while Path A recomputes $< 35\%$ of the nodes in Wave 2 and Wave 3.
3. **Verification Command**:
   - `cargo test --locked -p searchright-living -p searchright-prisma`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
