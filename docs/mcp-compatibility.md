# MCP compatibility

Searchright's local standard-I/O server targets Model Context Protocol revision
`2026-07-28` through the official Rust SDK `rmcp` 3.1.2, with an explicit
compatibility transcript for the previous `2025-11-25` protocol era.

## Protocol profile

- Current-era clients use the `server/discover` handshake. Every modern request
  carries `io.modelcontextprotocol/protocolVersion`,
  `io.modelcontextprotocol/clientInfo` and
  `io.modelcontextprotocol/clientCapabilities` in `params._meta`.
- Previous-era clients use the retired `initialize` handshake followed by
  `notifications/initialized`. That transcript requests `2025-11-25` and checks
  the negotiated protocol version before listing or invoking tools.
- Tool discovery is deterministic and each tool has an object input schema.
- Every tool advertises a field-level JSON Schema 2020-12 `outputSchema`
  derived from the canonical interface catalogue and referenced contract
  schemas. Tool calls return matching structured
  content and backwards-compatible text content. Current-era tool calls also
  return `resultType: complete`. Rendered Mermaid and diagnostic documents
  retain their human-readable text block while also returning a structured
  `{format, document}` object.
- Every stdio tool advertises `readOnlyHint: true`, `destructiveHint: false`,
  `idempotentHint: true` and `openWorldHint: false`. These are client hints,
  not an authorisation decision; the shared Searchright facade remains the
  enforcement boundary.
- Tool-originated failures use `isError: true`; protocol errors are reserved for
  malformed or unsupported MCP requests.
- Facade rejections are deliberately redacted to the fixed text
  `operation_rejected: operation rejected by the shared Searchright facade`.
  User-controlled identifiers, endpoints, paths and provider diagnostics are not
  reflected across the protocol boundary.
- The local server exposes no session identifier and keeps application state in
  explicit Searchright contracts rather than hidden transport sessions.

The server does not adopt deprecated Roots, Sampling or Logging features.
Resources, prompts, multi-round-trip input requests, subscriptions, Tasks,
pagination, cancellation and authenticated Streamable HTTP are separate roadmap
capabilities and are not claimed by the local stdio profile.

The advertised output schemas are generated at server construction from
`contracts/interface-catalog.json`. Object outputs carry properties and
required fields; array outputs carry item schemas; shared wire contracts embed
their canonical JSON Schema files. A unit test fails if any of the 31 tools
regresses to a root-only schema. The smoke harness validates invoked tools'
`structuredContent` against the advertised `outputSchema`; its
`--strict-schemas` flag is the transcript gate for rejecting trivial schemas.
Independent or official live-client validation of every success path remains
open and is not inferred from catalogue-derived advertisement.

## Verification

Build the server first, then reproduce the pinned current-era transcript with:

```text
python scripts/mcp_smoke.py --receipt verification/receipts/mcp-2026-07-28-stdio.json target/debug/searchright-mcp
```

Reproduce the pinned previous-era transcript with:

```text
python scripts/mcp_smoke.py --protocol-version 2025-11-25 --receipt verification/receipts/mcp-2025-11-25-stdio.json target/debug/searchright-mcp
```

The current `2026-07-28` transcript asserts:

- `server/discover` handshake support;
- interface-catalogue parity for all 31 tools;
- deterministic tool ordering;
- input and output schema advertisement;
- governed read-only/non-destructive annotations;
- `structuredContent` validation against the advertised `outputSchema` for each
  invoked tool;
- governed error redaction and JSON-RPC protocol error shape;
- `resultType: complete`; and
- `ttlMs` plus `cacheScope` cache metadata.

The previous `2025-11-25` transcript asserts:

- `initialize` plus `notifications/initialized` handshake support;
- interface-catalogue parity for all 31 tools;
- deterministic tool ordering;
- input and output schema advertisement;
- governed read-only/non-destructive annotations;
- `structuredContent` validation against the advertised `outputSchema` for each
  invoked tool;
- governed error redaction; and
- JSON-RPC protocol error shape.

`resultType`, `ttlMs` and `cacheScope` are intentionally current-era-only
assertions. They are `2026-07-28` protocol fields and were not part of the
`2025-11-25` response model, so the compatibility transcript must not require
or infer them for previous-era responses.

These transcripts invoke only tools with no required arguments: `list_providers`
and `workflow`. Tools with required arguments are listed, ordered and
schema-advertisement-checked, but they are not called by this harness. The smoke
receipts are therefore Searchright compatibility smoke evidence, not a claim
that every tool path, optional MCP feature or remote transport conformance
scenario is implemented.

Adding `output_contract` references to the interface catalogue is an additive
metadata change. It does not change a persisted payload or remove an accepted
field, so no data migration is required. Independent or official client
conformance remains a separate open gate.
