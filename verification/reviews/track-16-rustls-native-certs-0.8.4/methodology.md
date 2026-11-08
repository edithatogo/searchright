**Advisory disposition: DEFER** for `rustls-native-certs` **0.8.4**, criterion `safe-to-deploy`. This is a complete static reading of the supplied package, not certification. Only the accountable owner can adjudicate audit acceptance.

Line references below are relative to the supplied package.

1. **Packaged tests lack required fixtures.** `src/lib.rs:380–381` uses `include_str!` for two absent files under `tests/` and `integration-tests/`. The package inclusion list excludes those directories (`Cargo.toml:18–26`). Consequently, the embedded unit-test target cannot compile as supplied. This does not establish a runtime-library defect, but prevents treating packaged tests as reproducible assurance. The owner needs version-bound fixture provenance and test evidence, or an explicit acceptance of this gap.

2. **Error-free does not mean a usable trust store.** Certificates and errors accumulate independently (`src/lib.rs:127–135, 253–264, 308–322`). Malformed content can be ignored silently, as documented and asserted by a test (`src/lib.rs:226–230, 427–435`). `expect`/`unwrap` reject reported errors but accept an empty certificate collection (`src/lib.rs:137–157`). The owner must decide acceptable partial-result, empty-store, logging and startup-failure behavior in the consuming application. These are deployment decisions, not assurances supplied by the crate.

3. **Environment configuration replaces native trust discovery.** A configured file or nonempty directory list selects explicit paths instead of native stores (`src/lib.rs:118–124, 197–210`). Directory loading follows symlinks and attempts every regular file without filename filtering (`src/lib.rs:267–305`). Empty directory entries are discarded, so an empty `SSL_CERT_DIR` alone falls back to native discovery. README coverage omits `SSL_CERT_DIR` (`README.md:42–45`), and API documentation describes only colon delimiters despite platform-specific splitting (`src/lib.rs:53, 201–208`). The owner needs an explicit environment/path-integrity policy and boundary-case evidence.

4. **Windows has unreported panic paths.** Opening the store produces a structured error, but certificate-use and validity queries use unchecked `unwrap()` (`src/windows.rs:9–18`). Whether those calls can fail in supported deployments remains a dependency/API assumption. The owner should require justification or failure-path evidence before accepting availability risk.

5. **Platform trust semantics need explicit acceptance.** macOS preserves the first successfully obtained setting in User/Admin/System order, but continues after domain or certificate-setting failures (`src/macos.rs:22–56`). Thus lower-priority settings can contribute when higher-priority reads fail; errors accompany the result. Windows opens current-user `ROOT` and filters usage/time (`src/windows.rs:9–21`); Unix delegates discovery to `openssl-probe` (`src/unix.rs:3–9`). These branches export certificate snapshots, not demonstrated equivalence to complete native TLS verification. The README itself recommends a platform verifier (`README.md:6–11`). Owner evidence should cover precedence, distrust, errors and intended platform behavior.

6. **Dependency and identity evidence remain separate.** Runtime dependency ranges appear at `Cargo.toml:52–55, 81–88`; the bundled lock records particular resolutions, including `rustls-pki-types` 1.14.1, `schannel` 0.1.29 and `security-framework` 3.5.1 (`Cargo.lock:608–615, 629–655`). Neither ranges nor checksums certify dependency implementations or establish the consuming application's resolved graph. The request supplies an archive hash (`request.json:7`), but I did not independently verify archive-to-directory correspondence. Version metadata and the VCS identifier are supporting metadata, not that proof.

The embedded tests cover deduplication, ignored malformed input, missing paths, empty/nonregular input and permissions (`src/lib.rs:377–517`). They provide no packaged platform-specific tests, actual environment-selection tests, or evidence for the Windows panic paths and macOS precedence failures. Permission tests also require an execution identity for which mode `000` actually denies access. No tests ran.

**Files read in full**

- `/tmp/searchright-native-certs-review/request.json`
- `src/lib.rs`, `src/unix.rs`, `src/macos.rs`, `src/windows.rs`
- `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`
- `README.md`, `.cargo_vcs_info.json`
- `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-ISC`

No separate safety/security documentation, fixtures, integration tests, examples or CI definitions were present. Linked external documentation was not accessed.

**Assumptions and limitations:** The supplied directory is assumed to represent the requested registry package. Dependency implementations, OS contracts, consumer behavior and actual deployment targets remain outside this isolated review. I read no peer outputs or adjacent packages, performed no writes or network access, and executed no reviewed code. Configuration controls, future test receipts and this advisory review must remain distinguishable from owner-issued audit acceptance.
