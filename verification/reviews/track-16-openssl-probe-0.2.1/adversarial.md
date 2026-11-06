**Advisory: DEFER** for `openssl-probe 0.2.1` against `safe-to-deploy`. Static inspection found concrete trust-configuration defects requiring owner adjudication. These do not establish a remote exploit or memory-safety defect in the safe API.

All paths below are relative to the package root.

1. **Colon-separated `SSL_CERT_DIR` is interpreted as one filesystem path.**
   `src/lib.rs:120–127` converts the entire value to `PathBuf` and checks its existence. With `SSL_CERT_DIR=/private/ca:/partner/ca`, where both directories exist but the literal combined pathname does not, the override is discarded. `probe()` then supplies system directories (`src/lib.rs:82–86`), and the unsafe initializer can replace the original value (`src/lib.rs:38–49`). This can remove intended roots and introduce system roots outside an application's intended trust set. Actual TLS impact depends on downstream loading behavior.

2. **An existing directory override is broadened and duplicated.**
   `src/lib.rs:82–86` unconditionally appends system directories, even when `from_env()` accepted an override. For example, `/private/ca` becomes `/private/ca:/etc/ssl/certs` when that system directory exists. This contradicts the preservation promise at `src/lib.rs:8–9` and the initializer's assumption at `src/lib.rs:30–31`. Setting the override to `/etc/ssl/certs` produces a duplicate; repeated initialization can then encounter finding 1 and discard the combined value. Duplicates alone appear principally redundant, but override broadening is security-relevant.

3. **Existence is presented as stronger validation than implemented.**
   `src/lib.rs:76` and `src/lib.rs:121` use only `exists()`. A directory supplied as `SSL_CERT_FILE`, a regular file supplied as `SSL_CERT_DIR`, or an empty/non-certificate file can be accepted. Consequently, `has_ssl_cert_env_vars()` can return true without usable certificates, contrary to the implication at `src/lib.rs:103–108`. Paths can also change between probing and downstream opening; no file handle, ownership check or immutable snapshot binds the result. Exploitation requires control of the environment or relevant filesystem paths; this is not independently demonstrated privilege escalation.

4. **README usage is incompatible with the runtime API.**
   `README.md:21` matches `cert_dir` as `Option`, but `src/lib.rs:116` declares `Vec<PathBuf>`. The example is therefore inconsistent by inspection. This is a documentation defect, not independently grounds for rejecting `safe-to-deploy`.

**Other boundaries assessed**

- The environment initializer is explicitly `unsafe`, with environment-mutation safety documentation at `src/lib.rs:15–24`. Its comparison before `set_var` (`src/lib.rs:52–59`) provides no synchronization. Concurrent environment access remains a caller obligation; I found no safe wrapper that conceals this obligation.
- Target-specific constants and fallback branches (`src/lib.rs:137–219`) appear mutually exclusive and exhaustive by inspection. Unix-style fallback paths and unconditional colon joining do not demonstrate native trust-store support on every target.
- The normalized manifest declares no dependencies or build script. The package contains no behavioral tests; its CI configuration is not evidence that checks passed.

**Files read completely**

`/tmp/searchright-openssl-probe-review/request.json`; package files `src/lib.rs`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `examples/probe.rs`, `README.md`, `deny.toml`, `LICENSE-APACHE`, `LICENSE-MIT`, `.github/workflows/main.yml`, `.cargo_vcs_info.json`, and `.gitignore`. Safety documentation is embedded in `src/lib.rs`; no separate safety document was present.

**Constraints and residual gaps**

Read-only source inspection only: no writes, network, reviewed-code execution, compilation or tests. No peer outputs or unrelated files were read. Archive identity was supplied by the request, not independently verified. Downstream TLS behavior, platform behavior and exploitability were not experimentally established.

The accountable owner alone adjudicates final audit disposition; this advisory neither certifies nor promotes the package.
