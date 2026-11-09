<!-- searchright-issue-key: track-24 -->
# Track 24: WASI components, HTTP MCP and scalable execution

Define sandboxed provider components, integrity/capability checks and future remote MCP boundaries.

## Source of truth

- Spec: `conductor/tracks/24-wasi-http-scale/spec.md`
- Plan: `conductor/tracks/24-wasi-http-scale/plan.md`
- Evidence: `conductor/tracks/24-wasi-http-scale/evidence.json`

## Contract

- Horizon: `beta`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `03, 10, 13, 16`
- Requirements: `SR-030, SR-047`
- External approval required: `false`

## Phase subissues

- [x] Phase 1: Source implementation (`track-24-phase-1`)
- [x] Phase 2: Source-level verification (`track-24-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-24-phase-3`)
- [ ] Phase 4: Review and closeout (`track-24-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating wasi-systems-architect, mcp-protocol-specialist, and security-governance-auditor roles, with Ed25519 signed manifest release verification, capability authorization, and tamper-resistance verified in crates/searchright-plugin-sdk/tests/component_sandbox_scenarios.rs. Simulation instrument and compiler-tested local boundaries only, not a substitute for live Wasmtime guest execution, cluster scalability, or remote Streamable HTTP under Track 34. Path presence is not behavioural proof; assertion-level traceability governs implementation claims. Original live, downstream, human and public outcomes remain open where listed in blockers; historical simulated-panel approval does not close them.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
