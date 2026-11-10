# Plan: 44 Generic versioned corpus and downstream-research handoff

Status: **scaffolded**, Evidence level: **source_verified**. Retain historical issue IDs; no VOI/source-span extraction engine is promised within Searchright.

## Phase 1: Source implementation

<!-- github-subissue-key: track-44-phase-1 -->

- [ ] Implement task-scoped source changes with symbols and positive/negative tests.

## Phase 2: Source-level verification

<!-- github-subissue-key: track-44-phase-2 -->

- [x] Historical network-free scaffolding checks were recorded.
- [ ] Re-run traceability and static validators on this revised head.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-44-phase-3 -->

- [ ] **44.1 — Consumer-neutral versioned CSL+sidecar+rights+identity manifest**
  - Depends on: 40/42; reference resolution and access invariants
  - Required result: cargo test --locked -p searchright-interchange
  - Verify: `undefined`
- [ ] **44.2 — Deterministic full and delta corpora with typed change notices**
  - Depends on: 44.1/41; full-replay fixture parity
  - Required result: cargo test --locked -p searchright-living
  - Verify: `undefined`
- [ ] **44.3 — Optional tobac/voiage adapter, with no invented effect or risk-of-bias calculation**
  - Depends on: 44.2; real downstream parse and loss accounting
  - Required result: ./scripts/verify.sh
  - Verify: `undefined`
- [ ] **44.4 — Consumer compatibility, SemVer, access and tamper negative tests**
  - Depends on: 44.3; two independently qualified imports
  - Required result: ./scripts/verify.sh
  - Verify: `undefined`
- [ ] **44.5 — Reproducible package and versioned API release once exact-head gates pass**
  - Depends on: 44.4; no publication/registry acceptance claim
  - Required result: ./scripts/verify.sh
  - Verify: `undefined`

## Phase 4: Review and closeout

<!-- github-subissue-key: track-44-phase-4 -->

- [ ] Run Conductor review or clearly scoped repository-native parity checks.
- [ ] Correct findings and pass current CI/security/live/consumer checks relevant to claims.
- [ ] Archive only with all higher-evidence and traceability gates passed.
