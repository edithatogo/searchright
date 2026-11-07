# Agent Instructions

## Source of truth

Load `CONTEXT.md` and `context/manifest.json` first.

1. `context/claim-boundaries.json`, capability matrix and hazard log
2. `conductor/requirements.md` and roadmap coverage
3. active Conductor track specification, plan, metadata and evidence
4. `contracts/`
5. `conductor/design.md` and ADRs
6. implementation and tests
7. public documentation

## Non-negotiable rules

- Never silently change a protocol, eligibility criterion, search strategy or
  screening decision.
- Never represent PRISMA-S as a conduct standard; it is a reporting contract.
- Never claim a provider is supported without deterministic fixtures and an
  opt-in live receipt.
- Never scrape licensed sources or evade access controls.
- Never auto-exclude a study using an agent unless the review policy explicitly
  permits it and a human confirmation is recorded.
- Preserve records separately from studies and reports; linkage is explicit.
- Default to dry-run for external writes, GitHub mutations, release promotion
  and registry submissions.
- Treat Conductor as canonical. GitHub issues, nested subissues and Project
  state are projections and cannot promote source or evidence status.
- GitHub synchronisers must be additive/idempotent and must not delete or archive
  remote work automatically.
- Remote MCP must not inherit local authority implicitly: authenticate the
  principal, enforce tenant, region, scope, concurrency and approval policy, and
  preserve an auditable decision.
- Telemetry is disabled by default; never export full text, credentials or
  sensitive identifiers without an explicit allowlist and approval.
- Never claim backup recoverability without a successful restore drill.
- Use the shared `evidence-search-core`; do not reimplement provider runtime,
  receipt, retry, cache or query primitives in downstream crates.
- Determine implementation state from assertion-level traceability; file or path presence is never sufficient proof.
- Keep neutral provider/query/receipt/audit types in `evidence-search-contracts`; do not introduce Searchright review-workflow dependencies below the neutral core.
- Preserve exact native search text and source spans; never claim cross-database semantic equivalence without a complete loss report and expert evidence.
- Keep all crates non-publishable unless `release/public-packages.json` explicitly promotes one after compiler, API, SemVer, licence and supply-chain evidence.
- Keep benchmark labels sealed and external to development prompts, fixtures and ranking agents.
- Do not redistribute reference-only or licence-review-required integrations, standards text or fork content.
- Treat review-state snapshots and `.srpack` bundles as derived integrity artefacts, not canonical screening authority or methodological certification.
- Keep `unsafe` forbidden except in a separately reviewed sandbox/runtime adapter.
- Do not promote a release train, pilot, registry packet or version 1.0 decision
  from source completeness alone.
- Keep every pull request scoped to one coherent delivery unit. Declare one
  primary Conductor track; when other tracks are affected, declare all affected
  IDs and a concrete shared-outcome rationale. Split unrelated work, not merely
  files owned by different tracks. Legacy `MULTI` declarations remain supported.
- Treat pull requests as merge-bound delivery units, not parking branches.
  Enable rebase auto-merge after scope admission, repair failed required checks
  on the same PR, and finish by verifying the exact merged `main` revision.
  Closing without merge is reserved for an explicitly documented superseded,
  unsafe, or unauthorized change.

## Verification

Use the risk-based local verification profiles in `docs/quality/proportionate-governance.md`.
Run `scripts/verify.sh` for final candidate admission and releases; do not rerun
unchanged checks after every bookkeeping commit. Required hosted checks remain
mandatory. Record commands, outcomes and unrun gates in one candidate record or
CI run, linked from Conductor rather than copied into parallel receipts.
Before remote bootstrap, require a clean Git tree, read `CODEX_HANDOFF.md`, and
run `scripts/run_static_harness.py`. Run the GitHub convergence audit after
control-plane changes or delivery that changes its canonical inputs; an ordinary
code/docs PR does not require a new Project audit. Mutation success alone never
proves remote parity.

## Bounded authorisation

An approved plan authorises its ordinary edits, tests, commits, scoped PRs and
agreed merge actions without repeated confirmation. Ask again only for a material
scope change, new privilege, destructive operation, sensitive-data exposure,
methodological decision or release/publication promotion. Dry-run-first external
writes and human-only final research authority remain unchanged. Repository
policy governs repository work; global skill defaults are guidance, not grounds
for additional approval loops. Never override host safety or tool permissions.
