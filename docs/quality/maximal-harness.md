# Maximal verification harness

Searchright uses an evidence-layered harness rather than one undifferentiated
"tests passed" claim. The machine-readable matrix is
`verification/harness-matrix.json`.

## Always-on source gates

The network-free static harness currently executes 51 gates. A 53-command
catalogue also covers auxiliary traceability commands and assigns every command
an explicit evidence ceiling. The harness validates contracts and examples,
Conductor coverage, evidence debt, architecture fitness, GitHub issue
projection, schema migration policy, provider policy manifests, receipt
redaction, local recovery mechanics, integration passports, consumer-driven
contract suites, context hashes, default-deny policy, workflow hardening,
CLI/MCP/facade parity, migration packets, Rust source structure, secret
signatures, the source SBOM and reproducible archives.

These gates establish only **source-verified** evidence.

The gate catalogue is intentionally not a scorecard. It records what a command
can prove and prevents static results from being promoted to compiler, live,
legal, methodological, operational or external evidence. The derived
`verification/evidence-debt.json` register exposes the remaining proof work.

## Compiler-backed gates

The pinned Rust workflow adds formatting, strict Clippy, unit, integration,
documentation, property and metamorphic tests on Linux, macOS and Windows. It
requires a committed `Cargo.lock` and runs at the declared Rust version.

Coverage is gated at 91% line coverage. Mutation testing is scheduled, while
new and changed contract surfaces are fuzzed with persisted corpora.

## Adversarial and formal gates

- Kani proves bounded workflow-authority properties.
- Loom explores concurrent single-writer interleavings.
- Miri probes undefined behaviour in the assurance core.
- `cargo-careful` enables additional standard-library precondition checks.
- Fuzz targets exercise query parsing/compilation, CiteWeft-compatible document
  evidence, and audit-event validation.
- Hostile-record, SSRF, secret-redaction and prompt-injection cases remain
  explicit security-test families.

A successful bounded proof is not a proof of the whole product. Each receipt
must state its model, scope and assumptions.

## Clean-room and release gates

The clean-room workflow vendors the locked dependency graph, extracts the
reproducible source archive into a fresh directory, disables the network,
builds twice, compares exact binaries, installs the CLI and MCP server, and runs
an MCP discovery transcript. Release source, SBOM and binaries receive GitHub
artifact attestations and a checksum ledger.

## External evidence

Live-provider capability, licensed-source behaviour, information-specialist
review, screening calibration, accessibility, usability and registry acceptance
cannot be produced by repository source alone. Their Conductor phases remain
open until independent receipts exist.
