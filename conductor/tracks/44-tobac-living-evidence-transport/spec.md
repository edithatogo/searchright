# 44: Living systematic review to VOI and bias-transport handoff for tobac

## Objective

Bridge Searchright living systematic review updates (`crates/searchright-living`, `crates/searchright-contracts`, and `crates/searchright-provenance`) to downstream regulatory Value of Information (`edithatogo/voiage`) and tobacco/vaping policy evaluation (`edithatogo/tobac` track `dependency_ecosystem_codesign_20261110`) by exporting deterministic delta bundles containing sub-document byte-span-grounded study effect estimates, structured Risk-of-Bias (RoB 2 / ROBINS-I) domain weights, and PICO target-population transportability modifiers while automatically quarantining retracted or expression-of-concern studies.

## Scope

- Typed `StudyEffectTransportRecord`, `RiskOfBiasDomainScore`, `PicoTransportWeight`, and `LivingVoiDeltaBundle` contracts in `crates/searchright-contracts/src/living.rs`
- Deterministic living-delta exporter in `crates/searchright-living/src/lib.rs` emitting added, updated, and retraction-quarantined study effect records between two review snapshots
- Proof-carrying byte-span and `citeweft`/`sourceright` provenance digest enforcement on every exported numeric effect size and confidence interval
- Target-population PICO transportability and internal-validity bias-adjustment metadata formatting for direct ingestion by `tobac` and `voiage`
- Fail-closed quarantine guard preventing retracted or ungrounded study claims from entering active downstream prior-updating bundles

## Requirements owned

`SR-100`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

Follow these step-by-step instructions to implement the living systematic review to VOI and bias-transport handoff in `crates/searchright-contracts`, `crates/searchright-living`, and `crates/searchright-provenance`.

### Task 1: Define Living-to-VOI & Bias-Transport Contracts (`crates/searchright-contracts`)

1. **Target Files**:
   - `crates/searchright-contracts/src/living.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - `RiskOfBiasDomainScore`:
     - `tool: String` (`"RoB2"`, `"ROBINS-I"`, `"GRADE"`)
     - `domain: String`
     - `judgement: String` (`"low"`, `"some_concerns"`, `"high"`, `"critical"`)
     - `variance_inflation_bps: u32`
   - `PicoTransportWeight`:
     - `target_jurisdiction: String` (e.g. `"NZ"`)
     - `target_subpopulation: String`
     - `transportability_score_bps: u16` (`0..=10000`)
     - `SelectionMismatchNotes: Vec<String>`
   - `StudyEffectTransportRecord`:
     - `study_id: String`
     - `outcome_id: String`
     - `effect_measure: String` (`"RR"`, `"OR"`, `"HR"`, `"MD"`, `"RD"`)
     - `point_estimate: f64`
     - `standard_error: f64`
     - `ci_lower: f64`
     - `ci_upper: f64`
     - `source_doc_sha256: String`
     - `verbatim_span_byte_range: (usize, usize)`
     - `rob_scores: Vec<RiskOfBiasDomainScore>`
     - `pico_transport: PicoTransportWeight`
     - `quarantined_retraction: bool`
   - `LivingVoiDeltaBundle`:
     - `bundle_id: String`
     - `review_id: String`
     - `parent_snapshot_sha256: String`
     - `current_snapshot_sha256: String`
     - `added_or_updated_effects: Vec<StudyEffectTransportRecord>`
     - `withdrawn_study_ids: Vec<String>`
     - `bundle_digest_sha256: String`
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Living Delta Extraction & Retraction Quarantine (`crates/searchright-living`)

1. **Target Files**:
   - `crates/searchright-living/src/lib.rs`
2. **Algorithms to Implement**:
   - `pub fn build_living_voi_delta_bundle(...) -> Result<LivingVoiDeltaBundle, LivingError>`:
     - Reject any `StudyEffectTransportRecord` where `!point_estimate.is_finite()`, `standard_error <= 0.0`, `ci_lower > point_estimate`, `point_estimate > ci_upper`, or `verbatim_span_byte_range.0 >= verbatim_span_byte_range.1`.
     - If a study is flagged by Track 42 transitive retraction / expression-of-concern propagation (`quarantined_retraction == true`), exclude it from `added_or_updated_effects` and append its `study_id` to `withdrawn_study_ids` so downstream `tobac`/`voiage` posteriors automatically de-condition or rerun sensitivity analyses.
     - Compute a canonical deterministic SHA-256 commitment (`bundle_digest_sha256`) over sorted records.
3. **Unit Tests to Add**:
   - **Ungrounded Effect Rejection Test**: Verify that an effect estimate with an empty byte span or non-positive standard error fails closed.
   - **Retraction De-Conditioning Test**: Verify that a newly retracted study in a living update moves to `withdrawn_study_ids` and never appears in `added_or_updated_effects`.
   - **Deterministic Delta Replay Test**: Verify identical `bundle_digest_sha256` under permuted input study orderings.
4. **Verification Command**:
   - `cargo test --locked -p searchright-living`

### Task 3: Bind Living VOI Bundle into Research Object Export (`crates/searchright-provenance`)

1. **Target Files**:
   - `crates/searchright-provenance/src/lib.rs`
2. **Behavior to Implement**:
   - Register `LivingVoiDeltaBundle` as a typed RO-Crate / `.srpack` artifact with explicit `prov:wasDerivedFrom` links to the underlying study report digests and screening transcripts.
3. **Verification Command**:
   - `cargo test --locked -p searchright-provenance`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
