# Plan: 30 Maturity gate and gap closure

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-30`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-30-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
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
  - [x] Assertion ledger: `conductor/tracks/30-maturity-gap-closure/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-30-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/check_maturity_dossier.py`
  - [x] `python scripts/check_roadmap_coverage.py`
  - [x] `python scripts/generate_evidence_debt.py --check`
  - [x] `python scripts/check_gate_catalog.py --check`
  - [x] `python scripts/check_launch_preparation_roadmap.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-30-phase-3 -->

- [ ] Generate compiler, fixture, live-provider, migration, usability and external-evaluation receipts for every critical domain.
- [ ] Review and approve any explicit release-risk exception without hiding the open gap.
- [ ] Keep autonomous end-to-end review and final agent exclusions outside the release claim.
- [ ] LP-001 / Track 10: complete field-level MCP schemas and official current/previous-era client transcripts; run the locked MCP test, Clippy and transcript commands in conductor/launch-preparation-roadmap.json and preserve both version receipts.
- [ ] LP-002 / Track 24: implement resources, prompts, tasks, MRTR, subscriptions, pagination and cancellation; pass all-feature MCP tests and preserve the advanced-capabilities receipt.
- [ ] LP-003 / Track 34: implement authenticated Streamable HTTP with issuer, principal, tenant, region, scope, rate, replay and approval enforcement; preserve auth and adversarial tenancy receipts.
- [ ] LP-004 / Track 04: close DNS resolution/connection-pinning controls and run authorised redacted live canaries for every claimed provider after policy approval.
- [ ] LP-005 / Track 02: complete versioned filters and supported dialect semantics with loss/property tests and accountable information-specialist PRESS review.
- [ ] LP-006 / Track 05: harden audit/store crash recovery, retention, export and deletion semantics and preserve fault/policy receipts.
- [ ] LP-007 / Track 06: complete representative import round trips, permutation-safe deduplication, explicit linkage and governed screening fixture suites.
- [ ] LP-008 / Track 08: polish PRISMA, PRISMA-S, PRESS and accessible deterministic reporting outputs through complete fixture/snapshot evidence.
- [ ] LP-009 / Track 32: execute exact-revision CiteWeft and Sourceright compiled canaries, dual-run parity and rollback without deleting recovery paths.
- [ ] LP-010 / Track 16: exceed the governed maturity coverage target, triage mutation/fuzz/formal results, resolve or re-approve dependency exemptions and obtain an exact-candidate green matrix.
- [ ] LP-011 / Track 28: complete sensitive-data, least-privilege, accessibility and information-specialist usability review with disposition receipts.
- [ ] LP-012 / Track 29: execute the preregistered sealed methodological evaluation and publish limitations, subgroup results and the accountable response matrix.
- [ ] LP-013 / Track 33: execute encrypted restore, audit-chain, incident, cancellation and deployment-SLO rehearsals against the target profile.
- [ ] LP-014 / Track 35: generate and install-smoke locked Python/TypeScript SDKs and complete fixture-backed tutorial walkthroughs.
- [ ] LP-015 / Track 36: run the reproducible signed/attested release-candidate, bounded pilots and rollback rehearsal without automatic publication.
- [ ] LP-016 / Track 37: aggregate current evidence, adjudicate every residual risk and perform separately approved release/registry actions only after a not-ready/ready decision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-30-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
