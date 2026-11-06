# Track 16: rustls-native-certs 0.8.4 — advisory deferral

Registry archive SHA-256:
`dab5152771c58876a2146916e53e35057e1a4dfa2b9df0f0305b07f611fdea4d`.
The coordinator matched this with the lockfile at base revision
`4ddfbdbc54c500719889e09942783bdc04ee832b` and compared every extracted
package file with its archive member. All 13 file digests are recorded.

## Review outcome

All five separate read-only agent passes recommend **defer**. Their unedited
reports are preserved as role-named Markdown files. These are isolated agent
reviews, not independent institutional reviewers or demonstrated model diversity.
No cargo-vet audit is certified, exemption changed, deadline extended, publisher
trusted, upstream report submitted, or maturity gate promoted. The recorded
backlog remains 247; this packet is not a fresh full-backlog computation. Existing
approved exemptions retain the 2026-11-10 deadline. Owner disposition for this
new package remains unrecorded; the earlier openssl-probe deferral is separate.

## Confirmed local observations

Supplemental safe-API harness; Rust 1.97.1, aarch64-apple-darwin, offline build.
Each probe ran in a fresh child process with overrides supplied at creation;
the parent environment and native trust stores were not modified.

- Baseline returned 158 certificates and no errors.
- Empty and delimiter-only SSL_CERT_DIR values returned the same counts. Source
  routing confirms native fallback after empty components are discarded. This
  differs from the documented promise that setting either variable disables
  native loading. Count equality alone does not prove certificate-set identity.
- Two explicitly selected empty directories returned zero certificates and no
  errors, demonstrating directory-list selection without native fallback here.
- A missing explicitly selected file returned zero certificates and one error.
- A noncertificate file returned zero certificates and no errors. Error-free
  results are not proof of a usable root store; this behavior is documented.
- Supplying a directory as SSL_CERT_FILE returned zero certificates and one error.

Static inspection confirms the two include_str! PEM fixtures required by the
packaged unit tests are absent from the exact archive. Upstream tests were not
run or repaired; this is not an observed compiler failure or a runtime-library
build failure. The supplemental library/harness build did succeed.

## Qualification of reviewer findings

- Windows has unchecked native query results. Coordinator inspection of locked
  schannel 0.1.29 shows `valid_uses()` has real Err branches, but
  `is_time_valid()` always wraps its result in Ok. Thus its unwrap is not an
  established error-triggered panic for this resolution. No Windows runtime
  trigger was demonstrated. The original reports remain unedited.
- macOS may retain lower-priority trust after errors reading higher-priority
  settings when callers consume partial results. This is a static conditional
  risk; no OS error injection or trust-policy violation was demonstrated.
- Searchright's all-target dependency path uses rustls-platform-verifier 0.7.0.
  That crate calls rustls-native-certs in its non-Apple, non-Android Unix branch
  (`src/verification/others.rs:85-116`), warns on loading errors, uses parsable
  roots, and rejects an empty root store unless extra roots were supplied.
  These Windows/macOS package findings therefore are not evidence of those
  branches executing through Searchright's platform-verifier path.
- A certificate snapshot is not full native verification-policy equivalence.
  This packet does not demonstrate TLS acceptance, exploitation, all-target
  runtime correctness or certification of transitive dependencies.

## Reproduction and evidence

`request.json`, `source-manifest.json`, `fixture-check.json`, five raw reports,
`probe_harness.rs`, `harness-Cargo.toml`, `harness-Cargo.lock`,
`harness-build.log`, `probe-observations.json`, `consumer-tree.txt`,
`consumer-context.json`, and `disposition.json` are bound by
`packet-manifest.json`. The original registry source is not redistributed.

To rebuild, retrieve and verify the exact archive, extract it unchanged, place
`probe_harness.rs` at a supplemental project's `src/main.rs`, and adjust only
that project's path dependency to the extracted package. Its captured manifest
used the review scratch-directory layout. Use the captured lock and pinned
compiler with `cargo build --locked --offline`. Run the resulting executable in
fresh child environments using the cases in `probe-observations.json`; substitute
fresh temporary fixture paths and create directories/noncertificate input as
specified there. Do not mutate process-global environment in a shared runner.

## Recommended next disposition

Keep this review deferred without changing exemption history or deadlines.
Obtain native failure-path evidence and provenance-bound omitted fixtures before
any acceptance decision. Review rustls-platform-verifier separately to qualify
actual consumer behavior. Findings and probes are local handoff artifacts, not
merged changes or full-repository verification. See the accompanying receipt for
explicitly unrun gates.

## Integration formatting note

`raw-first-passes.json` preserves the exact original report strings, including
whitespace. Role-named Markdown copies have trailing whitespace removed and a
final newline added to satisfy repository text hygiene; findings are unchanged.
