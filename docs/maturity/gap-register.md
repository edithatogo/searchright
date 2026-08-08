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
