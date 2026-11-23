# Track 16 dependency advisory reconciliation — 23 November 2026

**Issue:** [#831](https://github.com/edithatogo/searchright/issues/831). **Candidate PR:** [#846](https://github.com/edithatogo/searchright/pull/846). **Impact on source selection:** [PR #844](https://github.com/edithatogo/searchright/pull/844) must remain blocked until all mandatory security checks pass.

## RUSTSEC-2026-0592: url — patched lockfile candidate

- Advisories: https://rustsec.org/advisories/RUSTSEC-2026-0592.html and https://github.com/servo/rust-url/pull/1186
- Existing pinned Cargo.lock had url 2.5.8, vulnerable to unbounded nested-blob URL recursion in Url::origin(); fixed in **2.5.9**.
- A **temporary read-only GitHub Actions workflow** (PR #846 initial head `b15d38ff5cb5cabe20199705245cd4a130bf6da8`; job [137573158461](https://github.com/edithatogo/searchright/actions/runs/45139333705/job/137573158461)) ran:
  `cargo +1.97.1 update -p url --precise 2.5.9` and `cargo +1.97.1 metadata --locked --format-version 1 --no-deps`, then printed the exact Cargo.lock diff. This used read-only GitHub permissions, no credentials persisted by checkout and **made no remote changes itself**.
- Cargo's actual resolver also changed `form_urlencoded 1.2.2 -> 1.2.3`, including the verified registry checksums. The only resulting lockfile changes were those two packages. The reviewed candidate was applied as a normal GitHub commit; the fetched resulting blob SHA begins `3c9efad`, **exactly matching the `git diff` target blob from the successful Cargo job**.
- The temporary workflow was then deleted from the PR branch. A Python lockfile regression prevents inadvertent reintroduction of the known-vulnerable URL version.
- *Pending final admission:* required hosted Rust, supply-chain, lockfile and downstream checks on the final head; application paths relying on URL origin semantics should be checked against the standards-conformant new opaque-origin handling. The SHA/blob match proves precise lockfile identity, **not** execution safety by itself.

## RUSTSEC-2026-0543: chrono — upstream dependency hold

- Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0543.html (informational/unmaintained; no patched version).
- Current Searchright `rmcp =3.9.0` depends on chrono non-optionally; `rmcp` also explicitly enables `schemars/chrono04` when schema functionality is active. The upstream Rust MCP SDK still declares chrono as a target dependency on its checked main manifest, including in versions available after 3.9.0: https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/Cargo.toml
- Neither simply bumping chrono nor disabling schemars/chrono04 in Searchright removes the dependency through rmcp. A direct swap to Jiff or time in Searchright would likewise **not** remove rmcp's use of chrono. Removing/rewriting MCP SDK internals or maintaining a fork carries a high compatibility, supply-chain and release risk.
- **Decision pending owner/integrator:** seek an upstream rmcp patch removing chrono (and its schemars feature) with the same wire semantics, or implement a separately reviewed and compatibility-tested replacement/fork only if upstream does not resolve it. Before a change is admitted, verify SDK API compatibility, tool/request/response/transport behaviour, schema parity, all-feature dependency graph and a lockfile showing no chrono. Consult upstream; do not invent an available patched version.
- No blanket ignore, no advisory exception, no temporary suppression of cargo-deny or cargo-audit, no bypass of required branch protection. The residual advisory is open and continues to block the security gate until an actual qualified remediation exists.

## Evidence and claim boundaries

The URL fix and chrono hold are deliberately distinct. The security result cannot be inferred from the source-catalogue's green unit tests or from cargo-deny's other passed checks. Every PR must be rebased/reconciled with an accepted repaired `main` and satisfy all required exact-head CI before merging.

**Ownership:** Track 16 maintains dependency remediation, while Track 11 owns source-selection PR #844. GitHub issue #831 is the shared receiving record. There is no release, deployment, registry acceptance or supply-chain certification from this note.
