# EFetch orchestration: offline implementation, delivery pending

The default PubMed ESummary identity and four-provider selection stay unchanged.
A separately registered `pubmed-efetch` identity prevents accidental summary-cache
reuse. No transport execution or live-provider support is established by this plan.

## Required neutral-core prerequisite

The existing runtime applies a rate reservation once per provider page, but both
ESearch/ESummary and prospective ESearch/EFetch pages contain two HTTP requests.
Immediate second requests would bypass that interval. Do not duplicate a rate,
retry, cache or timeout controller inside the connector.

The separately authored additive `SearchProvider::execute_page_with_context`
default admits the legacy page exactly once. Both PubMed adapters override it
and submit every HTTP operation factory to `PageExecutionContext::run_subrequest`.
There is no separate initial page admission or duplicate connector retry engine.
Both adapters now reject direct `execute_page`, even with `live_enabled=true`;
callers must use the registry. This is a deliberate behavior tightening.

This draft is ported onto a test-only union of the separate Track03 context and
Track04 parser-version changes. That union history is not a delivery unit and
must never be pushed as this PR; only this connector-owned delta may be rebased
onto the actual merged dependency revisions. Do not copy core source into this delta.
The context cannot grant live
authority, extend deadlines or lower caller/manifest intervals. Same-provider
concurrent operations must share reservations; separate PubMed and EFetch IDs
join the fixed trusted `ncbi-eutils` registry group through
`register_mvp_live_providers` and `register_pubmed_efetch_provider`. Manual
registration must explicitly select the same group to obtain cross-adapter
spacing; ordinary `register` only provides per-provider spacing. Per-provider gates
alone do not prove aggregate compliance across those IDs or across processes.

## Offline injected-byte acceptance matrix

- Exact ESearch then EFetch endpoints; no ESummary request or fallback.
- Numeric bounded unique ID batch, strict count and continuation semantics.
- Two independently computed raw digests attached to the page; no claim that
  the final source receipt retains the raw-response chain.
- Strict UTF-8, per-response byte bounds, DTD/entity rejection, complete XML,
  exact requested/returned identity reconciliation, unsupported-record denial.
- Rate admissions observed before each mock-byte invocation; exhaustion or
  cancellation before the second admission prevents its invocation.
- Errors remain static/redacted; configured query, contact, cursor and malformed
  body content never appear in diagnostics.
- Summary-cache entries do not satisfy the EFetch provider, and EFetch replay
  remains explicitly classified by the existing runtime.

The XML response cap is the smaller of request policy and 8 MiB, applied during
production body accumulation and rechecked at the injected byte boundary. DNS,
public-address pinning, proxy/redirect denial and HTTP status handling remain
in the shared private byte reader. JSON transport retains existing policy limits;
decode errors are redacted. Numeric Retry-After seconds retain existing handling;
HTTP-date Retry-After is not newly implemented here.

Both PubMed adapter behavior versions derive from `PROVIDER_PARSER_VERSION` with
`.subrequests.1`, isolating prior summary caches as well as the distinct EFetch
provider ID. This preserves old bytes without migration or old-version fallback.
Two raw body hashes live on ProviderPage diagnostics only, not a new authenticated
final receipt chain. EFetch is citations/abstracts, not full-text retrieval.

Tests were written before implementation; no preimplementation baseline execution
was possible while the parent owned Cargo. Subsequent package tests execute the
orchestration through a scripted byte boundary and actual registry, not HTTP.
Synchronous factory timestamps prove local admission spacing. Cache tests prove
summary/EFetch and immediate previous-summary-version partitioning, retained old
entries, and current EFetch replay with zero new factories. They do not establish
real transport, provider policy or live-response evidence. Full repository and
delivery validation remain separate; never deliver the test-only union history.
