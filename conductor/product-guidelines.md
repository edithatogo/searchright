# Product Guidelines

## Voice and claims

Use precise, sceptical and operational language. Say `supports`, `records`,
`checks` or `generates`; do not say `guarantees comprehensive`, `certifies
PRISMA compliance`, `fully automates` or `eliminates reviewer error`.

Every public capability must name its evidence level:

- **contracted** — schema/specification exists;
- **scaffolded** — source surface exists but is not runtime-proven;
- **fixture-backed** — deterministic local evidence passes;
- **opt-in live proven** — authorised provider smoke evidence exists;
- **publicly accepted** — external registry or publication lists the artefact.

## Human authority

Agents may propose questions, eligibility rules, sources, terms, translations,
prioritisation and explanations. Agents cannot silently amend a protocol,
activate live/licensed sources, delete records, resolve conflicts or make final
full-text exclusions. Authority is encoded in contracts and audited.

## Accessibility and interoperability

- JSON/YAML are canonical machine surfaces; human-facing reports are derived.
- Error messages name the failed contract, path, corrective action and whether
  partial output is safe.
- Mermaid diagrams must have text equivalents.
- CLI JSON output remains stable and scriptable.
- MCP tools expose effect and authority annotations in the catalogue.

## Methodological posture

PRISMA-S is a reporting standard, not proof that a search was well conducted.
Methodological checks combine review-type guidance, source-selection rationale,
PRESS-style peer review, sensitivity testing and complete reporting.

## Security and privacy

- No live network access by default.
- No secret is serialised to receipts, logs, errors or MCP results.
- Provider hosts are allowlisted and private/link-local destinations denied.
- Full text and reviewer identities are minimised and retained by declared policy.
- Derived outputs never overwrite canonical state without preview and explicit
  apply semantics.
