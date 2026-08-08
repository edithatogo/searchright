# Agent Instructions

## Source of truth

Load `CONTEXT.md` and `context/manifest.json` first.

1. `conductor/requirements.md`
2. `contracts/`
3. `conductor/design.md` and ADRs
4. implementation and tests
5. public documentation

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
- Default to dry-run for external writes and registry submissions.
- Use the shared `evidence-search-core`; do not reimplement provider runtime,
  receipt, retry, cache or query primitives in downstream crates.
- Keep `unsafe` forbidden except in a separately reviewed sandbox/runtime adapter.

## Verification

Run `scripts/verify.sh`; when unavailable, record exactly which gates were not run
in `verification/receipts/` rather than implying success.
