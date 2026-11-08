# Proportionate development governance

## Purpose and boundaries

Spend review and verification effort on risk, not the number of administrative
files produced. This policy does not relax research authority, data protection,
licensed access, provenance, dependency certification, required hosted checks or
release criteria. Historical approvals, expiry decisions and dissent are retained.

## One approval for a bounded plan

Approval of an execution plan covers its ordinary edits, tests, commits and the
PR/merge actions explicitly included in that plan. Reconfirm only a material
scope change, new privilege, destructive action, sensitive-data exposure,
methodological decision or publication/release promotion. Dry-run-first remains
required for external writes; approval of a plan is not unrestricted authority.
Never bypass host safety controls or tool permissions. Do not modify global
skills to resolve repository-specific conventions.

## Coherent delivery and ownership

A PR should have one explainable outcome and one primary Conductor track.
Additional affected tracks must be enumerated, including the primary, with a
concrete shared-outcome rationale. Split unrelated concerns, but do not split
solely because a coherent change touches more than one track. The scope checker
validates declared IDs and rationale presence; reviewers assess actual coherence.
Legacy MULTI/exception-label declarations remain compatible.

This policy change itself is delivered under the prior single-track rule as
Track 16 quality/admission maintenance. It does not grant itself an exception.

## Local verification profiles

- Documentation and evidence linkage: validate affected schemas, references,
  generated projections, claim wording and text hygiene. Run the relevant
  regression tests; do not run Rust tests merely because a receipt changed.
- Code changes: run affected unit/integration tests, formatting and lint checks,
  plus checks for interfaces and dependent components touched by the change.
- Policy, workflow, security, dependency or contract changes: inspect failure
  cases and authority boundaries, run focused regressions, then the full
  candidate verification suite before admission.
- Final candidate admission and release: run `scripts/verify.sh` and all required
  hosted checks. Release-specific live, compatibility, restore, methodological
  and publication criteria remain separate and mandatory where applicable.

Unknown impact uses the broader profile. A failed required gate blocks admission.
Record unavailable gates accurately; narrower local checks are not a waiver of
branch protection or evidence requirements. This change does not alter hosted
job selection, branch protection, verifier commands or release thresholds.

Avoid repeated execution after bookkeeping-only changes. A reusable result must
identify its command, relevant source/configuration/dependency input digests,
toolchain/environment, outcome and original run. Reuse only when those inputs
are unchanged; otherwise rerun. Never call reused evidence a fresh execution.
Until input matching is available, rerun rather than assume equivalence. Required
exact-head hosted checks are not reusable across heads unless CI explicitly
supports that guarantee. Post-merge verify the actual revision and relevant
inputs; rerun the full suite when results cannot be safely matched.

## Review depth

Routine maintenance uses focused review and meaningful tests. Use isolated
methodology, implementation, testing, safety and adversarial first passes for
high-risk authority, methodology, security-boundary and promotion decisions, or
when a specific gate requires them. Preserve first passes and dissent. Agent
panels advise; they cannot grant owner authority or establish institutional/model
independence merely by repeating reviews. Dependency certification still requires
qualifying exact-version review; passing tests or exemptions are not audits.

## Evidence and projections

Prefer one candidate verification record or CI run with commands, outcomes,
input identity, toolchain/environment, skipped gates and durable artifact links.
Conductor references it rather than copying outcomes into new receipts. Store
large intermediate logs in retained CI artifacts, with digests when referenced;
retain critical decisions and sufficient durable evidence beyond artifact expiry.
Do not remove existing evidence or rewrite historical records under this policy.

Conductor stays canonical; GitHub is a coordination projection. Run convergence
audits after control-plane changes or changes to projected canonical inputs, not
automatically after every unrelated code/documentation PR. No successful mutation
alone proves parity. Routine local status does not require the full native suite.

Existing issue/task projections remain unchanged. Reducing future projection
granularity requires a separate additive migration design; do not delete, archive
or silently stop tracking current remote work. No projection migration is part of
this policy change.
