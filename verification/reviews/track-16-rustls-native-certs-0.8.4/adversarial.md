**Advisory: DEFER** unconditional `safe-to-deploy` acceptance for rustls-native-certs 0.8.4. Static review identified the following counterexamples and unresolved platform boundaries; none was executed.

1. **Windows: native-query failures panic instead of returning errors.**
   [src/windows.rs:18](/tmp/searchright-native-certs-review/source/rustls-native-certs-0.8.4/src/windows.rs:18) unconditionally unwraps both `valid_uses()` and, when reached, `is_time_valid()`. An `Err` from either aborts loading through panic, bypassing `CertificateResult.errors`. This control-flow defect is certain; whether an attacker can induce those errors through an installable certificate is unverified because Schannel implementation and OS behavior are outside scope. This differs from callers explicitly choosing the documented panic helpers at `src/lib.rs:138–157`.

2. **macOS: failure to read higher-priority restrictions permits lower-priority trust.**
   At [src/macos.rs:24](/tmp/searchright-native-certs-review/source/rustls-native-certs-review/source/rustls-native-certs-0.8.4/src/macos.rs:24), domain-read errors continue to subsequent domains; certificate-specific errors similarly continue at lines 48–53 without inserting a restrictive entry. A certificate whose User-domain restriction cannot be read can consequently acquire System-domain `TrustRoot` at lines 56–67. Errors remain visible, so this is a **conditional fail-open result**, requiring a caller that consumes `certs` despite `errors`. No reproducible OS failure or exploit was demonstrated.

3. **Configured inputs permit blocking and unbounded accumulation.**
   [src/lib.rs:308](/tmp/searchright-native-certs-review/source/rustls-native-certs-0.8.4/src/lib.rs:308) passes explicit file paths directly to the PEM reader without regular-file checks or deadlines. A caller-selected Unix FIFO with no writer can block opening. Repeated valid PEM certificates accumulate at line 319 before deduplication at lines 261–263, allowing memory exhaustion with sufficiently large input. Directory loading also follows symlinks and reads every regular file, with no count/size limit (lines 277–304). These require configuration or filesystem influence; no remote TLS-peer trigger was established.

4. **Empty successful results do not establish successful trust-store loading.**
   The packaged test at [src/lib.rs:427](/tmp/searchright-native-certs-review/source/rustls-native-certs-0.8.4/src/lib.rs:427) explicitly expects noncertificate source text to produce zero certificates and zero errors. The behavior is documented at lines 96–105 and 226–230. Additionally, an empty/delimiter-only `SSL_CERT_DIR` is filtered away at lines 205–208, causing native-store fallback at lines 120–123 despite the “either … set” override contract at lines 55–57. Neither alone demonstrates unauthorized trust; callers must distinguish empty results and effective configuration.

**Evidence limitations**

- Embedded tests reference absent `tests/badssl-com-chain.pem` and `integration-tests/one-existing-ca.pem` (`src/lib.rs:380–381`); the exact package therefore lacks fixtures needed to compile those unit tests.
- No packaged Windows/macOS tests establish failure handling or native trust-policy equivalence.
- macOS policy interpretation, Windows enumeration semantics, PEM parsing, and Unix discovery remain dependency trust boundaries. Returning DER roots does not itself prove equivalence to native certificate verification.
- Archive identity/hash was supplied by the request, not independently verified.
- No tests ran. No reviewed code was executed; no network, writes, certification, or promotion occurred.

**All files read**

`/tmp/searchright-native-certs-review/request.json`, and package files: `src/lib.rs` (including all embedded tests and API/error documentation), `src/unix.rs`, `src/windows.rs`, `src/macos.rs`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `README.md`, `.cargo_vcs_info.json`, `LICENSE`, `LICENSE-APACHE`, `LICENSE-ISC`, `LICENSE-MIT`. No separate safety/error document was present. No peer outputs, adjacent packages, or unrelated files were read.

Only the accountable owner can adjudicate audit acceptance.
