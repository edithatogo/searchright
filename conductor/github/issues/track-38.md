<!-- searchright-issue-key: track-38 -->
# Track 38: Verifiable live two-source execution and zkTLS provenance receipts

Execute resumable two-source live retrieval (PubMed and Europe PMC) through CLI and MCP with cryptographic zkTLS transcript commitments in .srpack provenance bundles.

## Source of truth

- Spec: `conductor/tracks/38-zktls-verifiable-live-execution/spec.md`
- Plan: `conductor/tracks/38-zktls-verifiable-live-execution/plan.md`
- Evidence: `conductor/tracks/38-zktls-verifiable-live-execution/evidence.json`

## Contract

- Horizon: `mature`
- Status: `scaffolded`
- Implementation: `scaffolded`
- Evidence: `source_verified`
- Dependencies: `03, 04, 09, 10, 25`
- Requirements: `SR-094`
- External approval required: `true`

## Phase subissues

- [ ] Phase 1: Source implementation (`track-38-phase-1`)
- [ ] Phase 2: Source-level verification (`track-38-phase-2`)
- [ ] Phase 3: Higher-evidence gates (`track-38-phase-3`)
- [ ] Phase 4: Review and closeout (`track-38-phase-4`)

## Claim boundary

Source-verified specification and granular execution playbook only; live two-source execution and cryptographic zkTLS transcript attestation remain open implementation and live-verification gates. Path presence is not behavioural proof; assertion-level traceability governs implementation claims.

> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.
