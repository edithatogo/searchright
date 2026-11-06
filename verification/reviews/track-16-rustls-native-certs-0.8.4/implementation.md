**Advisory disposition: DEFER** for `rustls-native-certs` **0.8.4**, criterion `safe-to-deploy`. The implementation has an error-handling defect and an environment-override contract mismatch that warrant owner adjudication before acceptance.

All references below are relative to `/tmp/searchright-native-certs-review/source/rustls-native-certs-0.8.4`.

Findings:

1. **Medium — Windows certificate inspection can panic instead of returning errors.**
   `src/windows.rs:18` unconditionally unwraps both `valid_uses()` and `is_time_valid()`. If either returns an error, loading unwinds rather than recording it in `CertificateResult.errors`, potentially disrupting application initialization. Store-open failures are handled correctly at lines 9–14. Certificate inspection failures should likewise be recorded and handled explicitly. The circumstances that cause these dependency calls to fail remain outside this package-only review.

2. **Medium — Empty directory overrides fall back to native roots despite the documented contract.**
   `src/lib.rs:55–57` promises that setting either environment variable disables native loading. However, lines 205–209 discard empty directory entries, and lines 120–123 select the native loader when the resulting list is empty and `SSL_CERT_FILE` is absent. Thus `SSL_CERT_DIR=""`, or a delimiter-only value, takes the native path. On Windows and macOS this directly selects native stores; Unix subsequently delegates discovery to `openssl-probe`. This can select a broader trust source than the documented override implies.

3. **Medium verification gap — Packaged unit tests reference absent fixtures.**
   `src/lib.rs:380–381` uses `include_str!` for `tests/badssl-com-chain.pem` and `integration-tests/one-existing-ca.pem`. Neither exists in the supplied package inventory; the manifest inclusion list excludes those directories (`Cargo.toml:18–26`). Consequently, compiling this package’s unit-test configuration would encounter missing include files. This does not affect ordinary library compilation, but prevents standalone verification of the packaged deduplication test.

4. **Low — Public error and environment documentation is inconsistent with implementation.**
   `src/lib.rs:109` describes failure as `std::io::Error`, while the actual API returns accumulated `Io`, `Os`, and `Pem` errors (`src/lib.rs:118,130–134,357–360`). `README.md:42–45` omits `SSL_CERT_DIR` overrides, and `src/lib.rs:53` describes only colon delimiters although lines 201–206 use platform-specific splitting. Non-I/O PEM errors also lose their source path (`src/lib.rs:159–168`), making multi-directory failures harder to diagnose.

Other implementation conclusions:

- File and directory loading accumulates partial successes and errors, continues across entries, follows certificate symlinks, skips missing targets and nonregular directory entries, and avoids recursive traversal (`src/lib.rs:244–323`).
- Path-loaded certificates are sorted and deduplicated after aggregation (`src/lib.rs:261–263`). macOS deduplicates through DER map keys but returns unspecified order; Windows has no explicit sorting or deduplication. No universal ordering guarantee is documented.
- macOS precedence is implemented coherently as User → Admin → System with first entry retained, including explicit nontrust decisions (`src/macos.rs:22–56`). However, an error inspecting a higher-priority certificate skips insertion, allowing a lower-priority occurrence to contribute trust. Errors remain visible; consumers accepting partial results must account for that behavior.
- Empty success is possible. `expect()` and `unwrap()` check only whether errors exist, not whether certificates were found (`src/lib.rs:137–157`). Silent skipping of malformed PEM is explicitly documented.
- No package-owned `unsafe` blocks, build script, process execution, or network operations were found. Native API safety, PEM parsing, Unix discovery, and certificate comparison semantics cross dependency boundaries.

Files read completely:

- `/tmp/searchright-native-certs-review/request.json`
- `src/lib.rs`, including all embedded tests
- `src/unix.rs`
- `src/macos.rs`
- `src/windows.rs`
- `Cargo.toml`
- `Cargo.toml.orig`
- `Cargo.lock`
- `README.md`
- `.cargo_vcs_info.json`
- `LICENSE`
- `LICENSE-APACHE`
- `LICENSE-MIT`
- `LICENSE-ISC`

Assumptions and limitations: the supplied directory is the intended exact-version package; archive SHA-256 and provenance were not independently verified. Dependency source, linked external documentation, native platform behavior, and downstream error policy were not inspected. No separate safety/error document is present; applicable contracts are in the README and source documentation. **No builds or tests ran.** No writes, network access, certification, or promotion occurred.

Only the accountable owner can adjudicate final audit acceptance.
