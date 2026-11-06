**Advisory disposition: DEFER** for `rustls-native-certs 0.8.4`, criterion `safe-to-deploy`. The owner should adjudicate the platform error-handling and trust-policy concerns below before acceptance.

All line references are relative to the reviewed package.

1. **macOS: trust-domain errors can weaken effective precedence — medium, conditional.**
   `src/macos.rs:22–36,48–56` records errors and continues without reserving the affected certificate’s higher-priority decision. A certificate can subsequently be accepted from Admin/System when User/Admin settings could not be read. If the unavailable settings would deny trust, consumers using `.certs` despite `.errors` can accept a root contrary to that policy. Successful decisions preserve precedence, including denial (`:56–68`). Recommended resolution: require callers to reject incomplete trust results, or conservatively suppress affected fallback. Actual OS error triggers were not verified.

2. **Windows: certificate-property errors panic — medium, conditional availability risk.**
   `src/windows.rs:18` unconditionally unwraps `valid_uses()` and `is_time_valid()`. An error from either dependency call panics instead of populating `CertificateResult.errors`; panic-abort applications can terminate. Opening the store handles errors normally (`:9–14`). Error reachability requires dependency/platform qualification; no triggering certificate was established.

3. **Environment and filesystem inputs confer root-selection authority — deployment constraint.**
   `src/lib.rs:118–124,198–210` lets `SSL_CERT_FILE` or any nonempty `SSL_CERT_DIR` selection replace native trust. Directory loading accepts every regular file, follows symlinks, and imposes no filename, ownership or containment checks (`:267–304`). Explicit file loading has no regular-file check or resource limit (`:308–322`); special files or large inputs can block or exhaust resources. Metadata checking and opening are separate operations. These are material risks when environment variables or certificate paths are attacker-controlled; they are not evidence of malicious intent. An empty `SSL_CERT_DIR` falls back to native loading, despite the broader “either set” documentation (`:55–57`).

4. **Native root extraction does not establish native verification-policy equivalence.**
   Windows selects the current-user `"ROOT"` store and filters server-auth usage and time validity (`src/windows.rs:9–21,28–37`). macOS reduces dependency-provided TLS trust decisions to bare DER roots (`src/macos.rs:48–67`). Unix delegates location discovery to `openssl-probe` (`src/unix.rs:3–9`). The returned DER collection carries no separate platform policy metadata. Full native distrust, policy constraints and verification behavior therefore remain unproven. README `:6–11` recommends the platform verifier.

5. **Packaged test source is incomplete — verification limitation.**
   `src/lib.rs:380–381` includes two absent PEM fixtures. The package inventory contains neither referenced directory; the manifest include list excludes them (`Cargo.toml:18–26`). Consequently, the packaged unit-test target cannot compile as supplied. This is a source-level conclusion, not a test result.

Additional assessment:

- No package-local `unsafe`, raw-pointer manipulation, direct FFI, process launch, network operation, runtime write, hidden payload or malicious behavior was identified.
- `Cargo.toml:17` disables a package build script; none is present. Dependency build hooks and unsafe internals remain outside this review.
- PEM parsing delegates to `rustls-pki-types`; malformed sections may be silently skipped. Empty certificates with no errors are possible (`src/lib.rs:96–105,222–234,427–435`). Callers must assess both errors and usable roots.
- Public `expect`/`unwrap` intentionally panic on any recorded error (`src/lib.rs:137–157`). The documented error contract mentioning `std::io::Error` is incomplete relative to the actual PEM/OS/IO variants (`:107–109,355–360`).
- Runtime dependency boundaries are `rustls-pki-types`, `openssl-probe`, `schannel`, and `security-framework` (`Cargo.toml:52–55,81–88`). Their implementations, transitive dependencies and OS contracts are not certified here. Manifest ranges also allow resolutions beyond the included lockfile.

**Files read completely:** `/tmp/searchright-native-certs-review/request.json`; package files `src/lib.rs`, `src/unix.rs`, `src/macos.rs`, `src/windows.rs`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `README.md`, `.cargo_vcs_info.json`, `LICENSE`, `LICENSE-APACHE`, `LICENSE-ISC`, and `LICENSE-MIT`. No separate safety/error document exists in the supplied inventory; applicable contracts are in source documentation and README.

**Assumptions and limitations:** The supplied directory is assumed to represent the requested registry archive; archive identity and extraction integrity were not independently verified. Review was static and package-only. No dependencies, linked external documentation, peer outputs or adjacent packages were read. No writes, network access, builds, tests, reviewed-code execution, certification or promotion occurred. Only the accountable owner can accept the audit.
