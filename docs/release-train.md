# Cross-repository contract release train

Searchright, CiteWeft and Sourceright evolve independently but share contracts.
The release train prevents a source change in one repository from being treated
as compatible merely because its own tests pass.

## Promotion unit

The promotion unit is the exact combination recorded in
`integration/ecosystem-lock.json`, including:

- Searchright source revision;
- CiteWeft and Sourceright revisions;
- MCP protocol and Rust SDK revision;
- standards-pack and policy-pack identities;
- benchmark release identity;
- frozen contract surface;
- public-package policy.

The lock is default-deny and cannot promote itself. Automatic revision updates
are prohibited.

## Promotion order

1. Licence, origin and redistribution firewall.
2. Contract-surface and schema compatibility review.
3. Producer fixture and consumer-driven contract gates.
4. Compiler, MSRV, public-API and SemVer gates.
5. Downstream CiteWeft/Searchright/Sourceright canaries at the same exact pins.
6. Provider response-baseline and authorised live-canary review where relevant.
7. Release-candidate rehearsal, rollback and accountable human promotion.
8. Publication or registry submission as a separate approval-gated action.

Failed canaries restore the previous pin and feature boundary. No schema,
fixture, migration or failure evidence is deleted.

The canonical machine-readable plan is `integration/release-train.json`.
Scheduled jobs may detect drift and prepare receipts, but may not change pins,
merge changes, publish packages or alter public claims automatically.
