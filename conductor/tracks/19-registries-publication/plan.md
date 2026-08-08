# Plan: 19 Registries and scholarly publication

Current status: **submission_prepared**. Implementation state: **scaffolded**. Evidence level: **source_verified**.

GitHub issue key: `track-19`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-19-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `registry/status.json`
  - [x] Present source path: `registry/official-mcp/README.md`
  - [x] Present source path: `registry/glama/README.md`
  - [x] Present source path: `registry/smithery/README.md`
  - [x] Present source path: `registry/joss/paper.md`
  - [x] Present source path: `server.json`
  - [x] Present source path: `glama.json`
  - [x] Assertion ledger: `conductor/tracks/19-registries-publication/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-19-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-19-phase-3 -->

- [ ] Publish a verified release that registry packets can reference.
- [ ] Submit to each external registry with maintainer approval.
- [ ] Record public listing, rejection or revision evidence.
- [ ] Complete JOSS submission requirements and independent review.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-19-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
