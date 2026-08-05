# Registry and publication plan

Metadata preparation is not evidence of acceptance. Every target uses the states
`not_started`, `prepared`, `validated`, `submitted`, `accepted`, `rejected` or
`deferred`, with URL, version, date and evidence.

## Targets

- GitHub repository, topics, Pages and Releases.
- crates.io (`searchright`, `evidence-search-core`, optional interface crates).
- Official MCP Registry using `server.json` and `mcp-publisher`.
- Glama using repository submission, `glama.json` and Docker/MCP inspection.
- Smithery using a Streamable HTTP URL or signed MCPB bundle.
- MCPB/desktop-extension distribution for local stdio clients.
- `awesome-mcp-servers` and the user's MCP registry/catalogue repositories.
- JOSS after research-software maturity, archived release and external users.
- Zenodo/OSF release archive and DOI after the first stable artefact.

## Approval gate

External publication is write-capable and public. The submission workflow remains
manual/approval-gated until a compiled signed release, security receipts, public
repository and truthful support matrix exist.
