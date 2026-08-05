# Plan: 14 Maximal quality, context and security harness

Current status: **planned**.

## Phase 1: Generate lockfile and make deterministic CI green / Unit/integration/e2e/property/metamorphic/DST/CDC/fuzz/mutation tests

- [ ] Generate lockfile and make deterministic CI green
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Unit/integration/e2e/property/metamorphic/DST/CDC/fuzz/mutation tests
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Phase 2: Configure llvm-cov >90% and Codecov / Run cargo-deny/audit/semver/machete, CodeQL, Scorecard, zizmor, actionlint

- [ ] Configure llvm-cov >90% and Codecov
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Run cargo-deny/audit/semver/machete, CodeQL, Scorecard, zizmor, actionlint
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Phase 3: Add SSRF, secret leakage, parser bomb and resource exhaustion tests / Produce machine-readable verification receipt on every release

- [ ] Add SSRF, secret leakage, parser bomb and resource exhaustion tests
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.
- [ ] Produce machine-readable verification receipt on every release
  - [ ] Add or update governing contract/ADR.
  - [ ] Add deterministic tests and fixtures.
  - [ ] Update docs, evidence level and migration manifest.

## Review and closeout

- [ ] Run repository verification and track-specific gates.
- [ ] Record evidence receipt and unresolved blockers.
- [ ] Run Conductor review; append a review-fixes phase for any gaps.
- [ ] Update `conductor/tracks.md` without overstating external completion.
