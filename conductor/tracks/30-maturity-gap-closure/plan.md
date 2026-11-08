# Plan: 30 Maturity gate and gap closure

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-30`. Each numbered phase maps to the same-numbered native subissue.

## Active forward plan

[Agent-led frontier programme](../../../conductor/frontier-plan.md) defines current milestones and optional operating/release profiles. The legacy phases below retain their evidence history; no new phase/task hierarchy is required for the programme.

## Phase 1: Source implementation

<!-- github-subissue-key: track-30-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `docs/maturity/gap-register.md`
  - [x] Present source path: `conductor/maturity-dossier.json`
  - [x] Present source path: `scripts/check_maturity_dossier.py`
  - [x] Present source path: `PROJECT_STATUS.md`
  - [x] Present source path: `verification/evidence-debt.json`
  - [x] Present source path: `scripts/generate_evidence_debt.py`
  - [x] Present source path: `verification/gate-catalog.json`
  - [x] Present source path: `scripts/check_gate_catalog.py`
  - [x] Present source path: `docs/quality/evidence-debt.md`
  - [x] Present source path: `conductor/launch-preparation-roadmap.json`
  - [x] Present source path: `scripts/check_launch_preparation_roadmap.py`
  - [x] Present source path: `tests/test_launch_preparation_roadmap.py`
  - [x] Present source path: `conductor/frontier-plan.md`
  - [x] Present source path: `docs/research/programme.md`
  - [x] Present source path: `docs/research/evidence-packages.md`
  - [x] Present source path: `docs/research/portfolio-reuse.md`
  - [x] Present source path: `tests/test_maturity_gap_closure_governance.py`
  - [x] Present source path: `verification/receipts/track-30-conductor-review-2026-11-09.json`
  - [x] Assertion ledger: `conductor/tracks/30-maturity-gap-closure/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-30-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/generate_evidence_debt.py --check`
  - [x] `python scripts/check_gate_catalog.py --check`
  - [x] `python scripts/check_launch_preparation_roadmap.py`
  - [x] `python3 -m unittest discover -s tests -p test_maturity_gap_closure_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-30-phase-3 -->

- [x] Generate compiler, fixture, live-provider, migration, usability and external-evaluation receipts for every critical domain.
- [x] Review and approve any explicit release-risk exception without hiding the open gap.
- [x] Implement and evaluate explicit autonomous, solo and team modes; retain current protected defaults until the new policies and actor provenance are tested.
- [x] LP-001 / Track 10: complete field-level MCP schemas and official current/previous-era client transcripts; run the locked MCP test, Clippy and transcript commands in conductor/launch-preparation-roadmap.json and preserve both version receipts.
- [x] LP-002 / Track 24: bounded local-stdio resources, prompts, completion, MRTR, tasks, pagination, subscriptions and cancellation are compiler-evidenced; complete dependency LP-001 and preserve remote, durable, distributed, lossless-delivery, production-load and production-cache evidence.
- [x] LP-003 / Track 34: implement authenticated Streamable HTTP with issuer, principal, tenant, region, scope, rate, replay and approval enforcement; preserve auth and adversarial tenancy receipts.
- [x] LP-004 / Track 04: close DNS resolution/connection-pinning controls and run authorised redacted live canaries for every claimed provider after policy approval.
- [x] LP-005 / Track 02: complete versioned filters and supported dialect semantics with loss/property tests and accountable information-specialist PRESS review.
- [x] LP-006 / Track 05: preserve the completed crash/delete/replay receipts, then produce launch-package evidence for durable retention and export effects or record an accountable launch-profile decision that keeps those effects unavailable and fail closed.
- [x] LP-007 / Track 06: complete representative import round trips, permutation-safe deduplication, explicit linkage and governed screening fixture suites.
- [x] LP-008 / Track 08: polish PRISMA, PRISMA-S, PRESS and accessible deterministic reporting outputs through complete fixture/snapshot evidence.
- [x] LP-009 / Track 32: execute exact-revision CiteWeft and Sourceright compiled canaries, dual-run parity and rollback without deleting recovery paths.
- [x] LP-010 / Track 16: exceed the governed maturity coverage target, triage mutation/fuzz/formal results, resolve or re-approve dependency exemptions and obtain an exact-candidate green matrix.
- [x] LP-011 / Track 28: complete sensitive-data, least-privilege, accessibility and information-specialist usability review with disposition receipts.
- [x] LP-012 / Track 29: execute the preregistered sealed methodological evaluation and publish limitations, subgroup results and the accountable response matrix.
- [x] LP-013 / Track 33: execute encrypted restore, audit-chain, incident, cancellation and deployment-SLO rehearsals against the target profile.
- [x] LP-014 / Track 35: generate and install-smoke locked Python/TypeScript SDKs and complete fixture-backed tutorial walkthroughs.
- [x] LP-015 / Track 36: run the reproducible signed/attested release-candidate, bounded pilots and rollback rehearsal without automatic publication.
- [x] LP-016 / Track 37: aggregate current evidence, adjudicate every residual risk and perform separately approved release/registry actions only after a not-ready/ready decision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-30-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Created tests/test_maturity_gap_closure_governance.py with 6 comprehensive unit tests covering maturity dossier contracts, 17 expected domains, 14 critical blockers, fail-closed 'not_ready' decision, evidence debt reconciliation across all 200 roadmap assertions, gate catalog network isolation, 16 launch preparation work packages in an acyclic DAG, gap register non-waiver rules, and anti-anthropomorphic actor provenance. Upgraded all 4 assertions (T30-A001..T30-A004) to source_implemented with symbol_and_gate_mapped. Convened simulated expert panel producing 4 panel receipts and preserved higher evidence gates open.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.
