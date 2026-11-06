**Advisory: DEFER** for `rustls-native-certs` **0.8.4**, criterion `safe-to-deploy`. The packaged tests have a concrete reproducibility blocker and substantial coverage gaps. This is testing advice; only the owner can adjudicate audit acceptance.

Line references below are relative to the reviewed package.

1. **High — packaged unit tests require omitted fixtures.**
   `src/lib.rs:380–381` unconditionally embeds `tests/badssl-com-chain.pem` and `integration-tests/one-existing-ca.pem` within the test module. Neither file is present. The packaging allowlist excludes those directories (`Cargo.toml:18–26`; `Cargo.toml.orig:12`). Consequently, compiling the unchanged library unit-test target should fail before any tests run, even with a test-name filter. This is a source-derived conclusion, not an observed compiler result.

2. **High — native platform behavior has no packaged tests.**
   The seven packaged unit tests exercise common file-loading helpers (`src/lib.rs:377–516`). There are no tests in the platform modules:
   - Windows store-opening errors, usage filtering, validity filtering, and the two error-unwrapping calls remain untested (`src/windows.rs:9–34`, especially line 18).
   - macOS domain precedence, explicit distrust, default trust, and continuation after native errors remain untested (`src/macos.rs:22–68`).
   - Unix discovery and its interaction with discovered files/directories remain untested (`src/unix.rs:3–9`).

   A successful common-code test run would not establish these behaviors; cross-compilation alone would not establish native behavior either.

3. **Medium — environment-selection and partial-error contracts lack direct tests.**
   Despite their names, the `from_env_*` tests call helpers directly. None exercises `CertPaths::from_env()` or public `load_native_certs()` routing (`src/lib.rs:118–124,198–210,427–494`). Missing cases include file/directory combinations, platform delimiters, empty values, and suppression of native fallback after an explicit invalid override. Empty `SSL_CERT_DIR` components are discarded, which also needs reconciliation with the “either … set” documentation (`src/lib.rs:55–57,205–209`).

   No packaged assertions cover valid certificates alongside errors, actual PEM parser errors, directory-entry/metadata failures, dangling symlinks, nonrecursive traversal, error formatting/source chains, or `expect`/`unwrap` panic behavior (`src/lib.rs:137–188,244–360`). The malformed-input test checks only that Rust source yields zero certificates without errors (`src/lib.rs:428–435`).

4. **Medium — existing tests have environmental assumptions and weak success assertions.**
   Permission tests expect mode `000` to produce `PermissionDenied`; privileged execution can invalidate that assumption (`src/lib.rs:468–511`). Relative nonexistent paths assume a suitable working directory (`src/lib.rs:440,450`), while `file!()` must resolve to readable source (`src/lib.rs:432`). The Unix empty-input case depends on `/dev/null` (`src/lib.rs:457–463`).

   Deduplication checks only certificate counts, without checking exact certificate identities or an empty error collection (`src/lib.rs:405–424`). It could miss unexpected errors accompanying the expected count.

5. **Medium — reproducibility information is incomplete.**
   A dependency lockfile is present, but manifests permit version ranges (`Cargo.toml:52–88`); downstream resolution need not match it. Rust 1.71 is declared (`Cargo.toml:14`), but neither minimum-version compatibility nor compatibility of the locked development graph is demonstrated here. No packaged CI configuration, toolchain pin, or execution receipts establish a platform matrix. README links to CI, contribution instructions, and an example do not supply that evidence (`README.md:16–21,57–59`). Its environment description also omits `SSL_CERT_DIR`, unlike the API documentation (`README.md:42–45`; `src/lib.rs:48–57`).

Safe ways for the owner to obtain missing evidence:

- Obtain the two exact fixtures separately with provenance tied to the recorded source revision. Preserve the registry package unchanged; identify fixture additions as a supplemental test overlay.
- Use disposable, unprivileged Linux, Windows, and macOS workers with preprovisioned toolchains/dependencies, isolated writable scratch space and caches, and offline locked resolution. Record toolchain, target, dependency graph, environment, privileges, commands, and results. Do not install tools or alter permissions on the shared runner to make tests pass.
- Exercise native trust changes only in disposable OS images/accounts. Add deterministic supplemental tests for error paths and trust decisions that cannot safely be induced against a real store. Pass override variables to separate child processes rather than modifying a shared test process’s environment.
- Keep supplemental evidence distinct from evidence for the unchanged package.

**Files read completely:** `/tmp/searchright-native-certs-review/request.json`; package files `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`, `README.md`, `src/lib.rs`, `src/unix.rs`, `src/windows.rs`, `src/macos.rs`, `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT`, and `LICENSE-ISC`. No separate safety/error document was packaged; applicable contracts are in the source documentation and comments.

**Assumptions and limitations:** I treated the supplied directory as the exact package identified by the request; I did not independently verify its archive digest. Dependencies, linked external documentation, upstream tests, and native API semantics were outside this isolated read scope. No peer outputs were read. No reviewed code or tests ran; no writes, network access, certification, or promotion occurred.
