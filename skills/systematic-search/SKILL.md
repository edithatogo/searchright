---
name: systematic-search
description: Plan, execute, screen and report systematic, scoping, rapid and living literature searches through Searchright contracts, CLI or MCP tools. Use for PICO/PCC framing, source selection, query construction/translation, PRESS review, deduplication, screening, PRISMA-S and PRISMA flow work. Do not use as an autonomous final eligibility decision-maker or to bypass licensed database access.
version: 0.1.0-alpha.1
license: MIT OR Apache-2.0
compatibility:
  mcp_server: io.github.edithatogo/searchright
  protocol: 2026-07-28
  cli: searchright
metadata:
  author: Dylan Mordaunt
  repository: https://github.com/edithatogo/searchright
  data_access_level: raw
  task_type: open-ended
---

# Systematic Search

## Trigger boundary

Use this skill when the task involves a systematic/scoping/living review search,
search protocol, PICO/PCC/SPIDER/PEO question, database strategy, search
translation, PRESS review, deduplication, title/abstract or full-text screening,
PRISMA-S appendix, PRISMA flow or reproducible search update.

Do not activate for an ordinary one-off web search, a narrative bibliography with
no systematic method, or a request to make final clinical conclusions from search
results. Do not represent inaccessible licensed databases as searched.

## Non-negotiable rules

1. Separate the database/resource from the platform/interface.
2. Distinguish review conduct from reporting-standard completeness.
3. Preserve the exact executed syntax, date, limits, filters and result count for
   every source.
4. Never silently change eligibility criteria or canonical records.
5. Never let an agent make an irreversible exclusion under the default policy.
6. Obtain explicit approval before live network execution or durable writes.
7. Keep all source receipts, duplicate evidence, decisions, amendments and
   exclusions in the audit trail.
8. Report limitations and unsearched sources plainly.

## Workflow

### 1. Scope and plan

- Identify review kind: systematic, scoping, rapid, living, evidence map or
  umbrella review.
- Frame the question using PICO/PECOS for intervention/exposure questions, PCC
  commonly for scoping reviews, or another justified framework.
- Convert prose inclusion/exclusion criteria into operational, versioned rules.
- Select complementary databases, registers, grey-literature sources, citation
  methods, contacts and handsearching. Record access/platform constraints.
- Create or validate a `ReviewPlan`. Pause for human approval.

Use `validate_plan` through MCP or `searchright validate-plan` through the CLI.

### 2. Design strategies

- Build concept blocks from controlled vocabulary and free text, including
  spelling, abbreviations, older terminology and truncation where appropriate.
- Avoid mechanically searching every PICO outcome if this risks poor recall.
- Encode the strategy as a portable query AST, then compile separately for each
  source. Never claim automatic translation is exact when warnings are emitted.
- Record every limit/filter with its rationale and version/citation.

Use `compile_strategy`. Resolve every `review_required` warning before execution.

### 3. Independent PRESS review

A human information specialist should review translation of the question,
Boolean/proximity operators, subject headings, text words, spelling/syntax/line
numbers and limits/filters. The strategy author records responses and amendments;
the peer reviewer records approval or unresolved blocking findings.

Automated checks may prepare a PRESS worksheet but do not count as independent
peer review.

### 4. Execute

- Confirm the exact source/platform, credentials, allowed host, date and execution
  policy.
- Use fixture or replay mode during development. Live mode requires explicit
  approval and is bounded by time/page/record/rate limits.
- Preserve raw-response hashes where lawful, normalized records, the source
  receipt and any partial-result warning.
- Search registers and grey literature using their declared methods; do not
  substitute an open discovery API for an unavailable licensed database.

### 5. Import and deduplicate

- Import records with source provenance; quarantine malformed records rather than
  silently dropping them.
- Deduplicate exact DOI/PMID/registry identifiers first, then conservative
  title/author/year candidates.
- Review fuzzy clusters. Retain original records and merge evidence; do not erase
  source counts needed for PRISMA.

Use `deduplicate_records` in preview mode.

### 6. Screen

- Screen titles/abstracts independently against the current eligibility version.
- Resolve conflicts through the declared human process.
- Retrieve full text and record one primary exclusion reason tied to a criterion.
- Agent suggestions are advisory by default. Never infer a final exclusion merely
  from missing abstract text or uncertain metadata.
- Record protocol amendments and decide whether earlier records require
  re-screening.

### 7. Report and update

Generate from audit evidence:

- PRISMA-S 16-item ledger and gaps;
- complete source-specific search appendix;
- PRISMA 2020/ScR/LSR flow as appropriate;
- dates, limits, peer-review method, record totals and deduplication method;
- list of full-text exclusions and reasons;
- update lineage, prior-work reuse and amendments for living/update searches.

Use `generate_prisma` and `verify_audit`. A valid flow is not a methodological
endorsement; unresolved gaps remain visible.

## Artefact contract

A complete run should contain:

```text
review-plan.yaml
eligibility.vN.yaml
strategies/<source>-<platform>.yaml
compiled/<source>-<platform>.txt
press/<strategy>-review.yaml
runs/<run-id>/source-receipts.json
runs/<run-id>/records.jsonl
dedup/duplicate-clusters.json
screening/title-abstract.jsonl
screening/full-text.jsonl
screening/exclusion-reasons.json
report/prisma-s-ledger.json
report/prisma-flow.json
report/prisma-flow.mmd
report/search-appendix.md
audit.jsonl
```

## Failure handling

Stop and escalate when a required source is inaccessible, search syntax cannot be
translated without material loss, a provider returns partial/inconsistent pages,
PRESS has a blocking finding, eligibility changed without approval, screening
conflicts remain unresolved, full-text exclusions lack reasons, or PRISMA counts
do not reconcile.

Read `references/methodology.md`, `references/authority.md`,
`references/tool-map.md` and the agent role cards before high-consequence work.
