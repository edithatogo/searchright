---
license: apache-2.0
language:
  - en
pretty_name: Searchright Synthetic Methodology Fixtures
size_categories:
  - n<1K
tags:
  - systematic-review
  - scoping-review
  - evidence-synthesis
  - information-retrieval
  - synthetic-data
  - benchmark
  - searchright
  - text
---

# Searchright Synthetic Methodology Fixtures

This dataset is a deterministic, rights-clear package of small synthetic fixtures
used to test Searchright's systematic-search contracts and review-methodology
infrastructure.

## Status

**Prepared for publication; not yet published or externally validated.**

The canonical source is the `edithatogo/searchright` repository. A protected
release workflow may create or update this dataset only after an explicit owner
approval. A prepared dataset card, manifest or dry run is not Hub publication or
benchmark acceptance.

## Included material

- native-query examples for PubMed, Ovid MEDLINE, Embase, CINAHL/EBSCO,
  PsycINFO/Ovid, Scopus and Web of Science;
- synthetic duplicate-clustering cases;
- synthetic report-to-study linkage cases;
- synthetic screening-prioritisation cases;
- synthetic living-review change cases;
- a PRISMA flow arithmetic example;
- the Searchright methodology benchmark protocol and per-file integrity manifest.

All included records and query examples were authored for repository testing.
The package excludes sealed test labels, licensed database content, provider
responses, credentials, personal data and external benchmark corpora.

## Intended uses

The fixtures may be used for:

- parser and source-span regression tests;
- deterministic query round-trip checks;
- duplicate/linkage algorithm tests;
- advisory screening-ranking tests;
- PRISMA arithmetic checks;
- living-review lineage tests;
- SDK, CLI and MCP examples that require no live provider access.

## Prohibited interpretations

The dataset does **not** establish:

- cross-database semantic equivalence;
- search sensitivity or comprehensiveness;
- external deduplication, linkage or screening performance;
- methodological validity;
- production safety;
- state-of-the-art performance.

Screening labels are synthetic benchmark targets. Searchright agents remain
advisory and may not use a ranking score or benchmark label to make an
irreversible final exclusion.

## File integrity

`dataset-manifest.json` records every distributed path, SHA-256 digest, byte
length, media type, source path and rights statement. Consumers should verify the
manifest before use. The canonical build is deterministic: repeated builds from
the same Searchright tree produce the same file bytes and dataset root digest.

## Splits and leakage controls

Only development/validation fixtures with visible synthetic labels are included.
The repository's sealed-test manifest and all hidden labels are excluded. Final
methodological metrics require separately mounted labels and an evidence-bound
execution receipt.

## Licence

The packaged fixtures are distributed under Apache-2.0. External upstreams named
in the benchmark protocol, including SYNERGY, are not redistributed in this
package and retain their own licences and provenance.

## Citation

Cite the exact Hugging Face dataset revision and the corresponding Searchright
Git revision recorded by your execution receipt. A floating `main` reference is
not sufficient for reproducible evaluation.
