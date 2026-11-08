**Advisory disposition: DEFER** for `openssl-probe 0.2.1`. Static inspection found correctness and trust-selection concerns, but no evidence of malicious behavior. This is not certification or a safe-to-deploy authorization; that decision belongs to the owner.

Findings, with paths relative to the package root:

- **Medium — directory-list overrides are mishandled.** `src/lib.rs:121–126` treats `SSL_CERT_DIR` as one filesystem path rather than a colon-separated list. An override such as `/custom/a:/custom/b` will ordinarily fail the existence check and be discarded. Probing then substitutes system directories, and initialization can overwrite the original value (`src/lib.rs:38–49`). This can change intended trust anchors.
- **Medium — existing directory overrides are expanded.** `src/lib.rs:82–86` appends every existing candidate directory even when the environment supplied a valid directory. Initialization writes the expanded list back. This contradicts the preservation promise at `src/lib.rs:8–9` and the implementation comment at `src/lib.rs:30–31`; callers intending a restricted trust directory may receive additional system trust.
- **Low — existence is overstated as certificate usability.** `src/lib.rs:76`, `84`, and `121` check existence without checking object type, readable certificate contents, directory hash layout, ownership, or permissions. Consequently, `has_ssl_cert_env_vars()` can return true for unusable or inappropriate paths, despite the stronger implication in `src/lib.rs:103–108`.
- **Low — README example has an API mismatch.** `README.md:21` matches `cert_dir` as an `Option`, while `src/lib.rs:116` declares `Vec<PathBuf>`. The displayed usage is invalid for this version.

Safety and authority assessment:

- Environment mutation is confined to the explicitly unsafe `try_init_openssl_env_vars()` API (`src/lib.rs:25–64`). Its safety documentation identifies process-environment hazards and points to standard-library obligations. The equality check before `set_var` avoids redundant writes; it does not establish concurrency safety. Caller obligations remain.
- Filesystem activity consists of existence checks, including arbitrary environment-supplied paths. Relative paths and symlinks are accepted; paths are not pinned against later changes. The library neither reads certificate contents nor establishes their trustworthiness.
- Target-specific constants cover the listed Unix-family platforms; Android uses a Termux-specific certificate file. Remaining targets receive generic Unix paths (`src/lib.rs:137–219`). There is no native Windows certificate-store or macOS Keychain integration.
- No runtime network access, subprocess execution, filesystem writes, obfuscated payloads, or dependencies were observed. The normalized manifest explicitly disables a build script. The example probes and prints paths.

Complete files read:

- `/tmp/searchright-openssl-probe-review/request.json`
- `src/lib.rs`
- `Cargo.toml`
- `Cargo.toml.orig`
- `Cargo.lock`
- `examples/probe.rs`
- `README.md`
- `deny.toml`
- `.github/workflows/main.yml`
- `.cargo_vcs_info.json`
- `.gitignore`
- `LICENSE-MIT`
- `LICENSE-APACHE`

Safety documentation was embedded in `src/lib.rs`; no separate safety document appeared in the package inventory.

Constraints and residual gaps: read-only source inspection only. No reviewed code, builds, or tests ran; no network access, writes, peer-review reads, certification, or promotion occurred. Archive checksum/provenance was not independently verified. Actual platform paths, OpenSSL loading behavior, permissions, concurrent environment access, and downstream caller compliance remain unverified.
