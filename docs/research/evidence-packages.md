# Versioned review evidence packages

Design updated 7 November 2026. Existing `.srpack`, RO-Crate and PROV-style
exporters are the starting point. This document specifies improvements; it does
not claim independent conformance or a completed deposit.

## Decision

Make a **review evidence package** a first-class output. A package is a citable,
versioned research object containing the declared method, inputs or retrievable
references, observations, decisions, outputs and enough execution information to
inspect or replay the work. GitHub repository creation is an optional destination,
not necessary for using the package locally.

Use a per-review repository when a review has its own protocol, authors,
publication or update lifecycle. Group repeated monitoring runs in one project
repository with immutable releases. Do not create one repository per query,
provider or run. No new repository or public deposit is created by this plan.

## Minimal standards profile

| Need | Representation | Implementation decision |
| --- | --- | --- |
| Research-object description | RO-Crate 1.3 JSON-LD, schema.org entities | Improve existing exporter and validate independently before declaring conformance |
| Execution lineage | W3C PROV entity/activity/agent relations; optional Process Run Crate profile | Model inputs/outputs as entities and executions as activities; record software/model/human identities separately |
| Search reporting | PRISMA-S source/query/date/limit details and applicable PRISMA flow/reporting outputs | Generate from observed runs/decisions; reporting completeness does not certify conduct |
| Bibliography | CSL JSON, RIS/BibLaTeX exports and persistent identifiers | Reuse existing interchange and Sourceright canonicalisation where enabled |
| Tabular data | UTF-8 CSV for portability, Parquet for larger record/decision tables | Document column types, nulls, identifiers and schema version; add Frictionless metadata only where it aids actual consumers |
| Integrity and replay | Existing `.srpack` manifest, file digests, tool/environment lockfiles and deterministic archive | Keep compatibility; do not introduce a second custom container |
| Citation and authorship | CITATION.cff and dataset metadata/DOI when deposited | Cite a specific release; distinguish authors from software agents |
| Distribution | GitHub Release assets; pinned Hugging Face datasets; optional archival DOI | OCI/ORAS distribution is a later option for deployments that already use it |

Primary standards: [RO-Crate 1.3](https://www.researchobject.org/ro-crate/specification/1.3/),
[root metadata](https://www.researchobject.org/ro-crate/specification/1.3/root-data-entity.html),
[Workflow Run RO-Crate profiles](https://www.researchobject.org/workflow-run-crate/),
[W3C PROV-O](https://www.w3.org/TR/prov-o/),
[CSL JSON](https://github.com/citation-style-language/schema),
[Frictionless specifications](https://specs.frictionlessdata.io/),
[Citation File Format](https://citation-file-format.github.io/).

Start with basic RO-Crate and a small set of PROV relations. Adopt the Process Run
Crate profile before a full workflow/provenance profile unless a real consumer
needs the latter. BagIt is optional for a receiving archive that requires it.
Avoid stacking several packaging formats that duplicate the same hashes.

## Fix existing conformance gaps first

Inspect `crates/searchright-provenance` against an independent validator and
fixtures. The current root export needs the required publication-date/licensing
metadata; obtain these from the declared package metadata rather than inventing a
licence for underlying papers. The current PROV projection must ensure
`wasDerivedFrom` connects entities, with event/activity generation represented by
the appropriate relation. Correct the use of `contentSize` for a record count:
byte size and number of retrieved records are different quantities. Test identifier
resolution, type consistency and missing
metadata, not merely whether the output is valid JSON.

W3C PROV is the provenance model; the PROV-JSON serialization is a
[Member Submission](https://www.w3.org/submissions/prov-json/), not itself a W3C
Recommendation. Name the model and serialization precisely in conformance claims.

Metadata, code, source documents and derived annotations can have different
licences. Record them separately. A repository licence never broadens rights to
included abstracts, full text or restricted datasets. Support reference-only files
with original URL, permitted identifiers, checksum when available and acquisition
instructions; clearly state what cannot be replayed without access.

## Suggested generated repository

| Path | Contents |
| --- | --- |
| `README.md`, `CITATION.cff` | Review identity, mode, authors, supported use, latest release and reproduction instructions |
| `protocol/` | Versioned question, eligibility, source plan, amendments and reporting mappings |
| `searches/` | Exact executed native queries, dates/platforms, limits, completion states and permitted response references |
| `data/` | Small portable tables or manifests pointing to exact Hugging Face commits/files |
| `decisions/` | Screening/linkage/extraction events with actor, mode, evidence status and supersession links |
| `outputs/` | PRISMA/search appendix, bibliography, extraction tables, uncertainty and synthesis drafts |
| `provenance/` | Run manifests, PROV export and source/tool/model identities |
| `reproduce/` | Locked commands/environment and fixtures needed for replay; never credentials |
| `ro-crate-metadata.json` | Standards-based inventory and relationships |
| Release asset | Complete distributable `.srpack`/archive plus checksums and validation results |

This is a generated layout, not ten manually maintained forms. Allow a compact
package containing only the components actually produced. Keep large binary data
out of ordinary Git history. Package paths are planned interfaces; expose actual
CLI/MCP commands only after implementation and help/tests exist.

## Versioning, publishing and recovery

Separate software version, package-schema version, protocol version, dataset
revision and run ID. Each release records exact Git/HF commits, model identifiers,
configuration and source observations. A branch name or a moving `latest` tag is
not a reproducible data reference. Preserve earlier releases and describe the
delta: added records, corrected metadata, changed eligibility and changed
decisions. Corrections supersede previous results rather than silently replacing
them.

Provide export/validate/replay locally first. Add a destination adapter that shows
the intended repository/assets/visibility, then publishes within the user's
authorised destination policy. A preauthorised autonomous monitor can publish
routine releases within that policy without approval for every update. A new
public destination or unexpected content class requires a concrete decision.

Use GitHub Releases for convenient distribution and optionally archive a suitable
release through Zenodo for a persistent citation. A DOI is an identifier, not a
quality certification. OCI packaging is useful only if a consumer needs registry
pulls; it should not delay the ordinary downloadable package.

Official references: [GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases),
[Zenodo GitHub integration](https://help.zenodo.org/docs/github/),
[Hugging Face repository versions](https://huggingface.co/docs/huggingface_hub/guides/download),
[ORAS](https://oras.land/docs/).

## Dataset lifecycle through the existing estate

Use the source acquisition code in its existing GitHub repository and the
canonical [Hugging Face estate registry](https://huggingface.co/datasets/edithatogo/dataset-estate-registry).
The [portfolio map](portfolio-reuse.md) identifies current sources and limits.

1. Register the dataset identity, upstream, intended use and code repository in
   the existing catalogue; avoid duplicating an already canonical collection.
2. Capture permitted source material using the existing archive/provider path.
   Distinguish actual captures from fixtures, dry runs, stubs and catalogue rows.
3. Normalize into versioned Parquet/CSV plus a dataset card and source manifest.
   Preserve identifiers and rights; record excluded material and why.
4. Publish a new HF commit/release when authorised. Reference that exact commit
   and selected files from the review package, with hashes where practical.
5. Publish reusable derived annotations separately from upstream source content.
   Keep record/report/study identity and annotation provenance.
6. Separate development, validation and confirmatory partitions by review family
   and time. Protect unobserved confirmation labels; public availability is not
   equivalent to a pristine model holdout.
7. Update the existing catalogue with supersession and source links. The registry
   carries discovery metadata; dataset files remain in their owning repositories.

Future outputs may include a query-translation corpus, linkage annotations,
recorded temporal update sequences and agent-task traces. Publish only the
rights-clear portions. A public record or DOI alone does not establish that full
text may be redistributed or used as a model input.

Prefer one proposed `edithatogo/searchright-benchmarks` dataset with documented
configurations for new rights-clear derived benchmarks, rather than a repository
per test. This name is a proposal, not an existing dataset. Keep canonical SYNERGY
and other upstream datasets in their current locations. Use a separate restricted
holdout location when the study needs unobserved labels. A review repository can
be named for its review/project; generate it from a template maintained inside
Searchright before creating a separate template repository.

## Validation and useful research

The first conformance study should use independently authored packages and an
independent consumer. Test round trips, metadata completeness, file integrity,
PROV relation types, missing restricted inputs and recovery after interrupted
publication. Test both deterministic replay and inspection-only packages.

Retain stochastic model outputs as observations. Re-running a remote model is a
new execution; byte-identical regeneration is not guaranteed. For deterministic
parts, compare outputs under the same inputs/toolchain and explain permitted
environment metadata differences.

This output can support the reproducibility/update study in the
[research programme](programme.md). The useful claim concerns successful
independent reconstruction, discrepancy detection and saved reviewer effort;
having a metadata file alone does not establish those outcomes.
