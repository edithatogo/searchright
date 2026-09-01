# CHHHS institutional publication-intelligence pilot

## Track ownership

- **Conductor owner:** Track 36, release-candidate pilots.
- **Pilot identifier:** `track-36-chhhs-research-intelligence-demo`.
- **Scope:** one fixture-first institutional publication-monitoring vertical slice for Cairns and Hinterland Hospital and Health Service (CHHHS).
- **Dependencies exercised, not owned:** shared provider execution and receipts, open connectors, conservative record linkage, living updates and report rendering.

This pilot does not create a new product-level track. It evaluates whether existing SearchRight surfaces can support an institutional monitoring use case and records the product gaps revealed by that exercise.

## Demonstrated vertical slice

The pilot:

1. delegates live provider execution to a configured SearchRight adapter rather than implementing provider pagination, retry, receipt or network policy independently;
2. uses rights-clear synthetic records representing PubMed, Europe PMC, Crossref and OpenAlex;
3. links exact DOI overlaps while retaining every source and source-record identifier;
4. separates confirmed, probable, review-required and insufficient institutional evidence;
5. rejects a Cairns-only geographic mention as CHHHS attribution;
6. applies a versioned, explainable multi-label taxonomy and preserves matched terms;
7. stores incremental candidate state locally; and
8. renders deterministic HTML, JSON and CSV monthly reports.

## Acceptance criteria

- [ ] The application runs using only the Python standard library in fixture mode.
- [ ] Unit tests cover institutional aliases, facility aliases, non-affiliation mentions, geographic false positives, DOI overlap, source preservation, classification provenance and deterministic reports.
- [ ] The recurring workflow is read-only, uses commit-pinned actions, disables persisted checkout credentials and keeps live execution opt-in.
- [ ] The repository static harness, workspace tests, all-feature Clippy and source-integrity manifest pass at the exact PR head.
- [ ] The pull request declares Track 36 and changes no other Conductor track.
- [ ] Hosted checks complete successfully before the PR leaves draft.

## Evidence ceiling

Successful fixture tests show only deterministic local behavior against project-authored synthetic metadata. They do not prove:

- exhaustive discovery of all CHHHS publications;
- current live-provider compatibility or API coverage;
- sensitivity or positive predictive value of institutional attribution;
- correctness of the CHHHS alias and facility register;
- production tenancy, retention, backup, restoration or correction governance;
- methodological or information-specialist validation; or
- an authoritative researcher, service or publication-performance register.

## Improvement needs revealed

The pilot identifies follow-on work for separately scoped tracks or amendments:

1. a neutral institution-query contract with durable organisation identifiers, historical validity and facility relationships;
2. structured author-affiliation evidence and organisation identifiers in canonical records;
3. provider-specific institutional filters and enrichment, including PubMed affiliation retrieval and OpenAlex abstract reconstruction;
4. request-level admission and receipt coverage for multi-request provider operations;
5. durable recurring-monitor state, replay and recovery adapters;
6. a labelled institutional-attribution calibration corpus and measured review burden;
7. versioned taxonomy/model contracts with uncertainty, override provenance and drift monitoring; and
8. an institutional monthly-report contract distinct from systematic-review reporting.

## Operational decision gates

Before any CHHHS operational deployment, an accountable owner must approve the institution identity register, attribution policy, correction process, retention and access model, publication definitions, report audience and durable storage. Any comparative staff or service dashboard requires separate governance because public metadata can still create consequential personnel inferences.
