<!-- searchright-issue-key: track-06-phase-4-task-03 -->
# Track 06 / Phase 4 / Task 03

Parent phase key: `track-06-phase-4`
Conductor plan: `conductor/tracks/06-imports-dedup-linkage/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - [x] Review fix: Preserve attributed PubMed abstract sections and inline title text (`7af55c0`).
  - [x] Review fix: Treat ISBN and trial-registration matches as candidate signals rather than automatic report duplicates.
  - [x] Review fix: Make large-collection title-token blocking complete for the configured Jaccard comparator.
  - [x] Review fix: Enforce study-membership and ReportOfStudy-edge consistency with operation-specific evidence.
  - [x] Review fix: Support parenthesized BibTeX and quarantine unterminated BibTeX and EndNote records with line spans.
  - [x] Review fix: Execute the rights-clear study-linkage fixture through all four declared local regression metrics.
  - [x] Review fix: Parse compact multi-record EndNote and same-line BibTeX without silent record loss.
  - [x] Review fix: Require every labelled linkage report to be assigned or explicitly abstained.
  - [x] Review fix: Reject sole-report detachment explicitly without mutating the graph.
  - [x] Review fix: Reject zero and invalid title-similarity thresholds and cover the 64/65 blocking boundary.
  - [x] Review fix: Simplify the threshold guard using De Morgan's law without changing its accepted domain; strict Clippy and all 10 dedup tests pass (`58a1c2f`).
  - [x] Review fix: Parse XML structurally, decode references once and quarantine damaged records without losing later valid records.
  - [x] Review fix: Scope primary PubMed identifiers and fields to owning paths, reject malformed XML attributes, and preserve integration-panel dissent and remediation evidence.
  - [x] Review fix: Execute six archival-renderer regressions in the required static harness without changing unrelated track task identities.
  - [x] Review fix: Enforce the separately approved CVX-0259 dependency exception with exact checksum, resolved-feature and expiry checks; preserve baseline decisions and risk-acceptance boundaries.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
