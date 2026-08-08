# Plan: 29 External methodological evaluation and sustainability

Current status: **external_evidence_required**. Implementation state: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-29`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-29-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `docs/evaluation/external-methodological-evaluation.md`
  - [x] Present source path: `docs/evaluation/benchmark-and-calibration-protocol.md`
  - [x] Present source path: `docs/governance/sustainability.md`
  - [x] Present source path: `registry/joss/paper.md`
  - [x] Present source path: `GOVERNANCE.md`
  - [x] Assertion ledger: `conductor/tracks/29-external-evaluation-sustainability/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-29-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-29-phase-3 -->

- [ ] Preregister the independent evaluation protocol.
- [ ] Recruit external information specialists and execute blinded evaluation.
- [ ] Publish results, limitations and response-to-findings matrix.
- [ ] Demonstrate observed maintenance, standards surveillance and succession practice.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-29-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
