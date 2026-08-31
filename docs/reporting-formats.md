# Derived reporting formats

`searchright_prisma::render_flow` validates the same `PrismaFlow` arithmetic
before producing Markdown, JSON, Mermaid, accessible SVG text tables, Typst
table source or semantic HTML tables suitable for word-processor import.
HTML output is **not a DOCX archive**; no Microsoft Word import is claimed.
Typst 0.15.1 compiled the normal and hostile synthetic fixtures to PDFs, as
recorded in `verification/receipts/track-08-render-formats.json`. This proves
compilation for those exact sources, not visual layout, browser rendering,
accessibility conformance or external template parity. SVG is an
original text-table presentation, not a reproduction of an official template.

The JSON projection preserves the complete v1 flow, selected new/update context,
prior-review identifier and claim boundary. Human formats include every count
and exclusion reason in stable input order. Text metacharacters are escaped;
control characters are replaced in presentations but retained in JSON.
Mermaid includes a text-equivalent comment table; SVG includes title/description.
These transformations never modify canonical state, submit or certify anything.

Reproduce the local Typst smoke by running
`cargo run -p searchright-prisma --example render_typst_fixture --locked`, saving
its stdout as a `.typ` file and running `typst compile` on that file. Repeat with
`-- --hostile` to include inert quotation, slash and command-like text. This
example is an explicit local diagnostic; no renderer is invoked by the library.

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
