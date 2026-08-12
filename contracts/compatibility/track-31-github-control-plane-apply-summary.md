# GitHub control-plane apply summary compatibility note

Track 31 adds the alpha
`org.searchright.github-control-plane-apply-summary.v1` contract as a new,
additive receipt family. It does not change or supersede an existing persisted
contract.

The schema deliberately accepts only a successful, bounded apply summary: an
exact 40-character source revision, the protected `github-project-write`
environment, zero remaining or delete operations, a drift-free read-only audit,
zero audit mutations and SHA-256-bound hosted artifacts. Failed or partial runs
must retain their native receipts and cannot be represented as this proof
summary.

The contract has no Rust root type and carries no compiler or downstream
compatibility claim. Any relaxation of its fail-closed evidence invariants or
removal of required provenance fields requires a new schema version.
