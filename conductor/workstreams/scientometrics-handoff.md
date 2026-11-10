# Versioned scientometrics handoff

**Status:** contract/design proposal, not an implemented producer, admitted
consumer or compatibility claim. Primary owner: Track 41 (`SR-097`). Affected
owners: 01 (neutral contracts), 06 (interchange), 13 (consumer integration),
21 (licensed sources), 25/26 (provenance/evolution), 44 (VOI handoff). This is one
cross-repository delivery unit under existing ownership, not a new track/issue
hierarchy. Historical archived acceptance remains unchanged.

The downstream working name is `bibx`; CiteLattice is proposed, not selected.
No repository rename, release, data acquisition or new licensed access is implied.

## Requirements and boundaries

| ID | Priority | Contract/outcome | Existing owner |
| --- | --- | --- | --- |
| SH-01 | Must | Versioned source-neutral stream; publication CSL is preserved losslessly | 01, 06 |
| SH-02 | Must | Epoch/sequence, source revisions, changes, canonical digest, replay capabilities | 41, 26 |
| SH-03 | Must | Rights and source coverage survive every handoff | 13, 21, 25 |
| SH-04 | Should | Typed grant/patent/trial/policy/dataset extensions, without fabricated metadata | 01, 21 |
| SH-05 | Must | Exact schema/version negotiation and independent producer/consumer fixtures | 13, 26 |
| SH-06 | Should | Arrow/Parquet handoff for bounded batches without claiming end-to-end zero-copy | 06 |
| SH-07 | Must | Downstream calculations/tables/scenes/reports point to exact source snapshots | 25, 41 |
| SH-08 | Could | Innovation forecasting/VOI study consumes as-of evidence; no citation-to-utility shortcut | 44 |
| SH-09 | Won't | Searchright reimplements bibliometrics, scientific graphics or innovation/VOI engines | 01, 44 |

Searchright owns source retrieval, review state and receipts. Neutral wire types
belong in evidence-search-contracts only after promotion. Sourceright retains
canonical reference integrity; CiteWeft retains extraction. The downstream
library owns methods, results, typed tables, scientific scenes and reports.
Innovate owns forecasting; Voiage owns decisions and information value. No
mandatory circular Cargo dependency or duplicated provider runtime is introduced.

## Proposed wire profile

[Schema](../../contracts/proposals/scholarly-stream.v1.schema.json) and
[synthetic fixture](../../contracts/proposals/fixtures/scholarly-stream-upsert.json)
are deliberately outside the frozen published schema catalogue. The exact bytes
also ship in bibx design.2. [Candidate descriptor](../../integration/candidates/bibx-scholarly-stream.json)
records their identity; it is not an active integration passport.

The envelope carries contract/version, stream/epoch/sequence/event identity,
observed timestamp, source/prior revision, rights and change digest. Changes are
upsert, correction, retraction or rights deletion. Publications retain CSL JSON;
non-publication objects use typed separately versioned artifacts rather than
pretending grants or patents are journal articles. References, affiliations,
resolution decisions, taxonomies and coverage are provenance-bearing extensions.
Missing values remain unavailable rather than zero or invented enrichment.

Counters are decimal strings for lossless u64/browser interchange. Semantic
validation must reject overflow as well as shape errors. SHA-256 payload digest
covers RFC 8785 canonical `change` only, excluding the enclosing digest itself.
The synthetic ASCII/integer fixture exercises a compatible subset; it is not a
complete JCS conformance implementation. Pin the official CSL profile before
production admission; this proposal only requires its minimal container shape.

## Dynamic versioning and invalidation

Resolve version ranges/latest aliases to exact snapshots before a run. Keep
wire versions, source snapshots, scientific method versions, execution IDs,
result table revisions and render revisions distinct. Corrected source records
invalidate affected calculations; timestamp-only provenance refresh can early-cut
when semantic inputs are identical. Formula changes create new method lineage.
Fonts/palette changes invalidate rendering, not scientific results. Preserve old
results except where rights deletion requires removing restricted data and
derived disclosure surfaces; retain only permitted tombstones/digests.

Unknown major versions require an explicit migration with loss reporting.
Idempotence is same event identity plus same payload; conflicting replay or a
sequence gap is not silently accepted. Ordering is per stream/epoch, not an
invented total order across sources. A pipe has no restart guarantee unless
spooled. Durable sink acknowledgements and checkpoint state share one boundary.

## Granular implementation tasks

- [ ] `crates/evidence-search-contracts/src/`: promote the shared envelope only
  after Rust/Serde and JSON-schema round trips match both consumers, numeric
  bounds/semantic invariants pass and published-surface evolution is approved.
- [ ] `crates/searchright-interchange/src/`: add bounded streaming writer API;
  do not assemble the whole export into a String. Keep current JSONL/CSL exports
  backward compatible. Stdout carries data; diagnostics go to stderr.
- [ ] `crates/searchright-living/`: bind events and semantic revisions to Track
  41 DAG/checkpoints, preserving source history, review authority and rights.
  Test duplicate same/different payload, out-of-order and missing sequences,
  cancellation, partial writes and restart with/without a spool.
- [ ] Existing neutral provider/leaf adapters: optionally supply allowed citation,
  affiliation and research-object enrichment. Dimensions requires declared API
  entitlement/rights; do not emulate proprietary FCR/RCR from an arbitrary local
  corpus. Record imported provider metric definitions and eligible populations.
- [ ] Consumer integration: bind exact producer and consumer revisions/artifact
  identities, install the same fixtures, run both gates and demonstrate an
  independent read. Only then promote from candidate to an active passport.
- [ ] Output provenance: connect result/table/scene/document artifact digests to
  source receipts and the evidence bundle. Searchright does not render a second
  scientometrics dashboard or recalculate the same indicators.
- [ ] Study handoff: freeze first-observed/as-of dates and held-out splits before
  passing features to Innovate; pass explicit utilities/joint uncertainty/study
  designs to Voiage, using the actual supported method and binding versions.

## Imaging boundary

The downstream design requires mark-to-row/source traceability, separate
coverage/identity/statistical/layout uncertainty, stable temporal coordinates,
LOD membership/count conservation, multiverse sensitivity, accessible equivalent
tables and offline safe rendering. Scientific geometry must not be invented by a
generative model. These are research targets with comparator tests, not claimed
frontier achievements. Rights apply to labels, hover text, hidden layers/tiles,
SVG metadata, workbook cells and embedded sources.

## Verification and lean delivery

Run `python -m unittest discover -s tests -p test_scholarly_stream_proposal.py -v`
with the repository's jsonschema test dependency. Structural fixtures are not
runtime or scientific evidence. Before promotion add producer/consumer tests,
compiler-backed serialization parity and incremental/full-replay equivalence.
This design change does not replace required hosted checks. Routine authorised
edits/tests/PRs need no new approvals; keep explicit authority for release,
restricted disclosure, destructive changes or new external privileges.

Source baseline inspected: `fbfdde8814ce121fd4d79a3ff1285fa91e9e12a1`.
Dimensions sources: https://docs.dimensions.ai/dsl/data-sources.html and its
FCR/RCR help definitions (articles 9780801 and 9780865). Comparator inventory and
full rendering contracts are downstream, not copied into Searchright.
