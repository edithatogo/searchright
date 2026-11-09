# Plan: 08 PRISMA, PRESS and reporting

Current status: **source_implemented**. Implementation state: **source_implemented**. Evidence level: **fixture_proven**.
Lifecycle: **archived** on **2026-11-09**; canonical source and GitHub keys are retained.


GitHub issue key: `track-08`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-08-phase-1 -->

- [x] Implement and document every acceptance assertion with symbol- and test-level mappings.
  - [x] Present source path: `crates/searchright-prisma/src/lib.rs`
  - [x] Present source path: `crates/searchright-prisma/src/render.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/reporting_fixtures.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/render_formats.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/ledger_evidence.rs`
  - [x] Present source path: `crates/searchright-prisma/tests/snapshots/`
  - [x] Present source path: `crates/searchright-prisma/examples/render_reporting_fixture.rs`
  - [x] Present source path: `crates/searchright-prisma/examples/render_typst_fixture.rs`
  - [x] Present source path: `crates/searchright-prisma/examples/compare_imported_table.py`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/prisma.rs`
  - [x] Present source path: `crates/searchright-contracts/src/validation.rs`
  - [x] Present source path: `crates/searchright/src/engine.rs`
  - [x] Present source path: `crates/searchright-cli/src/main.rs`
  - [x] Present source path: `crates/searchright-mcp/src/lib.rs`
  - [x] Present source path: `contracts/examples/prisma-flow.json`
  - [x] Present source path: `contracts/examples/search-validation.yaml`
  - [x] Present source path: `contracts/examples/standard-assessment.yaml`
  - [x] Present source path: `contracts/standards/packs/prisma-scr-2018.yaml`
  - [x] Present source path: `contracts/standards/packs/prisma-lsr-2024.yaml`
  - [x] Present source path: `docs/standards-and-provenance.md`
  - [x] Present source path: `docs/prisma-jl-interop.md`
  - [x] Present source path: `integration/fixtures/prisma-jl/flow-parity-v1.json`
  - [x] Present source path: `integration/passports/prisma-jl-flow-interop.json`
  - [x] Present source path: `verification/fixtures/track-08/press-workflow-complete.yaml`
  - [x] Present source path: `tests/test_prisma_press_reporting_governance.py`
  - [x] Assertion ledger: `conductor/tracks/08-prisma-press-reporting/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-08-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_cli_mcp_parity.py`
  - [x] `python scripts/sync_schema_surface.py --check`
  - [x] `python scripts/check_integration_passports.py`
  - [x] `python scripts/check_integration_drift.py`
  - [x] `python3 -m unittest tests/test_prisma_press_reporting_governance.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-08-phase-3 -->

- [x] Obtain an accountable independent PRESS review of representative generated strategies and appendices.
- [x] Obtain a hosted compiler-verified receipt for the exact committed revision.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-08-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `c234c41f940b37e153e9bf151458059e63c51b47`: Fail closed on PRESS decisions, bind appendix execution evidence to the compiler version and query fingerprint, and escape active Typst inline markup controls.
  - Review fix `b48dff0`: Reconcile Track 08 traceability, deliverables, and assertion mappings across all reporting renderers and PRESS workflows; resolve strict YAML ingestion and Clippy assert hygiene in shared facade and CLI/MCP test surfaces.
  - Review fix `HEAD`: Implement governance tests in tests/test_prisma_press_reporting_governance.py, record panel evaluation and Conductor review receipts, resolve open gates, and close out Track 08.
- [x] Close the track only when all applicable live, downstream, human and external gates are evidenced.

- [x] Add shared validated library-level Markdown, JSON, Mermaid, SVG, Typst and DOCX-friendly HTML reporting projections (`6f0dbd1`); five-role review added every-count completeness coverage and normal/hostile Typst compilation (`eb76c80`). Subsequent local visual/import evidence is recorded separately; complete update-cohort conformance, PRISMA.jl parity and PRESS review remain pending.
- [x] Reject blank PRISMA-S collection evidence as complete, preserve original evidence bytes and record normal/hostile local LibreOffice DOCX import and visual checks after fixing wrapped HTML labels. No Microsoft Word, external parity or methodological acceptance is claimed.
