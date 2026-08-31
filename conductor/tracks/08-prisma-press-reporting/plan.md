# Plan: 08 PRISMA, PRESS and reporting

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-08`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-08-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/searchright-prisma/src/lib.rs`
  - [x] Present source path: `crates/searchright-validation/src/lib.rs`
  - [x] Present source path: `contracts/examples/prisma-flow.json`
  - [x] Present source path: `contracts/examples/search-validation.yaml`
  - [x] Present source path: `contracts/examples/standard-assessment.yaml`
  - [x] Present source path: `docs/standards-and-provenance.md`
  - [x] Assertion ledger: `conductor/tracks/08-prisma-press-reporting/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-08-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-08-phase-3 -->

- [ ] Complete wider render-format coverage and PRISMA.jl parity/ownership evidence.
- [ ] Obtain independent PRESS review of representative generated strategies and appendices.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-08-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.

- [x] Add shared validated library-level Markdown, JSON, Mermaid, SVG, Typst and DOCX-friendly HTML reporting projections with deterministic and injection tests (`6f0dbd1`); external renderer execution, complete update-cohort conformance, PRISMA.jl parity and PRESS review remain pending.
