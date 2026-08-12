# Data lifecycle policy

Searchright treats retention, export and deletion as explicit policy decisions,
not filesystem convenience operations. `DataLifecycleRequest` separates a
non-authorising `preview` from `apply`. An apply decision fails closed unless it
has an accountable approval scoped to the exact review and lifecycle action.

The policy evaluator does not itself mutate storage. A caller may act only when
`effects_authorized` is true, must persist the decision and a content-addressed
effect receipt, and must keep the original request and approval reference in the
append-only audit stream.

Deletion never includes the audit ledger. Mutable content is replaced by a
tombstone for each stable target identifier so later audit events and receipts
cannot silently point to an object that appears never to have existed. Preview
decisions list the same tombstones but never authorise their creation. A legal
or preservation hold denies deletion even when the request carries an otherwise
valid approval.

Exports require an explicit destination and always produce a destination-review
warning. This source policy does not establish that a destination is legally or
institutionally approved, that deletion is complete on every backup, or that an
external repository accepted an export. Deployment review and compiled
store-level effect tests remain separate evidence gates.

## Event evolution

`contracts/events/registry.json` lists the event types understood by the derived
state reducer and their payload versions. Unknown event types and unknown
payload versions are rejected. A declared migration transforms a deep copy for
reduction; it cannot rewrite the original event or its hash chain. Migration
fixtures demonstrate deterministic projection only, not an in-place persisted
store migration. The reducer rejects oversized, unregistered or prohibited-key
payloads, but the store must also invoke the same policy before persistence;
that ingestion-boundary wiring remains a storage implementation gate.

## RO-Crate and OSF handoff planning

`research-object-handoff-plan.v1` is deliberately plan-only. It binds proposed
inputs by digest, identifies RO-Crate and OSF as external-write destinations,
and fixes execution to `dry_run`. Deposit authority, RO-Crate conformance and
OSF acceptance are all false. Track 25 owns actual deterministic research-object
export and conformance; a later external deposit additionally requires explicit
current-session authority, destination review and a verified remote receipt.
