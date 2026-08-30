# 11: Systematic-search agent skill and workflows

## Objective

Package planning, execution, PRESS, screening and reporting workflows with conservative authority.

## Scope

- Publish SKILL.md with trigger and non-trigger boundaries
- Add planning, strategy, PRESS, execution, screening and reporting references
- Create subagent role cards and handoff contracts
- Add scenario, prompt-injection and authority tests
- Prepare a Searchright-owned sibling thin caller for explicit academic-research-skills handoffs; downstream adoption remains separately gated
- Publish skill registry packets after observed validation

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `11`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.

## Approved placement amendment — 2026-08-31

The owner approved the upstream maintainer's sibling route after ARS PR 807 was
closed without merge. The caller remains in Searchright and must not capture
ARS routing. This changes placement only: T11-G001 remains pending, a listing
is not adoption, and automated invocation requires separate runtime admission.
See `docs/adrs/0018-searchright-owned-sibling-caller.md` and
`verification/receipts/track-11-sibling-route.json` for scope and evidence.
