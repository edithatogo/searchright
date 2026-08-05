# Smithery packet

Current Smithery publication paths are:

1. a public Streamable HTTP URL (OAuth when authentication is required); or
2. a pre-built MCPB bundle for a local stdio server.

Searchright currently targets the MCPB path. Build signed platform binaries,
assemble and test `dist/searchright.mcpb`, then run:

```text
smithery mcp publish dist/searchright.mcpb -n @edithatogo/searchright
```

The bundle and submission are not claimed in this scaffold.
