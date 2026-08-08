# Provider governance

Searchright separates provider implementation from permission, policy review
and live-operability claims.

`integration/provider-policies/index.json` records conservative, versioned
local decisions for each MVP source:

- official endpoint and identified documentation/policy surfaces;
- access and credential class;
- receipt treatment for credentials and contact identity;
- query and response classification;
- raw-response retention default;
- redistribution caution;
- conservative minimum request intervals;
- live-canary opt-in;
- the date on which the source policy was inspected and a bounded review-due
  date;
- an unconditional manual-review requirement before a live-support release;
- policy-review status and evidence.

The default review status is `source_identified_not_legally_approved`. A source
may move to `reviewed_with_evidence` only when the manifest contains a current
review receipt. This avoids treating an API endpoint, open metadata statement or
passing fixture as blanket legal approval.

The runtime remains independently bounded by provider manifests, HTTPS host
allowlists, page/record/time/response-size budgets, retries and redacted
receipts. Policy manifests must match the provider response baselines before the
static harness passes.
