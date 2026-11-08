**Advisory: approve**, with the limitations below, for the inspected `openssl-probe 0.2.1` source. No blocking memory-safety defect or malicious behavior identified. This is not certification; the accountable owner alone adjudicates the final audit disposition.

Findings and caller obligations:

- **Environment mutation requires process-wide safety guarantees.** `src/lib.rs:25–60` exposes mutation through an `unsafe` public function and documents the hazard at `:15–24`. The sole unsafe operation is `env::set_var`. The equality check at `:56` avoids redundant writes but provides no synchronization. Callers must satisfy the standard-library environment-mutation contract, including concurrent native environment readers; a lock around this crate alone is insufficient.
- **Existing directory configuration can be replaced or broadened.** `src/lib.rs:121–126` treats `SSL_CERT_DIR` as one filesystem path rather than parsing a directory list. An ordinary colon-separated list can fail the existence check and be replaced during initialization. Even a recognized existing directory receives additional system directories at `:82–85`, subsequently serialized at `:38–49`. This conflicts with the preservation promise at `:8–9` and can broaden an intentionally restricted trust configuration.
- **Existence is not trust-anchor validation.** `src/lib.rs:76–85`, `:96–100`, and `:121` check only existence. They do not validate object type, certificate contents, ownership, permissions, or stability between discovery and use. Environment paths may be relative or symlinks. Consequently, the success implication at `:106` is too strong. Consumers must enforce their own trust policy and handle loading failures.
- **README example is stale.** `README.md:21` matches `cert_dir` as an `Option`, but `src/lib.rs:116` defines a `Vec<PathBuf>`. This is a documentation correctness issue.

Security inspection:

- No raw-pointer manipulation, FFI, custom allocation, or other unsafe operations beyond environment mutation.
- Filesystem activity consists of path-existence checks; no file-content reads, writes, directory traversal, or permission changes.
- No networking, subprocess execution, secret collection, or export behavior. The example prints discovered paths (`examples/probe.rs:4–5`).
- No declared dependencies or build script; `Cargo.toml:18` explicitly disables the build script. Packaged CI references external actions, but these are not dependency-build hooks.

Files read completely:

- `/tmp/searchright-openssl-probe-review/request.json`
- Package-relative: `src/lib.rs`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `examples/probe.rs`, `README.md`, `deny.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, `.cargo_vcs_info.json`, `.gitignore`, `.github/workflows/main.yml`.

The safety documentation is embedded in `src/lib.rs`; no separate safety document was present.

Constraints and residual gaps: static inspection only. No writes, network access, reviewed-code execution, builds, or tests were performed. Archive digest/provenance, platform behavior, actual certificate loading, and downstream caller compliance were not independently verified. No peer reviewer outputs or unrelated files were read.
