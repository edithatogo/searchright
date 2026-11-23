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

## RUSTSEC-2026-0543: chrono — pinned upstream candidate, pending source acceptance

- Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0543.html (unmaintained; **no patched chrono version**).
- The previously released `rmcp =3.9.0` uses chrono unconditionally. Current upstream **main** retains chrono; no released crate has been demonstrated to remove it with compatible features. Upstream [issue #1469](https://github.com/modelcontextprotocol/rust-sdk/issues/1469) and **unmerged** [PR #1474](https://github.com/modelcontextprotocol/rust-sdk/pull/1474) address this using Jiff, preserving timestamp wire format and offering an optional default-on chrono feature.
- [POC PR #849](https://github.com/edithatogo/searchright/pull/849), on pinned upstream commit `9043cdfa8694d83bf506c2f3eb0313ec173edd7e`, **successfully** resolved all workspace features without chrono and compiled `searchright-mcp` with default rmcp features disabled. [Read-only job 137579900619](https://github.com/edithatogo/searchright/actions/runs/45141368128/job/137579900619): `CHRONO_ABSENT_IN_ALL_FEATURE_WORKSPACE_GRAPH`, `MCP_BUILD_SUCCEEDED_ON_EXACT_CANDIDATE`.
- The same configuration is **provisionally implemented on this draft branch**: exact Git pin to the unmerged upstream revision, `default-features=false`, explicitly selected `base64, server, client, elicitation, macros, transport-io, schemars`. No `chrono` feature and no `auth` feature (which can pull chrono via oauth2).
- A second **read-only temporary GitHub Actions job** [137581580424](https://github.com/edithatogo/searchright/actions/runs/45141870326/job/137581580424) ran Cargo 1.97.1, resolved the **entire** candidate workspace graph, asserted chrono absent, printed the generated Cargo.lock patch, and successfully compiled `searchright-mcp` with all its features. The resulting Cargo.lock blob `651252ff7770663373d2b91426e4b14423359389` matches the generated diff target prefix `651252f`, across 12 hunks. The runner-only generator was then **deleted**. The commit introduces Jiff 1 and related Cargo-resolved transitive changes; it removes chrono and orphaned dependencies.
- Permanent `tests/test_rmcp_chrono_free_lock.py` asserts the exact source revision, default-off feature set, resolved dependency identity, chrono absence, and unchanged strict `deny.toml` policy.
- **Open source/compatibility gate:** upstream PR #1474 is **unmerged** and dependency semantics/maintenance have not received upstream release acceptance. Building and resolving its exact source is not sufficient supply-chain certification. Before **production merge**, evaluate its source, legal/supply-chain/audit provenance and stability, and achieve exact-head compiler, schema/Serde, MCP CLI/stdio/HTTP, API, contract, regression, cargo-deny, cargo-audit, cargo-vet, downstream and security checks. If any are unavailable or fail, keep the PR **draft** and advise the Track 16 owner; prefer an upstream merged and published release when available. Do not introduce a blanket waiver, ignore or bypass branch protection.

## Evidence and claim boundaries

The URL fix and chrono hold are deliberately distinct. The security result cannot be inferred from the source-catalogue's green unit tests or from cargo-deny's other passed checks. Every PR must be rebased/reconciled with an accepted repaired `main` and satisfy all required exact-head CI before merging.

**Ownership:** Track 16 maintains dependency remediation, while Track 11 owns source-selection PR #844. GitHub issue #831 is the shared receiving record. There is no release, deployment, registry acceptance or supply-chain certification from this note.
