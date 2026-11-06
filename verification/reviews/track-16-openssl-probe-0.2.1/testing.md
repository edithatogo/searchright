**Advisory disposition: DEFER** for the exact-version `safe-to-deploy` claim. Missing regression evidence accompanies observable environment-handling defects. This is not certification; the accountable owner alone adjudicates final disposition.

Findings from static inspection:

- **Directory-list handling lacks regression coverage and can discard configuration.** `src/lib.rs:121–127` treats `SSL_CERT_DIR` as one filesystem path. A colon-separated list of existing directories will ordinarily fail that existence check. `probe()` then substitutes system candidates (`src/lib.rs:82–86`), and initialization can overwrite the original list (`src/lib.rs:38–49`). Initialization itself produces colon-separated lists, so repeated calls need explicit stability tests.
- **Existence is insufficient evidence of usable certificates.** `src/lib.rs:76`, `84`, and `121` use only `exists()`. Wrong object types, empty or malformed certificate files, and directories without usable certificates can pass. The assurance that OpenSSL will find certificates (`src/lib.rs:103–108`) exceeds what is checked. Permission errors, dangling links and filesystem races also need coverage.
- **No behavioral regression tests are shipped.** The complete `src/lib.rs:1–225` contains no tests; no separate test files are present. The example (`examples/probe.rs:1–6`) only prints results. CI declares `cargo test` (`.github/workflows/main.yml:15–38`), but its configuration supplies neither execution receipts nor behavioral assertions.
- **The README example is stale.** `README.md:21` matches `cert_dir` as `Option`, whereas `src/lib.rs:116` declares `Vec<PathBuf>`. A compilation check of the published usage snippet would catch this; README code is not automatically a library doctest.
- **Safety and platform claims remain unverified.** The unsafe API documents environment-mutation obligations (`src/lib.rs:15–24`), but the no-write optimization for unchanged values (`src/lib.rs:52–60`) has no regression evidence, including the cited Perl scenario. Platform-specific tables (`src/lib.rs:137–219`) extend beyond the Linux/macOS/Windows CI matrix. Fallback Unix paths do not demonstrate native trust-store discovery on macOS or Windows.

Safe regression evidence to obtain in a separately authorized testing environment:

- Launch fresh helper processes with environment values supplied at process creation. Keep the multithreaded test runner’s environment unchanged. Exercise mutation only in controlled, single-threaded helpers satisfying the unsafe contract; `--test-threads=1` alone does not establish that contract.
- Cover unset, empty, nonexistent, relative, non-Unicode where supported, wrong-type, inaccessible, dangling-link and malformed-certificate paths.
- For `SSL_CERT_DIR`, cover multiple existing directories, mixed valid/invalid entries, leading/trailing/repeated separators, duplicates, and separator-containing names where supported.
- Assert precedence, fallback order, preservation of valid overrides, appended system directories, return values, unchanged-value behavior, and repeated-call stability.
- Use disposable filesystem fixtures or isolated filesystem images for hard-coded system locations. Actual OpenSSL loading and controlled local verification are needed to establish certificate usability; discovery assertions alone cannot.
- Obtain native platform evidence for relevant deployment targets. Compilation cannot validate path availability, permissions or trust-store behavior.

All files read, relative to the package root unless absolute:

`src/lib.rs`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `examples/probe.rs`, `README.md`, `.github/workflows/main.yml`, `.gitignore`, `.cargo_vcs_info.json`, `deny.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, and `/tmp/searchright-openssl-probe-review/request.json`. Safety documentation is embedded in `src/lib.rs`; no separate safety document was present.

Constraints and residual gaps: source was treated solely as data. No writes, network access, reviewed-code execution, tests, certification or promotion occurred. No peer outputs or unrelated files were read. Archive identity was supplied by the request and was not independently verified. The absent tests alone do not prove unsafety, but the specific defects and missing platform/runtime evidence prevent this reviewer from supporting an unconditional safe-to-deploy claim.
