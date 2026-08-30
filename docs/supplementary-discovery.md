# Supplementary discovery methods

Searchright represents supplementary discovery as bounded, reproducible evidence.
It does not claim that citation graphs, trial registers, repositories, websites,
contacts, or handsearches are exhaustive. Records found by these methods require
human release before entering screening.

## Source-specific methods

The machine-readable catalogue is
`contracts/fixtures/discovery-source-methods.json`. It covers:

- ClinicalTrials.gov, WHO ICTRP, and ANZCTR;
- OSF, Zenodo, Figshare, Dataverse, and explicitly named institutional
  repositories;
- conference/proceedings, thesis, policy, and organisational website searches;
- fixture-backed OpenCitations forward chaining and documented manual backward
  reference checking over declared seed reports and bibliography ranges;
- contact and handsearch logs.

Every source declares a reproducible procedure and at least one limitation.
Manual methods must record the exact query, browse path, contact template, or
handsearch range, plus structured scope details and the source's total result
count. Search dates, inspected-result counts, candidate identifiers, operator
role, and observed limitations are retained. Contact logs instead record a
privacy-safe response outcome and optional follow-up date. They record a
recipient role, not contact details or personal identifiers.

The OpenCitations adapter is fixture-only by default. Any live execution must
use the shared provider runtime, explicit host and response budgets, a separate
live opt-in, and a redacted receipt. The catalogue does not authorise network
access and is not evidence of live source support.

## Coverage and risk reporting

Citation candidates are ordered by minimum seed distance, then identifier,
before the record limit is applied. Evidence includes each edge supporting a
walk from any seed to that candidate within the declared depth. Walks may
contain cycles; this is not enumeration of simple paths or an exhaustive-search
claim. Edge and seed input order cannot change successful results.

Graph contracts cap depth at 8, edges and released records at 100,000 each,
seeds at 10,000, each identifier/custom-method label at 512 UTF-8 bytes, and
aggregate run/edge identifier bytes at 16 MiB (including repeated values).
Traversal additionally permits at most 1,000,000 charged operations and 100,000
output evidence memberships across all retained candidates. Exceeding either
traversal budget returns `DiscoveryError::ResourceLimit` for the entire call;
no partial candidates or silently truncated evidence are returned. These are
stricter validation limits and an additive error variant, not a persisted wire
schema change or an automatic migration. Invalid stored runs need explicit
review; they are not rewritten.

Each canonical source receives exactly one coverage assessment. An executed
source requires a matching method log, and a logged source must be marked
executed. Caller-supplied subsets cannot redefine completeness. An unexecuted
source cannot be rated low risk. Rationales must state topic-specific access, indexing, date, language,
format, and interface limitations where applicable.

Typical residual risks include registry updates after the search date,
federated repository gaps, dynamic website indexes, inaccessible or withdrawn
conference material, embargoed theses, selective contact response, and
incomplete citation graphs. A completed matrix describes what was done and what
may be missing; it does not certify methodological comprehensiveness.

Log identifiers, candidate identifiers and coverage rationales use the same
conservative sensitive-pattern screening as method text. This is not
de-identification: it can reject benign text and cannot detect every identity
or secret. Callers must still minimise and review content before persistence.

## Prohibited behaviour

- Do not scrape a source whose interface, licence, robots policy, or access
  controls do not permit the operation.
- Do not bypass authentication, rate limits, export limits, or technical access
  controls.
- Do not place credentials, contact identities, query-bearing URLs, cookies, or
  licensed response bodies in fixtures or receipts.
- Do not infer that a non-response, zero-result search, or citation-graph
  boundary proves that no eligible study exists.
- Do not add discovered records to screening without the declared human release.

## Reporting language

Report the named sources, exact dates, exact method text, filters, limits,
inspected counts, and candidate counts. Describe OpenCitations and other graph
expansion as bounded citation discovery. The historical simulated panel remains
a fixture-based simulation instrument. Actual isolated agent-panel reviews are
recorded separately, preserve findings and dissent, and return recommendations
to the repository owner for decision. Topic-specific agent-panel evaluation and
the owner's recorded decision are still required before adequacy claims; no
second person is required and agents do not authorize promotion.
