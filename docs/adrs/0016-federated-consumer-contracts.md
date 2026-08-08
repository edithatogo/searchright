# ADR 0016: Federated consumer-driven repository integration

- Status: Accepted
- Date: 2026-08-06

## Decision

Integrate independent repositories through exact-revision passports, neutral
contracts, golden fixtures, producer–consumer gates, explicit rollback and
read-only drift surveillance. Keep host-specific dependencies in optional leaf
adapters. Do not use Git submodules, copied implementation trees or automatic
revision/promotion jobs.

## Consequences

Repositories retain independent release histories while contract drift becomes
observable and reviewable. Source-level contract declarations remain distinct
from downstream execution evidence. Integration failures disable or quarantine
the adapter rather than silently coercing data or upgrading a public claim.
