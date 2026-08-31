# Plan: 03 Shared provider runtime and Sourceright extraction

Current status: **integration_prepared**. Implementation state: **partially_implemented**. Evidence level: **source_verified**.

GitHub issue key: `track-03`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-03-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `crates/evidence-search-core/src/provider.rs`
  - [x] Present source path: `crates/searchright-connectors/src/lib.rs`
  - [x] Present source path: `crates/searchright-contracts/src/migration.rs`
  - [x] Present source path: `crates/searchright-sourceright-compat/src/lib.rs`
  - [x] Present source path: `migration/sourceright/replacement-map.yaml`
  - [x] Present source path: `migration/sourceright/parity-cases.json`
  - [x] Present source path: `scripts/check_sourceright_migration.py`
  - [x] Present source path: `tests/test_sourceright_migration.py`
  - [x] Present source path: `release/public-packages.json`
  - [x] Present source path: `public-api/README.md`
  - [x] Present source path: `docs/msrv-and-package-policy.md`
  - [x] Assertion ledger: `conductor/tracks/03-shared-provider-runtime/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-03-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_sourceright_migration.py`
  - [x] `python scripts/check_public_package_policy.py`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-03-phase-3 -->

- [ ] Obtain exact-revision compiler CI and deterministic transport-level adversarial evidence for DNS pinning, proxy suppression, redirects and response bounds before closing H-002.
- [ ] Validate a revision-pinned provider/fixture/case/dimension execution matrix and run old/new Sourceright fixtures downstream; the v1 dimension-summary readiness flag is not complete execution evidence or cutover authority.
- [ ] Complete feature-gated Sourceright cutover, semver review and rollback exercise.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-03-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Disable proxy bypass, deny non-global address forms and bound streamed responses.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Fail closed on blank parity approvals and incomplete migration-case coverage.
  - Review fix `e6bcbec1001a9725dfda2c6250e15796fff8e9c6`: Reconcile source-level claims with pinned Searchright and Sourceright evidence.
  - Review fix `07144ea13c9fd5c4f1105d8a3c6582d3e3d25dfb`: Require exact, unique parity-case and dimension coverage before cutover readiness.
  - Review fix `07144ea13c9fd5c4f1105d8a3c6582d3e3d25dfb`: Reconcile the fixture-parser migration table with the four fixture-backed adapters.
  - Review fix `ca265354de725701085cd3e9d7a466a8c955f15d`: Reject missing or reassigned migration catalogue cells and Rust catalogue drift with mutation regressions.
  - Review fix `ca265354de725701085cd3e9d7a466a8c955f15d`: Preserve transport-execution evidence and complete downstream matrix gates; v1 summary readiness cannot authorize cutover.
  - Review fix `390ae69ba78b9431e464067ead51b75d8ecd1f2a`: Report an exact record-budget stop with known continuation once; preserve terminal-page and within-page overflow distinctions.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.
