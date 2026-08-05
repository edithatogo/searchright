# Official MCP Registry packet

`/server.json` targets the 2025-12-11 schema and the name
`io.github.edithatogo/searchright`. Validate with the current `mcp-publisher`
only after the OCI package exists. Publish is approval-gated by
`scripts/submit-registries.sh` and must update `registry/status.json` with the
public listing URL, version and date.
