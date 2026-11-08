# Plan: 29 External methodological evaluation and sustainability

Current status: **external_evidence_required**. Implementation state: **external_evidence_required**. Evidence level: **source_verified**.

GitHub issue key: `track-29`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-29-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `docs/evaluation/external-methodological-evaluation.md`
  - [x] Present source path: `docs/evaluation/benchmark-and-calibration-protocol.md`
  - [x] Present source path: `docs/governance/sustainability.md`
  - [x] Present source path: `registry/joss/paper.md`
  - [x] Present source path: `GOVERNANCE.md`
  - [x] Present source path: `evaluation/external-evaluation.json`
  - [x] Present source path: `scripts/check_external_evaluation.py`
  - [x] Present source path: `tests/test_external_evaluation_sustainability_governance.py`
  - [x] Present source path: `verification/receipts/track-29-conductor-review-2026-11-05.json`
  - [x] Assertion ledger: `conductor/tracks/29-external-evaluation-sustainability/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-29-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_external_evaluation.py`
  - [x] `python3 -B -m unittest discover -s tests -p test_external_evaluation_sustainability_governance.py`
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
  - [x] Review fix: Map all six Track 29 assertions (T29-A001..T29-A006) to symbols and gates in conductor/traceability-overrides.json; add tests/test_external_evaluation_sustainability_governance.py checking protocol/contract agreement, no preregistration or validation over-claim, blinded developer-independent adjudication, benchmark leakage controls, the unsubmitted JOSS draft, observation-not-documentation sustainability, decision-class governance and mandatory disclosures; record the conductor review receipt. Preregistration, external recruitment and execution, published results and observed sustainability practice remain open; the track stays external_evidence_required.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
