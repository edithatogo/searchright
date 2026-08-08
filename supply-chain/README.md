# Cargo Vet policy

This directory is the canonical `cargo-vet` store. It deliberately contains no
local exemptions or audits before the dependency graph has been resolved and
reviewed. CI runs `cargo vet --locked`; imported audits can reduce duplicated
review, but do not silently become Searchright-authored evidence.

A new exemption requires an owner, exact crate version, criteria, rationale,
expiry/review date and a linked issue. Wildcard trust is prohibited unless a
separate supply-chain policy explicitly approves the publisher and expiry.
