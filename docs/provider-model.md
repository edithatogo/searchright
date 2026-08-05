# Provider model

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
