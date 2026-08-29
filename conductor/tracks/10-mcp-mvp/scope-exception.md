# Track 10 multi-track scope exception

Track 10 cannot expose the contracted `press_review_strategy` and
`record_screening_decision` operations safely without the exact persistence
contracts that were still open in Tracks 08 and 07. Splitting those contracts
would leave the Track 10 adapters either non-functional or dependent on
replaceable snapshots that repository policy declares noncanonical.

Affected tracks:

- Track 07: immutable full screening-decision commits and role-policy replay.
- Track 08: standalone PRESS schema plus exact confirmed local persistence.
- Track 10: facade, CLI and MCP delegation, effect metadata and conformance.

The exception is bounded to those technically inseparable prerequisites. It
does not complete or archive Tracks 07 or 08, does not permit agent exclusions,
and does not certify PRESS completeness. `execute_search` remains fixture-only;
live network execution is denied until Track 03 closes H-002. The delivery pull
request must carry `scope:multi-track-exception` and identify this file.
