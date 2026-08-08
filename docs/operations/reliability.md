# Operational reliability

Searchright distinguishes liveness, readiness and evidence status. A process may
be alive while a provider, store or policy engine is degraded or not ready.
Health observations use stable diagnostic codes and never include query text,
full text, credentials or personal identifiers.

Remote deployments must define service-level objectives for availability,
bounded task completion, cancellation, audit persistence and restore recovery.
Long-running MCP tasks require cancellation propagation, idempotency keys,
request budgets and tenant-specific concurrency limits. Failure is reported as
failure or abstention rather than converted into an empty successful search.
