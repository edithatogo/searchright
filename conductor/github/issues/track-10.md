<!-- searchright-issue-key: track-10 -->
# Track 10: MCP stdio server MVP

Expose typed, authority-annotated MCP tools over the same facade with stable structured outputs.

## Source of truth

- Spec: `conductor/tracks/10-mcp-mvp/spec.md`
- Plan: `conductor/tracks/10-mcp-mvp/plan.md`
- Evidence: `conductor/tracks/10-mcp-mvp/evidence.json`

## Contract

- Horizon: `mvp`
- Status: `source_implemented`
- Implementation: `source_implemented`
- Evidence: `compiler_verified`
- Dependencies: `09`
- Requirements: `SR-029`
- External approval required: `false`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-10-phase-1`)
- [ ] Phase 2: Source-level verification (`track-10-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-10-phase-3`)
- [ ] Phase 4: Review and closeout (`track-10-phase-4`)

## Claim boundary

The local stdio MCP surface is compiler-verified with 35 typed tools on the current protocol, 31 backwards-compatible tools on the previous era, full structured result contracts, bounded contract resources/prompts and a default-deny trusted-host authority boundary that binds consequential effects to exact canonical local-store bytes. Fixture execution is network-disabled; live providers, remote hosted MCP, methodological certification, final screening automation, publication and release are not claimed.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
