# Plan: 44 Living systematic review to VOI and bias-transport handoff for tobac

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-44`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-44-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `conductor/tracks/44-tobac-living-evidence-transport/spec.md`
  - [x] Present source path: `crates/searchright-contracts/src/living.rs`
  - [x] Present source path: `crates/searchright-living/src/lib.rs`
  - [x] Present source path: `crates/searchright-provenance/src/lib.rs`
  - [x] Assertion ledger: `conductor/tracks/44-tobac-living-evidence-transport/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-44-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/check_traceability.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-44-phase-3 -->

- [ ] Define StudyEffectTransportRecord, RiskOfBiasDomainScore, PicoTransportWeight, and LivingVoiDeltaBundle contracts in crates/searchright-contracts/src/living.rs.
- [ ] Implement deterministic living-delta extraction and retraction-quarantine filtering for downstream VOI handoff in crates/searchright-living/src/lib.rs.
- [ ] Enforce sub-document byte-span digest and Sourceright/CiteWeft provenance binding on every exported effect estimate in crates/searchright-provenance/src/lib.rs.
- [ ] Add golden tobacco/vaping policy evidence transport fixtures and negative ungrounded-effect rejection tests for tobac and voiage.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-44-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
