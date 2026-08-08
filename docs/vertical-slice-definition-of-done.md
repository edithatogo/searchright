# Vertical-slice definition of done

Searchright does not treat a crate, connector name, schema, Conductor track, or
GitHub issue as evidence that a user-visible capability works. A vertical slice
is complete only when one research workflow crosses every applicable boundary
below without hidden manual repair or an unsupported claim.

## Required path

```text
review question and eligibility contract
  -> native source strategy retained byte-for-byte
  -> source-aware parse/compile with translation-loss report
  -> bounded provider execution or deterministic replay
  -> raw-response digest and redacted source receipt
  -> canonical record normalisation
  -> deduplication and report/study linkage
  -> human-governed screening and conflict resolution
  -> PRISMA/PRISMA-S evidence ledger
  -> deterministic review-state snapshot
  -> portable, verified .srpack review bundle
  -> equivalent CLI, Rust facade and MCP operation
```

## Evidence ladder for a slice

| Level | Minimum evidence | Permitted statement |
| --- | --- | --- |
| Contracted | Versioned inputs, outputs, invariants and failure semantics | The capability is specified. |
| Scaffolded | Source paths and interface placeholders exist | Scaffolding exists; behaviour is not claimed. |
| Source implemented | Every acceptance assertion maps to symbols and deterministic tests | Source implementation is mapped; compilation is not claimed. |
| Compiler verified | Lockfile, format, Clippy, build and tests pass at declared MSRV and current stable | The checked revision compiles and passes its local test suite. |
| Fixture proven | Rights-clear provider and workflow fixtures pass end to end, including failure cases | The declared fixture path works reproducibly. |
| Opt-in live proven | Redacted live canary, rate-limit, retry, pagination and cancellation receipts pass | The named provider worked at the recorded date and API version. |
| Externally validated | Independent information-specialist or methodological evaluation passes | The evaluated capability achieved the recorded external result. |
| Publicly accepted | Package, registry or publication shows the exact version | The exact artifact is publicly listed or accepted. |

No level inherits evidence that was not actually run. In particular, a live
provider response does not prove methodological adequacy, and a passing static
schema test does not prove Rust behaviour.

## MVP connector slice

A PubMed, Europe PMC, Crossref or OpenAlex connector reaches fixture-proven only
when all of the following are evidenced:

1. HTTPS host and redirect policy are explicit.
2. Query, cursor/offset, page size and source-specific limits are represented.
3. Total-run and per-request budgets, cancellation, retry and `Retry-After`
   behaviour are bounded.
4. At least one success, empty, malformed, pagination, rate-limit, transient
   failure and permanent failure fixture exists.
5. Raw response bytes are hashed before normalisation and retention policy is
   explicit.
6. Normalised identifiers, dates, authors, titles and source metadata have
   golden expectations.
7. The source receipt carries provider/runtime/parser versions and redacts
   credentials and contact information.
8. Default CI is replay-only; live canaries are opt-in and redacted.
9. CLI, facade and MCP produce contract-equivalent summaries and resource links.
10. The track traceability file maps each assertion to the exact evidence.

## Human-authority invariant

Agents may plan, rank, explain, identify conflicts and propose screening
recommendations. Only an authorised human event may create a final inclusion or
exclusion decision. Derived snapshots must reject attempts by a non-human actor
to claim final authority, while preserving the rejected event in the immutable
audit record.

## Change and drift

A completed slice is not permanently complete. Promotion is suspended when:

- a provider baseline or response shape drifts;
- a schema, WIT, OpenAPI or MCP surface changes without an explicit baseline
  update;
- a consumer contract fails;
- a licence or data-use basis becomes unclear;
- a benchmark label leaks into development context;
- a dependency, toolchain or source receipt can no longer be reproduced.

The source of truth for current status is assertion-level traceability plus the
latest evidence receipts—not the roadmap prose or issue title.
