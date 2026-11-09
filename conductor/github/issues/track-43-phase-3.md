<!-- searchright-issue-key: track-43-phase-3 -->
# Track 43 / Phase 3: Higher-evidence gates

Parent track key: `track-43`
Conductor plan: `conductor/tracks/43-adversarial-multi-agent-deliberation/plan.md`

## Task subissues

- [ ] T01: Define DeliberationRole, CriterionArgument, ArbiterDisposition, and DeliberationTranscript contracts in crates/searchright-contracts/src/screening.rs. (`track-43-phase-3-task-01`)
- [ ] T02: Implement verbatim evidence quote byte-range validation, epistemic divergence calculation, and conservative CalibratedArbiter adjudication in crates/searchright-screening/src/lib.rs. (`track-43-phase-3-task-02`)
- [ ] T03: Enforce Advocate-Skeptic prompt isolation and non-autonomous final exclusion guards in crates/searchright-agent/src/lib.rs. (`track-43-phase-3-task-03`)
- [ ] T04: Add adversarial hallucinated-quote rejection, high-divergence human escalation, and non-human-consensus invariant tests. (`track-43-phase-3-task-04`)

## Evidence rule

Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.
