# Searchright

<!-- mcp-name: io.github.edithatogo/searchright -->

**Programmable, auditable systematic literature searching — from a research question to a versioned, reusable corpus.**

Searchright is a Rust-core research tool for designing, translating, validating, executing and updating multi-source literature searches. It preserves source observations, search strategies, record identity, provenance and the information needed to independently inspect a search. Its outputs are intended for evidence synthesis **and** bibliometric/scientometric analyses.

> **Project status: technical alpha.** Source implementations, deterministic fixtures, contracts, Rust API, CLI, local MCP server and agent skill exist. A complete live multi-source search-to-consumer workflow, methodological superiority, independently validated reproducibility, production hosting and publication/registry acceptance are **not** established. Consult [the evidence model](docs/vertical-slice-definition-of-done.md), [Conductor](conductor/tracks.md) and [GitHub Actions](https://github.com/edithatogo/searchright/actions) for revision-bound evidence. The [historical status snapshot](PROJECT_STATUS.md) (source: `verification/receipts/project-status-snapshot.json`) records the **observed 10 September 2026 main revision** `9b4a4a4dba818d75f9dbb9ee6be458871b09a878` and **last fully admitted historical revision** `10f983d44e67c08be44131ea6e2b7cf75dc147f6`. It is dated September 2026 and is **not** a live dashboard.

## What Searchright does

| Stage | Purpose | Evidence boundary |
| --- | --- | --- |
| **Plan** | Record the research question, concepts, sources, limits, native strategies, changes and approvals | A validated protocol is not proof of a comprehensive search |
| **Translate and inspect** | Parse and compile source-specific strategies; surface unsupported features and possible semantic loss | Cross-database equivalence is conditional, not universal |
| **Acquire** | Apply bounded provider policies, paging, retry, cancellation, receipts and replay | Fixture-backed connectors do not imply authorised live access |
| **Reconcile** | Preserve source observations; conservatively deduplicate records while retaining identity links and disagreements | Bibliographic publications, reports and underlying studies remain distinct |
| **Export** | Produce a canonical CSL-JSON bibliography with versioned provenance/metadata sidecars and derived consumer views | CSV, RIS, BibTeX, Parquet and consumer files are **projections**, not competing canonical references |
| **Update and audit** | Track new/changed records, protocol amendments, update lineage, uncertainty and research artefacts | Historical exact replay needs preserved source responses or snapshots |

The **primary product boundary is systematic search and corpus construction**. Searchright is not intended to replace ASReview's screening/active learning, pybibx's analysis, bibliometrix, VOSviewer, or specialised synthesis tools. It supplies their inputs; optional adapters are independently qualified against real consumer versions.

### Canonical references and provenance

**CSL-JSON is the authoritative bibliographic representation.** Canonical reconciliation and reference integrity belong with [Sourceright](https://github.com/edithatogo/sourceright). Searchright retains stable CSL IDs, source-specific record identifiers, provenance and identity crosswalks. Structured cited-reference edges, authorship–affiliation relations, field-specific source observations, citation counts, licence/access restrictions and run histories belong in **versioned sidecars**, keyed to canonical references and corpus revision; they are not silently compressed into a bibliographic field.

Original source records remain available according to their rights and retention policies. Derived exports must document omitted or transformed fields and preserve a reversible mapping to canonical records where supported. Missing metadata is not interpreted as zero.

## Downstream research tools

| Consumer | Intended handoff | State |
| --- | --- | --- |
| **ASReview** | Deduplicated, identifier-stable title/abstract dataset; optional dataset/subcommand extension and return-event reconciliation | **Planned and under qualification**; Searchright does not implement competing screening models |
| **pybibx** | Separately qualified source/neutral import with stable identifiers and explicit information-loss reporting | **Planned**; no fabricated Scopus/WoS exports |
| **bibliometrix / VOSviewer** | Corpus plus citation-edge, author/affiliation, source and missingness sidecars, translated to supported consumer formats | **Planned**, real-consumer parity not established |
| **Other tools** | Standard, versioned CSL-JSON and neutral provenance-aware projections | Contract-first interface; each adapter must demonstrate compatibility |

The current implementation includes historical screening, ranking and reporting crates. These remain as compatibility and evaluation surfaces until migration is evidenced; **their presence does not change the intended ownership boundary**.

## Architecture

```text
Review question and source strategy
            │
   Searchright orchestration / Rust CLI / MCP / agent skill
            │
  evidence-search-contracts + evidence-search-core
            │
       Provider observations
            │
 Searchright source reconciliation and provenance
            │
      Canonical CSL-JSON  + versioned sidecars
            │
    ┌───────┼─────────┬────────────┐
 ASReview  pybibx  bibliometrix  Other consumers
```

- **[CiteWeft](https://github.com/edithatogo/citeweft)** owns document extraction and source spans; it is an optional one-way integration.
- **[Sourceright](https://github.com/edithatogo/sourceright)** owns canonical reference verification and reconciliation.
- **[Searchright](https://github.com/edithatogo/searchright)** owns systematic-search orchestration, search-specific evidence and corpus preparation.
- **`evidence-search-contracts` and `evidence-search-core`** provide neutral query/provider/receipt/runtime interfaces shared with appropriate consumers.

Network and consequential actions are deny-by-default, with explicit authorisation, budgets and source rights. Agent recommendations cannot silently amend strategies or assert human screening decisions. No source index is assumed immutable.

## Try the existing local interfaces

The following are source-backed CLI examples; they exercise local contracts and compilation, **not a validated live systematic review**. The workspace pins Rust 1.97.1 and uses a committed `Cargo.lock`.

```bash
./scripts/bootstrap.sh
cargo test --workspace --locked

cargo run --locked -p searchright-cli -- validate-plan contracts/examples/review-plan.yaml
cargo run --locked -p searchright-cli -- compile contracts/examples/search-strategy.yaml --dialect pubmed
cargo run --locked -p searchright-cli -- prisma contracts/examples/prisma-flow.json --format mermaid
cargo run --locked -p searchright-mcp
```

For broader repository verification run `./scripts/verify.sh` after installing the documented prerequisites. See [the docs index](docs/README.md), [architecture](docs/architecture.md), [contracts](docs/contracts.md) and [vertical-slice definition of done](docs/vertical-slice-definition-of-done.md). Live access requires separate provider policy, explicit authority and actual execution evidence.

## Research and development direction

Current priority is a **working two-source search → canonical corpus → independently consumable export**, including resumption, provenance and correct error reporting, before adding further architectural frameworks. The actively evolving [frontier roadmap](conductor/frontier-plan.md) and [research programme](docs/research/programme.md) cover search-translation fidelity, error/loss in reconciliation, the effects of corpus construction on bibliometrics, temporal search updates and external reconstruction.

The [CSL-first refinement and SEARCH-REPLAY protocol (PR #799)](https://github.com/edithatogo/searchright/pull/799) describe the proposed boundary and a metaresearch study of missing details in published search strategies, plausible reconstructions and retrieval uncertainty. **Draft planning is not implemented functionality, a registered study or an accepted publication.**

Existing libraries and frameworks are preferred **only after** demonstrated behavioural parity and a practical advantage in maintenance, security, performance or interoperability. OxiDD, Salsa and DVC are candidates, not mandatory dependencies; optional TLS attestation, advanced semantic reasoning and workflow integrations have separate value and evidence gates.

## Evidence, maturity and limitations

Searchright separates *contracted*, *source implemented*, *compiler verified*, *fixture proven*, *authorised live proven*, *externally validated* and *publicly accepted*. A passing static check does not prove a live database search; a matching hit count does not prove identical records; a portable research package does not establish search completeness.

The repository's Conductor inventory currently includes **44 track identities**, with historical archived tracks and newer scaffolded tracks, and **230 acceptance assertions** in the current evidence-debt inventory. These counts are planning/traceability facts, not measures of research validity; see [the canonical track index](conductor/tracks.md) for changes.

Searchright does not certify whole-literature recall, scientific truth, medical decisions, legal compliance or journal acceptance. It does not bypass licensed access, silently discard ambiguous records or claim autonomous human-equivalent systematic reviews.

## Licence

MIT OR Apache-2.0. Third-party source records, abstracts, full texts, data and proprietary database access retain their own terms.
