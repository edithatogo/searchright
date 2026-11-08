# Standards and provenance map

## Evidence-synthesis standards

| Standard/framework | Use in Searchright | Contract role |
| --- | --- | --- |
| PRISMA 2020 | Review reporting and flow counts | Flow and report ledgers |
| PRISMA-S | Detailed search reporting | 16-item executable report ledger |
| PRISMA-ScR | Scoping-review reporting | Review-kind policy pack |
| PRISMA-LSR | Living-review updates | Update/amendment policy pack |
| PRISMA-P | Protocol reporting | Review-plan completeness checks |
| PRESS 2015 | Peer review of electronic searches | Strategy review workflow |
| Cochrane/MECIR search practice | Conduct and selection guardrails | Default methods policy |
| JBI evidence-synthesis guidance | Scoping/systematic methods | Optional methods policy |
| PICO/PCC/SPIDER/PEO | Question decomposition | Framework-neutral question contract |

PRISMA-S is represented as reporting completeness, not proof that the underlying
search was methodologically adequate.

Validated PRISMA flow counts can be rendered as canonical JSON, Mermaid,
accessible Markdown, self-contained accessible SVG, Typst, or a semantic HTML
table suitable for DOCX editors. The same formats are exposed through the Rust
facade, CLI and local MCP server. A full source appendix preserves the exact
compiled query with its source/platform, search timestamp, compiler identity and
recorded limits; generation does not validate term choice.

PRESS readiness fails closed unless findings cover all six domains, material
findings are resolved, and the execution approver is identified separately from
the PRESS reviewer. The checked-in six-domain record is a workflow fixture, not
an independent methodological review. External PRESS review therefore remains
a distinct evidence gate.

The bounded PRISMA.jl comparison and canonical-ownership decision are documented
in [PRISMA.jl interoperability boundary](prisma-jl-interop.md).

## Technical standards

- JSON Schema 2020-12 for canonical data contracts.
- OpenAPI 3.1 for a future HTTP surface.
- MCP 2026-07-28 with compatibility negotiation for 2025-11-25 clients.
- WIT/WASI component model for sandboxed provider plugins.
- CSL JSON, RIS, BibLaTeX and EndNote XML interoperability.
- W3C PROV concepts and RO-Crate packaging for provenance exports.
- SPDX/SBOM, SLSA provenance and Sigstore signatures for releases.

## Source handling

Checklist text and diagrams must retain source, version, licence and checksum
metadata. The `standards_check` repository is the preferred upstream source for
normalised checklist artefacts; Searchright stores machine-executable mappings,
not unattributed copies.
