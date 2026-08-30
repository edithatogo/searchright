# Cargo Vet policy

This directory is the canonical `cargo-vet` store. CI runs `cargo vet --locked`;
imported audits can reduce duplicated review, but do not silently become
Searchright-authored evidence. Peer imports are limited to the authoritative
registries named in `config.toml` and locked in `imports.lock`. Exact temporary
exemptions are present for the residual backlog and remain risk acceptances—not
audits or safety certification.

A new exemption requires an owner, exact crate version, criteria, rationale,
expiry/review date and a linked issue. Wildcard trust is prohibited unless a
separate supply-chain policy explicitly approves the publisher and expiry.

`exemption-proposals.json` is a non-effective governance ledger. An entry only
becomes effective after its status is `approved` and the exact exemption is
separately added to cargo-vet's `config.toml`. The governance checker fails if a
cargo-vet exemption lacks a matching, unexpired approval. An empty ledger does
not waive any dependency review.

Track 06's `CVX-0259` is a separate owner-approved exception for the exact
`quick-xml 0.41.0` checksum and resolved `default` feature (an empty default
feature), expiring on 2026-09-29. Its decision and unresolved audit findings are
in `verification/receipts/track-06-dependency-risk-approval.json`. The governance
gate checks the lockfile and offline Cargo metadata and rejects feature drift.
This approval does not cover optional deserialization or namespace-reader use.
The baseline ledger generator refuses to overwrite later owner decisions.
