# Bounded offline PubMed EFetch citation and abstract mapping

This slice parses supplied synthetic PubMed XML into the existing
`BibliographicRecord` and `ProviderPage` contracts. It is citation-and-abstract
mapping, not article full-text retrieval, complete PubMed schema support or a
claim that a particular search retrieved all relevant reports. The existing live
provider is not switched to this parser. Constructing an EFetch URL does not
perform or authorise a request; identity/contact query parameters are not safe
to log merely because the URL uses HTTPS.

## Accepted envelope and identity

The supported envelope is `PubmedArticleSet` containing `PubmedArticle` records.
`PubmedBookArticle`, error documents and other record variants are unsupported:
reject the entire supplied page, including a mixed supported/unsupported page,
rather than silently omitting a record. This is a parser failure, not a screening
or eligibility decision.

The caller supplies a nonempty batch of distinct numeric PMIDs. The returned
identities must match that batch exactly, without duplicate, missing or extra
records. Preserve leading zeroes and document order; do not sort records into
request order. Canonical identity remains `pubmed-{PMID}`, with the same raw PMID
as `native_id` and `identifiers.pmid`. A successful page does not establish that
this batch is the entire result set of an earlier search.

## Explicit field mapping

| Canonical field | Mapping and limitation |
| --- | --- |
| `title` | Required `ArticleTitle` text, with descendant text in source order. Trim only the outer whitespace. Do not insert spaces around inline markup or infer missing words. |
| `abstract_text` | Ordered `AbstractText` sections. A nonblank `Label` renders as `Label: text`; unlabelled sections render as text. Join sections with one newline. Never turn `NlmCategory` into an invented label. Absence maps to null, not proof the publication has no abstract. |
| `authors` | Source order. Render a personal author as `LastName ForeName Suffix`, omitting absent components and using `Initials` only when `ForeName` is absent. Render `CollectiveName` directly. Mixed collective/personal identity is rejected; no author string splitting or inferred individuals. |
| `container_title` | Optional `Journal/Title`, not an inferred abbreviation expansion. |
| `publication_date` | The source `JournalIssue/PubDate` components, joined by spaces in Year, Month, Day, Season order, or standalone verbatim `MedlineDate`. This is source text, not an asserted ISO date. Mixed `MedlineDate` and component forms are rejected. |
| `publication_year` | Parse only an explicit numeric `Year`. Do not derive a year from a `MedlineDate` range or invent January/day 1 for an incomplete date. |
| `identifiers` | PMID from `MedlineCitation/PMID`; supported `ArticleIdList/ArticleId` types `pubmed`, `doi` and `pmc` map to PMID, DOI and PMCID. `Article/ELocationID` with `EIdType="doi"` also maps to DOI. Repeated mapped identifiers must agree exactly, including across the two DOI locations. Missing identifiers remain null; never infer DOI or PMCID from title or similarity. Other identifier types do not populate canonical identifiers in this subset. |
| `languages` | `Language` strings in source order; no translation, normalisation or inferred language. |
| `subjects` | MeSH descriptor text in source order. Qualifier text remains associated with its descriptor in metadata, not flattened into invented standalone subject terms. UI identifiers and major-topic flags are not represented by this canonical string list. |
| `kind` | `journal_article` for this accepted citation subset. This is not a study-design, eligibility or publication-status classification; publication-type semantics are not comprehensively mapped. |
| `urls` | Construct the PMID landing-page URL; it is not a full-text availability or access-rights claim. |
| `source_receipt_id` | `pending-receipt` before runtime binding. Parser success does not establish a real source receipt. |

Mixed title and abstract text retains decoded character order and internal
whitespace. Inline XML markup is not retained in the canonical strings, so this
is not byte-lossless formatting preservation. The supported inline subset is
`i`, `b`, `u`, `sup`, `sub`, `italic` and `bold`; unsupported nested constructs
fail closed rather than being silently treated as equivalent formatting.
Mapped scalar identity, date and name fields require leaf text, not arbitrary
nested structures. The raw-response digest binds the
supplied bytes; it does not by itself archive or license them.

## Retained metadata and deliberate omissions

`provider_metadata` identifies the format as `pubmed-efetch-xml` and retains:

- Ordered abstract sections with label, category, decoded text and section
  attributes, so the flat abstract does not erase section boundaries.
- Ordered personal/collective author components, including both supplied given
  names and initials even when only one appears in the rendered author string.
- Supplied publication-date components, without manufactured precision.
- MeSH descriptor/qualifier text associations.
- ELocationID type, value and attributes, including values not mapped to a
  canonical identifier.

This is a selected metadata projection, not the original XML tree. It does not
claim complete citation metadata, author affiliations, ORCID, corrections,
funding, publication histories, other abstracts, full-text structure or all
identifier types. Unmapped metadata is not evidence that the source lacks it.
Preserving metadata also does not grant redistribution rights or establish
de-identification.

Missing optional containers differ from malformed supplied ones. A present
Abstract must contain AbstractText; a present AuthorList, MeshHeadingList or
ArticleIdList must contain supported entries. Empty or malformed mapped
containers fail rather than becoming apparently absent metadata. Abstract
CopyrightInformation is tolerated but not mapped or interpreted as a rights
decision. A supplied PubDate without a supported component is rejected.

## Parser and execution boundaries

The bounded parser rejects malformed XML, DTDs, custom entities and processing
instructions; it does not retrieve DTDs or external entities. The source limits
are 8 MiB input, depth 64, 1,000 records, 100,000 elements, 64 attributes per
element, 64 KiB per mapped field/attribute and 4 MiB cumulative decoded tree text
(ancestor copies count). The mapped-field cap is checked after tree assembly;
the aggregate decoded-text cap is enforced before appending text. These bounds
are implementation limits, not provider
quotas or claims of comprehensive upstream compatibility.

`total_available` means the number of citations parsed in this supplied batch;
`next_cursor: null` does not prove the originating search is complete. The raw
response digest is BLAKE3 over the exact supplied UTF-8 XML bytes, before XML text
decoding. An actual retrieval path still requires request/response correspondence,
runtime receipt binding, transport controls and separate authorised execution.

## Acceptance evidence required

The separate `runtime_receipts.rs` fixture tests exercise parsed pages through
the actual registry and in-memory page cache. All seven fixture-runtime tests
passed with the budget fix, within a full local validation snapshot recorded in
`verification/receipts/track-04-full-validation.json`. That Homebrew 1.98.0 run
predates the cache-version fix; new repository-pinned 1.97.1 full validation is
pending. The current version-declared fixture tests pass in the 61-test focused
Homebrew run recorded in `verification/receipts/track-04-cache-version-panel.json`.
Their bounded evidence is freshly issued receipts bound
to returned records, cache-envelope integrity and synthetic replay—not live
transport. Synthetic cursor scheduling is explicit, and fixture-added raw hashes
are not evidence of an end-to-end raw-response hash chain into issued receipts.
Replay compares bibliographic payloads after normalising only the new receipt
binding. Cache corruption detection assumes a trusted backend; the test namespace
is not proof of authenticated tenancy.

Review complete serialized `ProviderPage` golden output manually against the
synthetic XML: every identifier, field, null, collection, metadata association,
record/section order, digest, count, cursor and pending receipt. Do not generate
expected output from the implementation under test. Include mixed-content text,
labelled/unlabelled abstracts, partial dates/ranges, collective/personal authors,
identifier agreement/conflicts, malformed mapped content and unsupported book
records. Error tests must demonstrate whole-page failure rather than silent
partial success.

Local fixtures and compiler tests do not close live-support, provider-policy,
rights/currentness, methodological adequacy or article-full-text gates. No wire
schema or historical record is changed by this parser; any future persisted-data
migration or live-path switch requires its own explicit evidence and authority.
