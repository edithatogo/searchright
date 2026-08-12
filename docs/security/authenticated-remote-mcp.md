# Authenticated remote MCP and tenancy

Local stdio MCP remains the default. The optional `remote-http` feature builds a
separate `searchright-mcp-http` binary for an authenticated, read-only
Streamable HTTP profile. It is an integration profile, not evidence of a hosted
or production-secure service.

The adapter is disabled unless `SEARCHRIGHT_REMOTE_MCP_ENABLED=1`, binds only to
loopback behind a separately governed TLS terminator, and requires:

- an exact public Host authority and explicit HTTPS Origin allowlist;
- one exact issuer, audience, RS256 JWKS and deployment region;
- a valid tenant-policy v1 document loaded by the server, never the client;
- bounded authentication concurrency, request duration, per-principal rate and
  tenant-wide in-process concurrency;
- a request identifier and in-process replay ledger; and
- a writable append-only redacted audit JSONL sink.

The trusted adapter constructs the existing access-request v1 value after JWT
verification. Clients cannot supply `authenticated`, tenant policy, counters or
approval facts. The frozen access-request and tenant-policy v1 schemas are not
changed by this transport.

All 31 currently exposed MCP tools are classified read-only and
non-destructive. Remote mode is limited to that surface and requires
`review_read`. Any future state-changing tool must introduce an exhaustive
tool-to-scope/effect mapping and cryptographically bound approval context before
remote exposure.

## Configuration

The binary requires these variables:

- `SEARCHRIGHT_REMOTE_MCP_BIND` (loopback socket)
- `SEARCHRIGHT_REMOTE_MCP_ALLOWED_HOST`
- `SEARCHRIGHT_REMOTE_MCP_ALLOWED_ORIGINS` (comma-separated HTTPS origins)
- `SEARCHRIGHT_REMOTE_MCP_AUDIENCE`
- `SEARCHRIGHT_REMOTE_MCP_JWKS` (rotating local JWKS file)
- `SEARCHRIGHT_REMOTE_MCP_TENANT_POLICY` (remote policy wrapper below)
- `SEARCHRIGHT_REMOTE_MCP_AUDIT_LOG`
- `SEARCHRIGHT_REMOTE_MCP_REQUEST_TIMEOUT_SECONDS` (1–300)
- `SEARCHRIGHT_REMOTE_MCP_AUTH_CONCURRENCY` (1–256)

The remote policy wrapper is deployment configuration, not a client contract:

```json
{
  "schema_version": "org.searchright.remote-mcp-policy.v1",
  "issuer": "https://issuer.example.edu",
  "maximum_token_age_seconds": 300,
  "maximum_requests_per_minute": 60,
  "deployment_region": "AU",
  "tenant_policy": {
    "schema_version": "org.searchright.tenant-policy.v1",
    "tenant_id": "tenant-demo",
    "allowed_regions": ["AU"],
    "allowed_scopes": ["review_read"],
    "maximum_concurrent_tasks": 4,
    "external_model_processing_allowed": false,
    "restricted_full_text_persistence_allowed": false,
    "cross_tenant_aggregation_allowed": false,
    "approved_by": "Governance officer",
    "policy_version": "1"
  }
}
```

Audit events contain only digests of the tenant, principal and request binding,
the policy version, timestamp and outcome. JWTs, bearer tokens, full request
bodies and raw principal/tenant identifiers are not written.

## Evidence boundary and remaining deployment gates

Deterministic tests exercise JWT verification and key removal, issuer/audience/
region/time denials, actual loopback Streamable HTTP initialization, replay,
rate, tenant concurrency, request timeout, audit redaction and stable errors.
The fixture private key is non-production test material derived from the Apache
2.0 `jsonwebtoken` test fixtures and must never be deployed.

This does not prove multi-replica replay/rate/concurrency, TLS-proxy identity,
live IdP revocation, cloud residency, external-write approval, production
observability, or independent security/privacy review. These remain higher
evidence gates; the repository must not claim hosted multi-tenant security.

## Rollback rehearsal

Before real traffic, an operator must rehearse: freeze edge admission; stop the
HTTP binary; verify its listener is unavailable; revoke the client grant/key;
drain or cancel bounded work while retaining the audit log; restore the last
approved policy version; and verify the stdio binary still initializes. The
fixture harness can prove only the feature/env default-deny and stdio fallback,
not a real deployment rollback.
