# Provider model

Supplementary sources and manual methods are governed separately from live
bibliographic providers. See [Supplementary discovery methods](supplementary-discovery.md)
for the source catalogue, exact-method logging, coverage-risk matrix, and
fixture-only OpenCitations boundary.

A provider adapter translates a canonical strategy, executes an authorised query
and returns records plus an evidence receipt. It does not decide eligibility or
silently mutate canonical records.

## Required adapter capabilities

- stable provider identifier and source/platform distinction;
- dialect compiler or explicit import-only status;
- configuration schema with secret fields marked;
- rate-limit, retry, timeout, pagination and maximum-record policy;
- fixture replay independent of the live service;
- response-size and content-type validation;
- terms/licence note and allowed storage fields;
- retrieval timestamp, query hash, endpoint, request metadata and result counts;
- opt-in live smoke with redacted receipt.

## Support levels

| Level | Meaning |
| --- | --- |
| Planned | Contract/track only. |
| Fixture-backed | Parser and pagination proven with checked-in fixtures. |
| Opt-in live | Redacted live smoke proves current API behaviour. |
| Maintained | Fixture, live, policy and compatibility checks are current. |

Licensed providers remain bring-your-own access and may use manual export/import
where APIs or licence terms do not permit automation.

## Source selection is not provider support

The [source-selection catalogue](source-selection-catalogue.md) is an **advisory,
versioned source/platform assessment index** for review planning. It links a
small first-party-confirmed sample to existing provider manifests and manual
methods without giving those entries executable authority. A database/platform
entry, a planned connector, a fixture-backed parser and an observed live run
are distinct claims. RefHunter is treated as a secondary reference only; it is
not a Searchright live provider or a right to bulk-redistribute descriptions.
