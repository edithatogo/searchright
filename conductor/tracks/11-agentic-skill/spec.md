# 11: Systematic-search agent skill and workflows

## Objective

Package planning, execution, PRESS-aligned critique, screening and reporting
workflows with conservative authority under the single-accountable-owner model.

## Scope

- Publish `SKILL.md` with trigger and non-trigger boundaries.
- Add planning, strategy, PRESS-aligned critique, execution, screening and
  reporting references.
- Implement a sealed five-role agent panel with preserved dissent and explicit
  accountable-owner adjudication.
- Keep automated lint, owner-adjudicated agent review and independent human PRESS
  peer review as distinct evidence classes.
- Create subagent role cards and handoff contracts.
- Add scenario, prompt-injection, authority, panel-isolation and claim-boundary
  tests.
- Prepare a Searchright-owned sibling thin caller for explicit
  academic-research-skills handoffs; downstream adoption remains separately
  gated.
- Publish skill registry packets only after observed validation and explicit
  owner authorisation.

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `11`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- The default project review gate uses isolated specialist agents and accountable
  owner adjudication; it does not require or claim additional human reviewers.
- Optional external human calibration remains separately labelled and cannot be
  inferred from agent-panel completion.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

- Claiming that agent-panel output is independent human peer review.
- Granting agents final eligibility, protocol, live-write, publication or
  evidence-promotion authority.
- Work owned by later tracks, which is documented but not promoted as
  implemented.

## Governance amendment — 2026-09-10

The owner confirmed that Searchright is a single human/developer project whose
internal reviews are performed by sealed panels of agents. The accountable owner
adjudicates findings and retains all consequential decision rights. Optional
external human review remains a distinct evidence class and is not a default
Track 11 closure requirement. See
`docs/adrs/0019-single-owner-agent-panel-governance.md`.

## Approved placement amendment — 2026-08-31

The owner approved the upstream maintainer's sibling route after ARS PR 807 was
closed without merge. The caller remains in Searchright and must not capture
ARS routing. This changes placement only: T11-G001 remains pending, a listing
is not adoption, and automated invocation requires separate runtime admission.
See `docs/adrs/0018-searchright-owned-sibling-caller.md` and
`verification/receipts/track-11-sibling-route.json` for scope and evidence.
