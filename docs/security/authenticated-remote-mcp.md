# Authenticated remote MCP and tenancy

Local stdio MCP remains the lowest-risk default. Remote Streamable HTTP is a
separate deployment profile requiring authenticated principals, tenant-scoped
authorisation, explicit capability scopes, data-region policy, bounded tasks,
rate limits, cancellation and audit correlation.

The v1 access policy is default-deny. Agents may recommend screening outcomes but
cannot make final eligibility decisions. External writes and final decisions
require the relevant scope and explicit human approval. Cross-tenant aggregation
is prohibited. Credentials and tokens never enter Searchright receipts.

OAuth/OIDC issuer configuration, key rotation, tenant provisioning, data
residency and incident response are deployment responsibilities and must be
threat-modelled and independently tested before a hosted-service claim.
