# Parity execution matrix v2: bounded evidence validation

Track 03 adds a separate expected catalogue and observed matrix to the
Searchright contracts layer, plus an inward-facing validator in
`searchright-sourceright-compat`. The v1 schema, example, Rust API and advisory
summary remain unchanged. There is no v1-to-v2 migration: a summary cannot be
expanded into invented execution observations. No downstream source or persisted
records are rewritten, and there is no cutover consumer in this change.

## Trust and scope

`validate_execution_matrix(expected, expected_pin, matrix)` requires the caller
to select a trusted expected catalogue and its canonical BLAKE3 pin independently
of the report. The pin is a separate argument, never taken from the report.
Deriving both inputs from the report would defeat this boundary. The validator
cannot decide whether the caller chose sufficient provider/fixture coverage.
Expected cells are explicit tuples, not a Cartesian product invented by the
validator. Only existing case-to-dimension assignments are admitted, and a
regression compares all 20 assignments with the v1 migration catalogue.

Every cell identifies a provider, fixture SHA-256, case and dimension, declared
fixture provenance and the supported comparator. Each side binds an explicit
legacy/shared role, repository identifier, full 40-digit lower-case Git commit,
configuration SHA-256 and harness SHA-256. The matrix must match these bindings
exactly. The validator rejects missing, duplicate, unexpected and reassigned
cells; it cannot accept a report simply because its own shorter inventory agrees
with itself. The catalogue pin is cell-order independent.

Fixture/configuration/harness/evidence SHA-256 fields are declared artifact pins.
No artifact is read, retrieved, authenticated or rehashed by this validator.
`source_id`, `rights_basis`, execution IDs and decision references are declared
references, not source-rights clearance, verified execution or authenticated owner
decisions. Unknown fields in typed envelopes are rejected. Observed JSON values
are already parsed: this API does not reject duplicate keys lost by an earlier
JSON decoder or preserve original number spelling, whitespace or escapes.

## Comparator and digests

The sole comparator is `canonical-json-blake3.v1`. Canonical JSON uses recursively
sorted object keys and compact `serde_json` serialization, preserves array order,
and does not trim, case-fold, coerce numbers, or infer semantic equivalence.
Parsed 1 versus 1.0 and positive versus negative floating zero remain different
serialized observations. This is not an RFC 8785/JCS implementation or proof of
cross-language number fidelity. Consumers must preserve this comparator identity
and use the same admitted representation.

Observation digests are BLAKE3 of the UTF-8 domain
`searchright.parity-observation.blake3.v2`, one zero byte, then canonical JSON of
the ordered tuple `(expected cell, run binding, status, value, execution_id,
evidence_sha256)`. The claimed digest itself is excluded. Side and cell binding
prevent transplantation even when identical values or identical repository pins
appear in multiple comparisons. Catalogue pins use domain
`searchright.parity-catalogue.blake3.v2`, zero byte and canonical JSON of the
catalogue sorted by cell tuple. These are integrity checks, not signatures.

Comparison equality is separately recomputed over `(status, value)` using domain
`searchright.parity-comparison.blake3.v2`. It intentionally ignores distinct run
references and artifact hashes, which remain separately integrity-bound. The
validator returns equality plus both outcome classifications and whether both
sides declare a provider execution. Equal provider errors can be valid negative
test observations; equal harness failures or skipped operations remain visibly
not executed. Different values remain different even with decision references.
The original observations remain in the input report. No return field authorizes
cutover, release, deletion, live support, or approval of a difference.

## Resource and privacy boundaries

Inputs are preallocated Rust structures, not an untrusted-byte parser. Callers
must bound raw input bytes and deserialization before this API. Validation limits
each catalogue/report to 4096 cells, each observation value to 64 KiB serialized
JSON and depth 32, and aggregate observation JSON to 8 MiB. Depth/breadth/string
preflight precedes recursive canonicalization; metadata fields are bounded to
512 UTF-8 bytes, and each cell has at most 16 unique decision references. These
limits bound additional validation work, not memory already allocated by callers.
Schemas describe structural constraints; byte, aggregate, digest and external
scope checks require the Rust validator.

Use minimised observations. Errors contain static categories rather than echoing
payloads, references or repository identifiers. This is not arbitrary-content
redaction, and the validator performs no network, filesystem or external writes.

## Evidence and next step

The two contract examples are synthetic illustrations with fabricated artifact
pins, not old/new execution receipts or a complete provider inventory. Native
and full-repository validation are separately recorded by the coordinating task.
The next evidence step is an independently selected, rights-qualified fixture
inventory and actual exact-revision old/new execution producer. Companion pure
parser APIs may support a bounded offline comparison; runtime retry/cache/host
cases need suitable test seams rather than inferred observations. Feature-gated
downstream cutover, owner decisions, SemVer, rollback and hosted gates stay open.
