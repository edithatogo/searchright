# Exact dependency source review: MCP 3.9.0 and schema tooling 1.2.3

**Track:** 16. **Scope:** exact versions in Searchright's committed Cargo.lock.
**Reviewer:** OpenAI ChatGPT coding agent (AI-assisted source review, not a human or external attestation).
**Evidence date:** 2026-11-10. **Decision:** local source-vet evaluation for the locked Searchright feature graph, with the limitations below.

## Upstream identity and reviewed source

| Crate | Exact version | Reviewed upstream commit | Crate identity |
| --- | --- | --- | --- |
| `rmcp` | 3.9.0 | `modelcontextprotocol/rust-sdk` tag `rmcp-v3.9.0`, commit `5596a8a0bba2a1ec5519cc855cdadf23e4dcecdc` | crates.io index checksum `d24a7e92f4bcaa36b97fa93c4fa7ca8f3e669eced949345b3462dba333c7f909` |
| `rmcp-macros` | 3.9.0 | same tagged commit | Version/checksum from `Cargo.lock` |
| `schemars` | 1.2.3 | `GREsau/schemars` tag `v1.2.3`, commit `4791bb7352555d339e45658ca09c092ed9fa1f5d` | Version/checksum from `Cargo.lock` |
| `schemars_derive` | 1.2.3 | same tagged commit | Version/checksum from `Cargo.lock` |

**Identity limitation:** The tagged repository sources were inspected using GitHub blob contents and exact commit trees; the published crate archives were not separately unpacked and byte-compared to those tag trees. The crates.io index checksums and Cargo.lock are provenance/lock evidence, not proof that every tagged source byte equals the published crate archive.

## Scope and method

- Enumerated and mechanically inspected **76 source files** in the tagged `rmcp` and `rmcp-macros` source trees (~2.2 MB including embedded unit tests), and **48 Rust source files** in `schemars` and `schemars_derive` (~264 kB). Checked occurrences of `unsafe`, unconditional `unwrap`/`expect`, process/FS/network access and boundary/size-limit constructs.
- Read security-significant implementations rather than relying on scan results alone: `rmcp/src/transport/auth.rs` metadata host/IP policy and bounded OAuth fetch, `transport/common/client_side_http.rs` streaming body cap, reqwest and Unix-socket client call sites, `transport/streamable_http_server/tower.rs` request limits and protocol lifecycle, `service/client/cache.rs` bounded-by-default response cache, `transport/io.rs` stdio and test-only unsafe usage, `crates/rmcp/build.rs`, and macro handler and schema generation/derivation implementations.
- Compared upstream `rmcp-v3.8.0...rmcp-v3.9.0`: 3.9.0 adds bounded non-streaming HTTP response consumption and related regression tests, along with protocol/skills additions. Confirmed the upstream patched response-body cap is 16 MiB JSON and 64 KiB error bodies and that both reqwest and Unix-socket call sites use the bounded reader.
- Cross-checked the prior three RustSec findings: `RUSTSEC-2026-0457` (OAuth SSRF), `-0461` (unbounded server tool-schema cache), `-0462` (unbounded HTTP response buffering). The patched 3.9.0 branch's `cargo-deny` and `cargo-audit` passed in hosted checks before this local source-vet change.
- Inspected `schemars` schema generation/serialization invariants and `schemars_derive` token/attribute parsing. Panics in schema convenience macros and conversions are possible for unsupported or internally inconsistent values; no arbitrary network/process/FS side effects observed in source tree.
- Cross-checked published `safe-to-deploy` delta reviews of `schemars` and `schemars_derive` 1.2.2→1.2.3 from `nautechsystems/nautilus_trader` and `greenbone/rust-gvm-api`. Their attribution is preserved. They are **not** misrepresented as approved imports in Searchright's existing four-registry allowlist.

## Findings and residual risks

1. **No production `unsafe` blocks were located** in the inspected tagged Rust source. The `libc::fcntl` unsafe occurrence in `rmcp/src/transport/io.rs` is in a test-only module. This is a source-scan conclusion, not a formal memory-safety proof.
2. `rmcp` has a **build script** which may call `git config core.hooksPath .githooks` if built inside the upstream workspace with both `.git` and `.githooks`. The registry dependency build in Searchright does not satisfy that condition; nevertheless this side effect is recorded and should be considered for other build environments.
3. The **default MCP HTTP cache** caps entries at 512 and private entries are partitioned; callers can explicitly set the maximum to zero (unbounded). Searchright should not enable an unbounded cache or cross-tenant private caching.
4. The **OAuth metadata filtering** resolves addresses in the standard constructor, blocks internal/link-local classes as documented, and validates redirects. Calling `new_with_oauth_http_client` delegates address policy to the caller; Searchright must not treat it as protected by the default resolver.
5. No filesystem, process-spawn, network or `unsafe` source paths were located within either schema crate or the proc-macro crate. The proc-macro inputs can cause compiler errors; JSON schema generation is not a substitute for independent runtime payload validation.
6. Searchright enables the official `rmcp` server, client, elicitation, macros, stdio and schema features, plus optional Streamable HTTP **server** in its remote HTTP build. The full upstream SDK additionally exposes features not currently enabled; **this review does not validate future feature changes or custom OAuth/HTTP clients**.
7. The existing hosted Searchright all-features tests, cross-platform compilation, clean-room replay and static conformance provide integration evidence, **not** proof that upstream dependencies are free of unknown vulnerabilities. The same exact-head gates must rerun after this change.

## Review boundaries

- These are **attributed AI-assisted exact-version source reviews** and do **not** imply independent human security certification, penetration testing, formal verification, new authorisation for privileged hosting, or publisher endorsement.
- No old `rmcp 3.1.2` exemption is carried forward as a new approved exemption. All four local audit entries refer to precisely inspected versions; no approved peer import URLs, publisher wildcard trust, RustSec ignore list, CI rule or owner-authorisation ledger was modified.
- Subsequent security advisories, feature changes, new dependencies or upgrades require re-review and a fresh review identity. A successful `cargo-vet` graph check establishes **audit-policy coverage**, not a guarantee of absence of vulnerabilities.
- Historical receipts remain historical; a new workflow's results must be observed at its exact commit before merging.

## Sources

- [Official rmcp 3.9.0 tag](https://github.com/modelcontextprotocol/rust-sdk/tree/rmcp-v3.9.0)
- [Official 3.8→3.9 source comparison](https://github.com/modelcontextprotocol/rust-sdk/compare/rmcp-v3.8.0...rmcp-v3.9.0)
- [Schemars 1.2.3 tag](https://github.com/GREsau/schemars/tree/v1.2.3)
- [Nautech audit ledger](https://github.com/nautechsystems/nautilus_trader/blob/develop/.supply-chain/audits.toml)
- [Greenbone audit ledger](https://github.com/greenbone/rust-gvm-api/blob/main/supply-chain/audits.toml)
- [Searchright cargo-vet governance](cargo-vet-governance.md)
