# Provider parser integrity and historical identity

Track 04 parser integrity work concerns newly parsed PubMed, Europe PMC,
Crossref and OpenAlex responses. It does not change the frozen wire schema,
rewrite persisted records, or establish live-provider support.

## Fail closed on malformed pages

A successful page must not silently discard a malformed row, invent an identity
from its position, or interpret malformed pagination metadata as normal search
completion. Missing or invalid required identities, duplicate identities and
inconsistent page metadata require an explicit failure, not an apparently
complete partial result. Consumers must retain the failure and must not present
the affected search as complete or use the failure as an exclusion decision.

PubMed ESearch identifiers and counts must be parsed without filtering invalid
entries into an empty success. ESummary must account for the identifiers that
ESearch requested: missing, additional, duplicate or conflicting identifiers
are not interchangeable with successful retrieval. This correspondence is a
retrieval-integrity check, not a judgment about study eligibility.

Opaque request cursors may contain sensitive material. Failure messages must
not echo their values. Redacted errors do not imply arbitrary provider metadata
or response content has been de-identified or cleared for redistribution.

## Forward-only identity corrections

New record identities must depend on provider-native identity rather than the
row's index in a page. Reordering a page or retrieving a different page must not
change the identity of the same native record or assign one positional identity
to unrelated records. Europe PMC identity must also distinguish source
namespaces; equal native IDs in different sources must remain separate.

Existing PubMed `pubmed-{pmid}` and Crossref `crossref-{DOI}` record-ID encodings
are preserved. Corrected OpenAlex and Europe PMC IDs use the provider name
followed by one or more `:<UTF-8 byte length>:<raw component>` segments. OpenAlex
uses its native ID as the component; Europe PMC uses source and then native
ID. An illustrative Europe PMC identity is
`europe-pmc:3:MED:8:00000003`. Lengths
count UTF-8 bytes, not characters; raw values are not trimmed or normalized.
Length prefixes prevent delimiter-containing components from creating ambiguous
identities. Raw `native_id` and retained provider metadata remain unchanged.
Only Europe PMC and OpenAlex change newly emitted record-ID encodings.

Missing, non-string or blank required identity components and duplicate
provider-qualified identities within a page are rejected. Optional count fields
may be absent; when present they must be unsigned integers, not null or a
different JSON type. Optional cursor fields may be absent or null to indicate
no cursor; a supplied cursor must be a nonblank string and is preserved exactly.
These structural checks alone do not establish that an upstream service has
returned every matching result.

These corrections apply to newly parsed outputs only. Preserve existing record
bytes, receipt IDs, raw-response digests, audit events and screening decisions.
Do not silently re-key a stored record, transfer a screening decision, merge
records or regenerate historical receipts using the corrected parser.

If a historical migration is needed, prepare an explicit mapping tied to the
old and new parser revisions, exact old record and receipt, retained source
provenance and new native identity. Only propose mappings where the retained
evidence establishes an unambiguous correspondence. Record the mapping as a
separate derived artifact; keep the original evidence intact and obtain the
owner's decision before applying a migration. An agent panel may advise but
cannot authorize screening-state changes or promotion.

Historical positional collisions cannot be repaired by guessing from the old
record ID or current result order. Where provenance cannot distinguish the
records, retain an explicit unresolved collision and do not migrate it
automatically. Parser identity is record identity, not proof that different
records describe the same report or study; linkage remains explicit.

## Evidence and content limits

- Deterministic fixtures and adversarial parser tests establish only the tested
  local behavior. They do not establish current provider behavior, completeness
  of an actual search, live execution, terms compliance or rights clearance.
- PubMed ESummary is not EFetch or full-text retrieval. Missing abstracts or
  other fields cannot be interpreted as evidence those fields do not exist.
- OpenAlex inverted abstract indexes are not mapped into canonical abstract
  text by this work. Retained provider metadata is not a claim of complete
  canonical field coverage.
- Byte-preserving historical retention does not authorize exporting sensitive
  metadata or redistributing licensed content.
- Existing provider, transport, downstream, methodology and owner gates remain
  separate. Neither parser success nor this document authorizes a live call,
  provider-support claim, release, cutover or archival.
