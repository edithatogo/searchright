# Abuse and misuse cases

- Supply a paper containing instructions that attempt to invoke MCP tools.
- Configure a provider URL targeting localhost, link-local or cloud metadata.
- Insert API keys into query strings, fixture files or audit fields.
- Ask an agent to exclude every record below an opaque score.
- Mark prepared registry metadata as accepted or publicly listed.
- Use CiteWeft fields as canonical citations without review or verification.
- Retain or export complete full text when only spans/metadata are needed.
- Run issue sync without explicit apply authority or against a dirty tree.
- Automatically advance an upstream revision because drift was detected.
- Delete legacy Sourceright code before dual-run and rollback evidence.

Every case has at least one source-level denial or review gate. Runtime,
fault-injection and human-factor evidence remains tracked as higher-level work.
