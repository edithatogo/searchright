# Planned rcagent permitted-evidence consumer

**8 November 2026 — planning only. Primary Conductor owner: Track 13.**

This note incorporates the generic Searchright handoff from the
[cognitive-autopsy programme](https://github.com/edithatogo/rcagent/pull/117).
It maps future work into the existing
[Track 13 plan](../../conductor/tracks/13-integration-passports-github-context/plan.md)
Phase 3 consumer-driven contract task and
[estate ownership boundaries](../estate-integration.md). It creates no
runtime schema, passport, dependency pin, data access or evidence promotion.

The inspected Searchright baseline is
`4bfffc43e25641962fca394d43446888376156b4`; the consumer design was inspected at
`edithatogo/rcagent@0c8607661e64a34b351b87ca1c6148d0e291fbed`. These are inspection
references, not a claim of tested runtime compatibility or automatic pin updates.

## Ownership and current capability

- `evidence-search-contracts` owns neutral query/provider/record/receipt/audit
  types; `evidence-search-core` owns the shared execution/replay/cache machinery.
  Reuse these layers before introducing a consumer-specific primitive.
- Searchright owns public literature-search and review orchestration. It does
  not own the HDC source corpus, clinical assertions, temporal entitlement,
  patient/family perspective, materiality scoring or RCA conclusions.
- rcagent owns its permitted evidence view, local case retrieval, semantic
  verification, report synthesis and study evaluation. Its existing SQLite
  FTS5/BM25 path remains the baseline and does not depend on this integration.
- Sourceright owns citation identity/canonicalisation and provenance linkage;
  its linkage result is not a semantic-support verdict. Preserve this distinction
  in any receipt handoff.

## Separate two operations

| Operation | Permitted use | Required boundary |
| --- | --- | --- |
| Public literature discovery | A declared, admissible public research question using the existing provider runtime and receipts. | The local consumer decides which query may leave the environment; no case-derived text is forwarded implicitly. Public-search permission does not authorise case access. |
| Case-evidence retrieval | An optional local adapter over a consumer-provided source view, if a concrete need survives comparison with rcagent's existing retrieval. | No external provider, telemetry or error payload receives case text, identifiers, embeddings, prompts or results. Unsupported local operation returns unavailable; never fall back to cloud search. |

Classification and authorisation belong to trusted local policy, not to an LLM
redaction promise or the contents of a retrieved document. Do not change the
default-deny policies or treat an opaque identifier alone as access authority.

## Neutral binding requirements

These are proposed acceptance requirements, not currently supported wire fields.

1. Bind each request and receipt to an immutable consumer view ID/digest and
   policy revision, exact source revisions/hashes and permitted span ranges.
   Carry any context/cutoff/recipient-scope reference as consumer-defined data;
   the core must not infer clinical awareness or broaden membership.
2. Preserve source identity, provider/local-store origin, record/version,
   extracted coordinates and uncertainty in returned evidence. A valid source ID
   does not admit a prohibited span from that source.
3. Include the view identity in replay/cache keys and derived-result bindings.
   Do not reuse a broader or later view for a restricted request. A changed
   source, policy, claim or view requires invalidation or a new scoped receipt.
4. Keep provider acquisition/publication timestamps distinct from a consumer's
   event, documentation and information-availability times. Historical policy
   applicability and jurisdiction are consumer decisions, not consequences of
   search rank or a recent capture date.
5. Permit declared queries for potentially contrary evidence within the same
   view. Return candidates and retrieval diagnostics, not a finding that the
   claim is contradicted or that all contrary evidence has been found.
6. Return incomplete, denied and unavailable states explicitly. Empty results
   do not establish absence of a fact, event or relevant evidence. Preserve
   candidate/version disagreement rather than silently selecting a clinical
   interpretation.

Support classification and report rendering downstream must continue to use the
same view; public/background references cannot silently enter the decision-time
packet. Outbound status must be allowlisted and non-sensitive. Logs, errors,
receipt exports and `.srpack` packaging are part of the local data boundary.

## Bounded implementation sequence

| Step under Track 13 | Consumer mapping | Acceptance before progression |
| --- | --- | --- |
| Compare existing contracts with the proposed view envelope; choose the smallest additive adapter or document no new core requirement. | rcagent CAW08 exact-source checks and CAW10 temporal entitlement; contradictory retrieval is a separate consumer requirement. | Exact field ownership, source-span convention, failure states and cache/receipt identity agreed; no duplicated provider runtime. |
| Prepare synthetic positive/negative consumer fixtures and a passport proposal only if a new capability is needed. | rcagent CAW08/09/21; source/semantic support and combined verifier evaluation. | Deny out-of-view spans, stale views, missing sources, unavailable local capability and source-instruction attacks. Separate reference-evidence retrieval tests from end-to-end quality tests. |
| Execute producer/consumer parity and no-egress tests at exact revisions; record errors and resource behaviour. | Existing consumer-contract suite and RCA evaluation harness. | Neither fixture presence nor a passing schema is compatibility, privacy or clinical validation. Real sensitive data are not needed for boundary fixtures. |
| Admit the optional adapter through the existing passport/lock/release-train process. | Existing Track 13 drift and consumer gates. | Dual receipts, version/rights/compatibility review and tested rollback; otherwise retain the local baseline and public-search handoff. |

No new project-wide completion counters or task hierarchy are introduced. The
existing 200-assertion ledger is not extended or promoted by this design note.
Clinical/local model experiments, behavioural perturbations, dataset labels and
patient-centred outcomes remain in rcagent and learningfromharm. Source release
and Hugging Face distribution remain with the existing corpus owner.

## Required future adversarial fixtures

- A source is allowed but the cited paragraph is outside the view.
- A decision-time view and later-outcome view share source IDs but differ in
  permitted spans, scope or policy revision; cached results must remain isolated.
- A local lookup fails while public providers are available; no provider fallback
  or sensitive error/telemetry export occurs.
- A source contains instructions to search externally, retrieve a hidden record
  or relabel an assertion; it remains untrusted evidence.
- A contrary-evidence request has no hits; report the retrieval limitation
  without asserting that no contradiction exists.
- An otherwise valid citation is attached to an unsupported claim; Searchright
  preserves provenance without issuing a semantic verdict.
- A derived report is edited or the view changes; old receipt bindings cannot
  authorise the changed output automatically.

The acceptance evidence is a synthetic consumer test suite, local network/log
boundary tests and exact producer/consumer receipts. No such execution or live
adapter is claimed by this document. Failure leaves the existing local workflow
available and does not require a new provider, general vector store or clinical
reasoning subsystem in Searchright.
