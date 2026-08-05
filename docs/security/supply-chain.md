# Supply-chain controls

- Pin Rust toolchain and commit `Cargo.lock` before the first release.
- Run cargo-deny, cargo-audit, cargo-vet, cargo-machete and duplicate-dependency
  review.
- Pin GitHub Actions by immutable digest during hardening track 14.
- Generate CycloneDX and SPDX SBOMs, SLSA provenance and checksums.
- Sign release artefacts and OCI images with Sigstore/cosign.
- Run CodeQL, OpenSSF Scorecard, zizmor, actionlint and container scanning.
- Keep experimental git dependencies feature-gated, revision-pinned and excluded
  from default release features until reviewed.
