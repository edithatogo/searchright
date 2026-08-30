# ADR 0018: Searchright-owned sibling caller

- Status: accepted for preparation; runtime admission pending
- Date: 2026-08-31
- Track: 11

## Context

ARS maintainer feedback on issue 806 and the closure of PR 807 reject an
overlapping fifth top-level skill and a single-alpha in-tree binding. The
maintainer recommends a sibling caller owned by Searchright. The accountable
owner explicitly approved proceeding with that route on 2026-08-31.

## Decision

Keep original caller content in Searchright, reference canonical upstream
`Imbad0202/academic-research-skills`, and require an explicit user handoff. Do
not change ARS triggers, copy its CC BY-NC 4.0 content, reopen PR 807, submit a
listing, or infer maintenance acceptance from placement approval.

Automated invocation stays disabled. Before runtime admission, implement an
executable verifier for exact package/source/schema bytes and actual hashed
synthetic artifacts, handoffs and approval receipts. Exercise missing,
incompatible, wrong-digest, malformed, denied and successful paths, including
final-exclusion and amendment refusals, full-pipeline and host-portability
tests. Static declaration checks are deliberately not that verifier.

## Evidence and consequences

The rejected proposal's exact head `4e4264a2557f9d296119d3021c53266127ac581d`
passed all 158 local CI-manifest entries. This does not validate this revised
sibling workflow, prove hosted CI, or establish downstream adoption. A future
THIRD_PARTY listing would establish listing only. T11-G001 remains pending;
host/model, calibration and registry gates are unchanged. Rollback preserves
manual handoffs and disables this caller without changing upstream history.

Exact references and log digests are recorded in
`verification/receipts/track-11-sibling-route.json`; the older readiness receipt
is retained as historical evidence, not current adoption status.
