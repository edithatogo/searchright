# Plan: 42 Sub-document byte-span entailment and transitive retraction propagation

Current status: **scaffolded**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-42`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-42-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `conductor/tracks/42-subdocument-entailment-retraction/spec.md`
  - [x] Present source path: `crates/searchright-contracts/src/document.rs`
  - [x] Present source path: `crates/searchright-contracts/src/study.rs`
  - [x] Present source path: `crates/searchright-citeweft/src/lib.rs`
  - [x] Present source path: `crates/searchright-study/src/lib.rs`
  - [x] Present source path: `crates/searchright-living/src/lib.rs`
  - [x] Assertion ledger: `conductor/tracks/42-subdocument-entailment-retraction/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-42-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/check_traceability.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-42-phase-3 -->

- [ ] Define PdfBoundingBox, ProofCarryingSpan, GroundedClaim, IntegrityNotice, and SynthesisQuarantineFlag contracts in crates/searchright-contracts/src/document.rs and crates/searchright-contracts/src/study.rs.
- [ ] Implement deterministic UTF-8 byte-slice verification and negation-aware claim entailment checking in crates/searchright-citeweft/src/lib.rs.
- [ ] Implement transitive retraction and expression-of-concern poisoning across Report-to-Study-to-Synthesis graphs in crates/searchright-study/src/lib.rs and crates/searchright-living/src/lib.rs.
- [ ] Add end-to-end span-tampering rejection and post-inclusion retraction quarantine tests across study synthesis and PRISMA living flow outputs.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-42-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
