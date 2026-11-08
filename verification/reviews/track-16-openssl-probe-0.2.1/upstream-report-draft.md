# Unpublished upstream report draft: SSL_CERT_DIR handling in 0.2.1

Status: draft for owner review. Not submitted. Do not describe this as a
confirmed remote exploit or as a demonstrated Searchright vulnerability.

## Suggested title

`probe()` discards colon-separated SSL_CERT_DIR overrides and appends system directories

## Package and environment

- openssl-probe 0.2.1, registry archive SHA-256
  `7c87def4c32ab89d880effc9e097653c8da5d6ef28e6b539d313baaacfbafcbe`.
- Rust 1.97.1 on aarch64-apple-darwin.
- Existing candidate directory on the observed host: `/etc/ssl/certs`.
- Reproduction used safe APIs and separate helper processes. It did not invoke
  the unsafe environment initializer or perform TLS handshakes.

## Minimal reproduction

Create two existing temporary directories, A and B. Run a fresh helper process
with `SSL_CERT_DIR=A:B` and an empty `SSL_CERT_FILE`. Have the helper print the
results of `has_ssl_cert_env_vars()` and `probe()` without changing its environment.

Observed: environment recognition is false; neither A nor B appears in the
returned `cert_dir`. Instead, `/etc/ssl/certs` is returned on this host.

Run a fresh helper with `SSL_CERT_DIR=A`. Environment recognition is true, but
`probe().cert_dir` contains both A and `/etc/ssl/certs`.

The first behavior follows from converting the entire directory-list string
into one `PathBuf` and testing its existence (`src/lib.rs:120-127`). The second
follows from appending system candidates after accepting an override
(`src/lib.rs:82-86`). The initializer documentation promises preservation of
accessible overrides, and the initializer serializes returned directories back
into a colon-separated environment value. We did not execute that unsafe API.

## Expected behavior / clarification requested

Please clarify whether environment overrides should be exclusive, and whether
colon-separated directory lists should be parsed with platform-aware path-list
handling. If overriding directories are intentionally supplemented with system
locations, please document that explicitly rather than implying preservation of
the original trust selection.

A secondary documentation issue is that the README matches `cert_dir` as an
`Option`, while the version 0.2.1 API defines it as `Vec<PathBuf>`.

## Security and downstream qualification

The probe result can differ from caller-selected trust paths. Actual certificate
acceptance depends on downstream loading behavior; no exploit or TLS acceptance
was demonstrated. In particular, rustls-native-certs 0.8.4 parses explicit
environment overrides before its platform fallback, which limits direct
applicability to that normal consumer path.

This report should be triaged for the appropriate upstream reporting channel
before publication. No severity, disclosure deadline, CVE request or upstream
maintainer response is asserted.
