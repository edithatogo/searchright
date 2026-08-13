# Maturity gap register

This register separates source completeness from evidence required for a mature
release. A gap closes only when its named receipt exists and the applicable
Conductor track records the promoted evidence level.

| Domain | Current state | Closure evidence | Owner track |
| --- | --- | --- | --- |
| Compiler and lockfile | Not evidenced | Committed lockfile and supported-platform Cargo receipts | 00, 16, 36 |
| Determinism and crash recovery | Source implemented | Replay, property, metamorphic and recovery receipts | 05, 17, 33 |
| Provider coverage | Fixture source only | Source-specific fixture and authorised live receipts | 04, 20, 21 |
| Methodology | Evaluation prepared | PRESS, seed-set, usability and independent evaluation | 17, 27, 29 |
| Security and formal assurance | Automation prepared | Current scans, fuzz, mutation, Miri, Loom and Kani receipts | 16, 26 |
| GitHub control plane | Source implemented | Observed remote repository, issues, Project and ruleset receipt | 31 |
| Cross-repository compatibility | Prepared | Producer/consumer canary and rollback receipts | 14, 15, 32 |
| Operational reliability | Source implemented | Health, restore, incident and resilience exercise receipts | 33 |
| Remote access and tenancy | Source implemented | Auth, isolation, rate-limit and threat-model evidence | 34 |
| SDKs and adoption | Planned/generated boundary | Built clients, install smokes and tutorial user tests | 35 |
| Pilot and release candidate | Prepared | Completed pilot exit and release rehearsal receipts | 36 |
| External acceptance | Prepared packets | Public registry, software-paper and release evidence | 19, 29, 36, 37 |

No unchecked row can be waived by changing documentation alone. Exceptions must
be recorded as explicit release risks, approved by a human and remain visible in
the final release decision.

The executable dependency order, owning tracks, commands, required receipts and
exit criteria are canonical in `conductor/launch-preparation-roadmap.json` and
validated by `scripts/check_launch_preparation_roadmap.py`. Track 30 Phase 3
projects each work package as a native task subissue. Those coordination tasks
do not supersede the owning track or promote its evidence.

Each work package also carries a fail-closed progress state. `not_started`
means no completion evidence is admitted; `partially_evidenced` names the exact
receipts and residual gates; and `completed` is accepted only when every named
receipt exists and every dependency is complete. Track 05 is semantically
archived, but LP-006 remains partially evidenced because durable retention and
export effects are not part of the current filesystem-store launch profile.
LP-002 is also partially evidenced: the bounded local-stdio receipt covers
resources, prompts, completion, current-only non-authoritative MRTR, pagination,
task admission/completion/cancellation and aggregate task-activity updates.
Completion still requires LP-001 plus the separately named remote, durable,
distributed, lossless-delivery, production-load and production-cache evidence.
