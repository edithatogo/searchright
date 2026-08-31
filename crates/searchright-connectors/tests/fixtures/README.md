# Synthetic canonical-page expectations

These four JSON files were authored field by field from the existing synthetic
`provider-fixtures/mvp` inputs and the declared parser mapping, not captured from
parser output. Do not automatically regenerate them to accommodate a failure.

The comparison covers the complete serialized page and every record field,
including explicit nulls, empty collections, provider metadata, order, counts,
cursors and the pre-execution `pending-receipt` placeholder. PubMed and Crossref
retain their existing stable provider-prefixed record IDs. Europe PMC and
OpenAlex use the corrected provider-qualified UTF-8 byte-length-prefixed identity
encoding.

- PubMed: two UID-ordered summary records, leading-zero identifiers preserved,
  source author strings and dates, one DOI and one absent DOI, constructed PubMed
  URLs, no abstract. The summary page count is two, not an ESearch universe count.
- Europe PMC: source `MED` qualifies the record identity while the native ID
  remains unchanged; PMID, PMCID, DOI, one unsplit source author string and the
  supplied abstract are retained. No publication date or URL is inferred.
- Crossref: DOI identity, first title/container strings, `D, Fixture` author
  rendering, and year from the supplied online date. The full publication date
  remains unmapped, while the original date-parts survive in provider metadata.
- OpenAlex: native work-URL identity, DOI URL-prefix removal, fallback from absent
  `display_name` to the supplied `title`, author/container/landing URL and source
  date. The inverted abstract index remains in provider metadata; `abstract_text`
  deliberately remains null rather than implying reconstruction.

These are local synthetic field-fidelity and determinism checks. They are not
upstream-currentness, live transport, EFetch/full-report, recall, methodology,
rights approval or historical record-ID migration evidence. Actual source-receipt
binding occurs after parser execution and requires separate runtime tests.
