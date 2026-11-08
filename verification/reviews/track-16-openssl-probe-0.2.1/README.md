# Track 16: openssl-probe 0.2.1 — deferred audit packet

This is an advisory packet, not an accepted cargo-vet audit. The exemption and
2026-11-10 deadline are unchanged. No upstream issue or external report was sent.

## Evidence

- `request.json`: common request supplied before isolated first passes.
- Five role Markdown files: unedited final first-pass responses from separate
  ephemeral, read-only Codex CLI processes; no peer outputs supplied.
- `source-manifest.json`: exact package file digests and verified archive digest.
- `probe_harness.rs`: coordinator-authored, safe-API-only reproduction harness.
- `probe-observations.json`: observed child-process results on macOS arm64.
- `consumer-tree.txt`: all-target, all-feature dependency path.
- `disposition.json`: synthesis, dissent, consumer-path qualification and pending
  accountable-owner decision.

The coordinator verified that the cached registry archive's SHA-256 equals
`7c87def4c32ab89d880effc9e097653c8da5d6ef28e6b539d313baaacfbafcbe`
from the locked identity and that all extracted files match their archive
members byte-for-byte. Digest agreement is not publisher trust.

## Findings and limits

Separate child processes received `SSL_CERT_FILE` and `SSL_CERT_DIR` through their
creation environment; the parent environment was not mutated. The safe probe
APIs demonstrated:

1. A colon-separated list of two existing directories is discarded.
2. A single existing private directory is supplemented by `/etc/ssl/certs` on
   the observed host.
3. A directory is accepted as the `SSL_CERT_FILE` path, with positive environment
   recognition. This is not certificate validation.

No unsafe initializer or TLS handshake was tested. There is no demonstrated
remote exploit or claim that all Searchright trust-loading paths are affected.
In particular, rustls-native-certs 0.8.4 parses explicit environment overrides
before selecting its platform fallback, which limits direct applicability of
these package-level findings to the normal consumer path.

## Reproduction recipe

Use Rust 1.97.1. Obtain the exact checksum-verified package in a disposable local
directory; do not alter the Cargo registry cache. Compile its dependency-free
library with `rustc --edition=2021 --crate-type rlib --crate-name openssl_probe
<package>/src/lib.rs --out-dir <temporary-build>`. Compile `probe_harness.rs`
with `--extern openssl_probe=<temporary-build>/libopenssl_probe.rlib`.

Run that harness in separate child processes with environment values supplied
at process creation. Create two empty temporary directories A and B. Compare
`SSL_CERT_DIR=A:B`, `SSL_CERT_DIR=A`, and `SSL_CERT_FILE=A`, with the other variable
empty. Output prefixes distinguish returned directories, certificate file,
environment recognition and candidate directories. Host system candidates vary;
the broadening observation requires an existing candidate directory.

## Recommended disposition

Defer certification and preserve the exemption. Security/safety recommended
approval with limitations; the other four passes deferred. Both positions are
preserved. The owner has directed deferral with the exemption unchanged and requested an
unpublished upstream-report draft (`upstream-report-draft.md`). No certification
or publication is authorised by that disposition. This packet remains a local
handoff artifact, not a merged source change.

## Integration formatting note

`raw-first-passes.json` preserves the exact original report strings, including
whitespace. Role-named Markdown copies have trailing whitespace removed and a
final newline added to satisfy repository text hygiene; findings are unchanged.
