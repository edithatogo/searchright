# Egress policy

Network access is disabled unless all conditions hold:

1. the review plan names the information source;
2. a provider policy manifest permits the operation;
3. required identity/credentials are present without being logged;
4. the query and page budget are within configured limits;
5. live execution is explicitly enabled;
6. the provider's current terms and licence permit the intended use.

Redirects across hosts, non-HTTPS endpoints, arbitrary user-supplied URLs and
unbounded pagination are denied by default. All live calls produce a redacted
receipt with provider, platform, query hash, timestamp, page/count metadata and
runtime policy.
