# Serde boundary qualification — 18 November 2026

**Disposition: retain existing dependencies; scoped assurance only.** This is a review artifact within the existing quality/security track, **not** a new numbered track, phase, status or issue hierarchy. No past acceptance is reopened.

## Verified runtime boundaries
- Root `Cargo.toml` declares `serde` derive, `serde_json` with `preserve_order`, `csv`, and `quick-xml`; schema-and-contract-dependent crates use these at the actual interchange boundaries.
- `crates/searchright-interchange/src/xml.rs` uses `quick_xml::Reader` for error-aware tokenisation, byte offsets, record segmentation and recovery from damaged XML. Serde cannot substitute those capabilities. `csv` adds CSV parsing semantics and can itself use Serde.
- `serde_json` is the necessary JSON format adapter; replacing it with `serde` alone is not technically meaningful.

## Decisions
1. Keep typed Serde DTOs and versioned interchange envelopes as the default for stable APIs, with explicit schema validation and source/provenance requirements.
2. Keep JSON values dynamic where provider extensions or loss-preserving unknown fields are part of the public contract. Do not impose global `deny_unknown_fields` on forward-compatible provider payloads.
3. Preserve XML parser recovery and source offsets. Any alternative must demonstrate byte-offset, malformed-record recovery, XXE/entity limits, escaping and corpus parity, not merely a successful deserialize.
4. JCS/canonical hashing, rights and evidence-level assertions remain format/contract obligations, not Serde guarantees. Distinguish receipt byte identity from semantic equivalence.

## Pending bounded checks (no new implementation presumed)
- [ ] Inventory existing DTO and dynamic-Value call sites against the actual live stable-schema contracts, using the existing assurance and interchange test suites.
- [ ] Freeze malformed/duplicate-key, integer/float, Unicode, preserved ordering, XML fragment and CSV quoting fixtures; decide adapt/retain/no-go at each *actual* redundant codec.
- [ ] Verify exact-head `cargo test --locked`, fuzz, and applicable release/schema/consumer gates before claiming improvement.
- [ ] Route any resulting code task to existing ownership/track 16/26 and Conductor's canonical issue hierarchy; no automatic backfill or duplicate track. Current review is planning only.
