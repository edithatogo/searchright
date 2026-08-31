# Plan: 11 Systematic-search agent skill and workflows

Current status: **partially_implemented**. Implementation state: **partially_implemented**. Evidence level: **compiler_verified**.

GitHub issue key: `track-11`. Each numbered phase maps to the same-numbered native subissue.

## Phase 1: Source implementation

<!-- github-subissue-key: track-11-phase-1 -->

- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only.
  - [x] Present source path: `skills/systematic-search/SKILL.md`
  - [x] Present source path: `skills/systematic-search/workflows/systematic-review.yaml`
  - [x] Present source path: `skills/systematic-search/references/authority.md`
  - [x] Present source path: `skills/systematic-search/references/handoffs.md`
  - [x] Present source path: `skills/systematic-search/evaluations/authority-scenarios.json`
  - [x] Present source path: `skills/systematic-search/evaluations/host-model-matrix.json`
  - [x] Present source path: `skills/systematic-search/evaluations/host-evaluation-protocol.md`
  - [x] Present source path: `scripts/run_agent_host_eval.py`
  - [x] Present source path: `scripts/test_agent_host_eval.py`
  - [x] Present source path: `skills/systematic-search/evaluations/human-calibration-protocol.md`
  - [x] Present source path: `skills/systematic-search/evaluations/human-calibration-template.json`
  - [x] Present source path: `skills/systematic-search/evaluations/human-calibration-recruitment.md`
  - [x] Present source path: `skills/systematic-search/integrations/academic-research-skills/SKILL.md`
  - [x] Present source path: `scripts/test_agent_skill_policy.py`
  - [x] Present source path: `docs/adrs/0018-searchright-owned-sibling-caller.md`
  - [x] Present source path: `verification/receipts/track-11-sibling-route.json`
  - [x] Present source path: `registry/skills/systematic-search/manifest.json`
  - [x] Present source path: `registry/skills/systematic-search/authorization-request.json`
  - [x] Present source path: `contracts/json-schema/agent-handoff.v1.schema.json`
  - [x] Present source path: `contracts/examples/agent-handoff.json`
  - [x] Present source path: `contracts/schema-catalog.json`
  - [x] Present source path: `crates/searchright-agent/src/lib.rs`
  - [x] Present source path: `crates/searchright-agent/tests/skill_scenarios.rs`
  - [x] Present source path: `scripts/check_agent_skill.py`
  - [x] Present source path: `verification/receipts/systematic-search-skill.json`
  - [x] Assertion ledger: `conductor/tracks/11-agentic-skill/traceability.json`

## Phase 2: Source-level verification

<!-- github-subissue-key: track-11-phase-2 -->

- [x] Run deterministic, network-free contract and policy checks.
  - [x] `python scripts/validate_repository.py`
  - [x] `python scripts/check_agent_skill.py`
  - [x] `cargo test -p searchright-agent --locked`
  - [x] `cargo clippy -p searchright-agent --all-targets --all-features --locked -- -D warnings`
- [x] Record machine-readable evidence without promoting compiler, live or external claims.

## Phase 3: Higher-evidence gates

<!-- github-subissue-key: track-11-phase-3 -->

- [ ] Obtain downstream academic-research-skills maintainer adoption and consumer-test evidence after licence and upstream-drift review.
- [ ] Run scenario-based agent evaluations across supported hosts and models.
- [ ] Calibrate authority and failure modes with human information specialists.
- [ ] Obtain explicit registry-submission authorization and an observed acceptance receipt before claiming publication.

## Phase 4: Review and closeout

<!-- github-subissue-key: track-11-phase-4 -->

- [x] Reconcile source paths, requirements, interface effects and claim boundaries.
- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.
- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `a170ff0ebc3226bb3fb390841eb7bb81e8fb1e2c`: Separate approval verification from untrusted proposals; deny generic final exclusions and amendments; enforce bounded, adjacent, purpose-bound and byte-verified handoffs; validate methodology, deduplication, telemetry and receipt freshness boundaries.
  - Review fix `b7f42271ca25860acc83ef16414b514e96f39c2b`: Enforce the complete advisory workflow, exact artifact-bound transition approvals, fixture-versus-live execution, bounded retained artifact bytes, kebab-case wire compatibility, a governed handoff schema, and explicit external closeout gates.
- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.

- [x] Prepare the owner-approved Searchright-owned sibling caller, enforce static routing/admission declarations, and reconcile historical consumer success with PR 807 closure (`e6969ac`); isolated five-role review recorded in verification/receipts/track-11-sibling-review.json.

### Runtime admission follow-up in progress

- [~] Implement and review local sibling byte/handoff admission and isolated
  host-evaluation failure handling. This is repository-owned preparation;
  automated invocation and all four mandatory gates remain pending.
