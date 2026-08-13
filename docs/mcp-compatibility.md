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

The server does not adopt deprecated Roots, Sampling or Logging features. Its
bounded local-stdio advanced profile additionally provides two immutable
resources, one aggregate task-activity resource, two authored prompts, bounded
prompt completion, endpoint-specific cursor pagination, a form-elicitation MRTR
claim-boundary acknowledgement, genuine aggregate resource-update notifications,
and opt-in Tasks with cooperative cancellation and a four-task admission cap.
Official `rmcp` clients exercise both `2026-07-28` and `2025-11-25` profiles.
The previous-era profile remains static and cannot create Tasks or use MRTR.

These capabilities deliberately do not extend to authenticated Streamable
HTTP. The remote profile advertises and accepts tools only until Track 34 binds
advanced state to an authenticated principal, tenant, region, scope, approval,
quota and auditable decision. Task state is local-process only: it is not
durable, resumable, multi-replica or a source of screening/execution authority.
The MRTR retry carries only a fixed acknowledgement and proves protocol control
flow, not approval, execution authority or methodological review. The catalogues
are immutable, so no list-change capability is advertised. Current local clients
may subscribe only to the aggregate task-activity resource; notifications occur
after a real task start or terminal transition and expose no task identifier or
payload. This is bounded local-process behavior, not lossless event delivery,
production load, cache, durability or scale evidence.

The advertised output schemas are generated at server construction from
`contracts/interface-catalog.json`. Object outputs carry properties and
required fields; array outputs carry item schemas; shared wire contracts embed
their canonical JSON Schema files. A unit test fails if any of the 31 tools
regresses to a root-only schema. The smoke harness validates invoked tools'
`structuredContent` against the advertised `outputSchema`; its
`--strict-schemas` flag is the transcript gate for rejecting trivial schemas.
The Track 10 official-client harness launches the actual `searchright-mcp`
binary over child-process stdio for both supported eras. It invokes all 31
tools with deterministic valid arguments, exercises both `generate_prisma`
success variants, and independently validates every observed
`structuredContent` value against the `outputSchema` received through
`tools/list`. Governed failures remain `isError` responses without successful
structured content. The receipt records official `rmcp` 3.1.2 evidence; it
does not claim interoperability with unrelated third-party client libraries.

Run and record the official-client evidence only from a clean tracked tree:

```text
python scripts/record_mcp_live_client_conformance.py --receipt-dir verification/receipts
```

Run the SDK-backed advanced profile tests with:

```text
cargo test -p searchright-mcp --test advanced_mcp --all-features --locked
```

They verify current and previous-era negotiation, bounded pagination and cursor
rejection, prompt identifier isolation and completion, current-only form MRTR,
synchronous fallback without the Tasks extension, successful task completion,
cooperative cancellation, four-slot capacity rejection and recovery, and a real
aggregate task-activity resource-update notification.

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
scenario is implemented. The separate official-client harness above supplies
all-success-path local stdio conformance.

Adding `output_contract` references to the interface catalogue is an additive
metadata change. It does not change a persisted payload or remove an accepted
field, so no data migration is required.
