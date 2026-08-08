# Plan: 11 Systematic-search agent skill and workflows

Current status: **source_implemented_unverified**. Evidence level: **source_verified**.

GitHub issue key: `track-11`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-11-phase-1 -->

- [x] Implement and document the track's source deliverables.
  - [x] `skills/systematic-search/SKILL.md`
  - [x] `skills/systematic-search/workflows/systematic-review.yaml`
  - [x] `skills/systematic-search/references/authority.md`
  - [x] `crates/searchright-agent/src/lib.rs`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-11-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-11-phase-3 -->

- [ ] Run scenario-based agent evaluations across supported hosts and models.
- [ ] Calibrate authority and failure modes with human information specialists.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-11-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
