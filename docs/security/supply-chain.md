# Supply-chain policy

Searchright uses a default-deny, exact-pin supply-chain model.

- Rust dependencies are resolved through a committed `Cargo.lock` before any
  compiler-backed claim or release.
- Cross-repository integrations use exact 40-character revisions recorded in
  versioned integration passports and `integration/locks.json`.
- Producer–consumer interactions declare contracts, fixtures, gates and
  fail-closed behaviour; passing declarations do not promote compatibility.
- Internal path dependencies carry the exact workspace version. Packages with
  Git-only dependencies remain `publish = false`; the publishable facade cannot
  depend on them.
- Drift surveillance is read-only and cannot automatically update a revision,
  dependency, issue, pull request or public claim.
- GitHub Actions use full commit SHAs. Checkout credentials are not persisted.
- Cargo-installed CI tools are installed one at a time with exact versions and
  `--locked`.
- `cargo-deny`, `cargo-audit`, dependency review, CodeQL, OpenSSF Scorecard,
  Gitleaks and zizmor cover complementary dependency, code and workflow risks.
- Clean-room builds vendor the locked graph and compile offline from a
  reproducible source archive.
- Source archives, source SBOMs and release binaries are checksummed and
  attested. Attestation is provenance evidence, not a claim that software is
  vulnerability-free.
- Optional WASI provider components require exact digests and declared
  capabilities before loading.

Renovate may propose changes but must not bypass checks, alter integration pins
without a passport update, or merge a change that raises the evidence claim.
