# Redaction and data minimisation

Search queries can disclose unpublished questions, rare-disease phenotypes,
institutional priorities, contact addresses and credentials. Searchright
therefore treats query text and receipt diagnostics as potentially confidential
even when returned bibliographic metadata is public.

`policy/redaction-profile.json` is the default receipt-minimisation profile.
`scripts/redaction.py` applies it deterministically to URLs, nested objects and
free-text diagnostics. The policy:

- redacts credentials, authorisation material, cookies, contact addresses and
  query-bearing parameters;
- preserves only explicitly safe transport controls;
- removes URL fragments;
- detects common secret assignments, bearer tokens and probable high-entropy
  values;
- preserves field names so audit structure remains intelligible;
- never enables raw-response retention.

`scripts/check_redaction_policy.py --self-test` runs adversarial cases and
verifies determinism. The live Rust connector also emits generic pre-response
and body-read errors rather than serialising potentially query-bearing request
errors.

Pattern redaction is a minimisation control, not proof of de-identification.
Full text, arbitrary provider payloads and user-supplied free text still require
classification, retention and human-review policy.
