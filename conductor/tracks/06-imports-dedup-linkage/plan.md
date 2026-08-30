# Plan: 06 Imports, deduplication and study linkage

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **compiler_verified**.
Lifecycle: **archived** on **2026-08-30**; canonical source and GitHub keys are retained.


GitHub issue key: `track-06`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-06-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-dedup/src/lib.rs`
  - [x] Present source path: `crates/searchright-interchange/src/lib.rs`
  - [x] Present source path: `crates/searchright-interchange/src/xml.rs`
  - [x] Present source path: `crates/searchright-interchange/tests/xml_integrity.rs`
  - [x] Present source path: `docs/xml-imports.md`
  - [x] Present source path: `crates/searchright-study/src/lib.rs`
  - [x] Present source path: `contracts/examples/interchange-receipt.yaml`
  - [x] Present source path: `contracts/examples/study-graph.yaml`
  - [x] Present source path: `docs/adrs/0005-record-report-study-separation.md`
  - [x] Assertion ledger: `conductor/tracks/06-imports-dedup-linkage/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-06-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-06-phase-3 -->

- [x] Compile and run golden import/export round-trip tests across every supported format.
- [x] Run a digest-bound multi-agent panel evaluation of linkage and deduplication, then obtain the repository owner's recorded decision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-06-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
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
- [x] Close the track only after the agent-panel packet is evidenced and the repository owner records an approve, revise or reject decision.
