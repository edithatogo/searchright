# Context loading profiles

## Minimal code change

Load `AGENTS.md`, `CONTEXT.md`, the active track quartet, affected schemas and
the relevant crate. Add hazard/claim files whenever behavior, authority, egress,
persistence or public claims change.

## Integration change

Also load `integration/passports/index.json`, the specific passport,
`integration/locks.json`, `docs/integration-architecture.md`, the downstream
migration packet and consumer-driven fixtures.

## Agent or MCP change

Also load the capability matrix, untrusted-content policy, MCP tool catalogue,
interface parity catalogue and screening authority contract.

## Release or external write

Load every policy-tier file, release runbook, registry status, supply-chain
document, threat model and the explicit approval receipt. A prepared file never
acts as approval.
