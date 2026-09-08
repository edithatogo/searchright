<!-- searchright-issue-key: track-00-phase-4 -->
# Track 00 / Phase 4: Review and closeout

Parent track key: `track-00`
Conductor plan: `conductor/tracks/00-foundation-conductor-toolchain/plan.md`

## Task subissues

- [x] T01: Reconcile source paths, requirements, interface effects and claim boundaries. (`track-00-phase-4-task-01`)
- [x] T02: Record unresolved blockers in `evidence.json` and the roadmap coverage ledger. (`track-00-phase-4-task-02`)
- [x] T03: Run compiler-backed Conductor review and append review fixes after Cargo gates execute. (`track-00-phase-4-task-03`)
- [ ] T04: Close the track only when all applicable live, downstream, human and external gates are evidenced. (`track-00-phase-4-task-04`)
- [x] T05: Add a read-only native Conductor v3 status adapter with adversarial checks (`9b73b49`, review fixes `f47c92a`), preserving stable IDs, evidence-aware states and in-place archives; verification/receipts/track-00-native-status-adapter.json records bounded evidence. (`track-00-phase-4-task-05`)
- [x] T06: Reconcile historical aggregate-verification failures with exact-tree full verification and separate frozen cargo-vet policy evidence (`f372c67`); verification/receipts/track-00-verification-reconciliation.json preserves exemption limits and the unresolved upstream registration gate. (`track-00-phase-4-task-06`)

## Evidence rule

Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.
