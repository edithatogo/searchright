<!-- searchright-issue-key: track-34 -->
# Track 34: Authenticated remote MCP, tenancy and data residency

Add default-deny principal, scope, tenant and region policy before any hosted Streamable HTTP MCP deployment.

## Source of truth

- Spec: `conductor/tracks/34-authenticated-remote-mcp/spec.md`
- Plan: `conductor/tracks/34-authenticated-remote-mcp/plan.md`
- Evidence: `conductor/tracks/34-authenticated-remote-mcp/evidence.json`

## Contract

- Horizon: `mature`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `fixture_proven`
- Dependencies: `10, 24, 28, 33`
- Requirements: `SR-047, SR-082, SR-083, SR-084, SR-085`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-34-phase-1`)
- [ ] Phase 2: Source-level verification (`track-34-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-34-phase-3`)
- [ ] Phase 4: Review and closeout (`track-34-phase-4`)

## Claim boundary

Fixture-proven via structured agent-panel evaluation simulating remote-mcp-security-architect, authorization-tenancy-auditor, and abuse-budget-auditor roles, with feature-gated loopback Streamable HTTP adapter, TLS terminator separation, rotating JWKS RS256 token verification, request replay nonce enforcement, tenant isolation, regional data residency, human-only final screening decisions, sliding-window rate limits, active task concurrency limits, request timeout budgets, and privacy-preserving redacted audit logging verified in tests/test_authenticated_remote_mcp_governance.py. Simulation instrument and source-level contracts only; live OAuth/OIDC IdP integration, multi-replica distributed replay/rate ledgers, physical cloud data residency verification, and production rollback rehearsals remain open operational gates. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
