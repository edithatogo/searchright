# MCP compatibility

Searchright's local standard-I/O server targets Model Context Protocol revision
`2026-07-28` through the official Rust SDK `rmcp` 3.1.2.

## Protocol profile

- Every modern request carries `io.modelcontextprotocol/protocolVersion`,
  `io.modelcontextprotocol/clientInfo` and
  `io.modelcontextprotocol/clientCapabilities` in `params._meta`.
- `server/discover` replaces the retired `initialize` / `initialized`
  handshake for the 2026 protocol era.
- Tool discovery is deterministic and each tool has an object input schema.
- Tool calls return `resultType: complete`, structured content, and the
  backwards-compatible JSON text content emitted by the SDK.
- Tool-originated failures use `isError: true`; protocol errors are reserved
  for malformed or unsupported MCP requests.
- The local server exposes no session identifier and keeps application state in
  explicit Searchright contracts rather than hidden transport sessions.

The server does not adopt deprecated Roots, Sampling or Logging features.
Resources, prompts, multi-round-trip input requests, subscriptions, Tasks, and
authenticated Streamable HTTP are separate roadmap capabilities and are not
claimed by the local stdio profile.

## Verification

Run:

```text
cargo build -p searchright-mcp --locked
python scripts/mcp_smoke.py target/debug/searchright-mcp
```

The transcript performs stateless discovery, lists all contracted tools with
per-request metadata, and invokes a read-only tool to verify a complete
structured result. This is a Searchright compatibility smoke test, not a claim
that every optional MCP feature or remote transport conformance scenario is
implemented.
