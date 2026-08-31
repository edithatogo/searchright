# Derived reporting formats

`searchright_prisma::render_flow` validates the same `PrismaFlow` arithmetic
before producing Markdown, JSON, Mermaid, accessible SVG text tables, Typst
table source or semantic HTML tables suitable for word-processor import.
HTML output is **not a DOCX archive**; no claim of Microsoft Word import or
Typst compilation is made without a separate renderer receipt. SVG is an
original text-table presentation, not a reproduction of an official template.

The JSON projection preserves the complete v1 flow, selected new/update context,
prior-review identifier and claim boundary. Human formats include every count
and exclusion reason in stable input order. Text metacharacters are escaped;
control characters are replaced in presentations but retained in JSON.
Mermaid includes a text-equivalent comment table; SVG includes title/description.
These transformations never modify canonical state, submit or certify anything.

An updated flow requires distinct prior-review lineage. The v1 contract cannot
express separate historical and newly included cohorts: these renderers do not
invent them or claim a complete PRISMA updated-review template. Existing facade,
CLI and MCP Mermaid behavior is unchanged; these new formats are library-level
presentation adapters, not newly advertised CLI/MCP operations.

## Remaining acceptance evidence

The PRISMA.jl companion packet retains Searchright arithmetic ownership and
identifies ceceoco/PRISMA.jl separately from the personal fork. Completing parity
requires exact producer/consumer revisions, fixture and output hashes, a
field-by-field difference receipt and accountable fork maintenance disposition.
No upstream code or standards text was imported and no external parity or
maintenance acceptance is inferred from these local tests.

Independent PRESS review remains a separate gate: representative native
strategies and generated appendices need reviewer identity, all six domains,
findings, responses and explicit decision tied to exact artifact versions.
Reporting arithmetic and local rendering tests cannot provide that approval.
