# Local candidate API evidence (Track 03)

`scripts/run_local_api_evidence.py` mirrors the candidate package set and API
commands in `.github/workflows/public-api.yml`, without its installation or
upload steps. The workflow and publication policy are unchanged. It is not a
new public wire contract, release gate waiver or downstream cutover decision.

## Invocation and safety boundary

From a clean committed worktree, choose an explicit locally available base:

```sh
python3 -B scripts/run_local_api_evidence.py --base <chosen-local-base>
python3 -B scripts/run_local_api_evidence.py --base <chosen-local-base> --execute
```

The first command performs only local Git/policy inspection and emits a JSON
plan. It does not probe Cargo/rustup, create output or install anything. The
second explicitly executes trusted local tools and crate/build-script code.
The base is resolved once to an exact commit; there is no default branch, parent
or remote fetch fallback. Both HEAD/base commits and trees are recorded. A
committed lockfile, current candidate manifests and exact candidate-policy pins
are required. Untracked source also makes the worktree dirty.

Installed toolchains must include `1.97.1` and `nightly-2026-08-11`; tool versions
must be cargo-public-api `0.52.0` and cargo-semver-checks `0.50.0`. All Cargo
invocations use `rustup run`, including version probes: a standalone Homebrew
Cargo on PATH must not silently replace the pinned compiler. No toolchain
default is changed. Missing tools, wrong versions or unavailable offline
dependencies cannot produce a passing comparison.

Nonempty compiler, rustdoc, wrapper and Rust/rustdoc flag environment overrides
are rejected before tool execution, not silently cleared. The receipt records
only rejected variable names, never their values. This includes `RUSTC`,
`RUSTDOC`, `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, their `CARGO_BUILD_*`
variants, and ordinary/encoded Rust and rustdoc flag variables. Repository and
user/global Cargo configuration are still trusted inputs; this is not a
hermetic compiler attestation. Review those configurations before execution.

`CARGO_NET_OFFLINE=true` and `RUSTUP_AUTO_INSTALL=0` prevent normal Cargo network
resolution and rustup auto-installation. These flags are **not an OS sandbox**:
use only trusted binaries, repository code and Cargo configuration. Build scripts
or malicious tools could otherwise perform arbitrary actions, including network
access. The runner does not install tools, publish, upload, change remote state,
execute provider calls, or authorize package promotion.

Output defaults to `target/local-api-evidence/<head-prefix>-<base-prefix>`.
`--output` may select another child directory under this repository's `target`.
It must be absent or empty and cannot traverse a symlink beneath the repository
root. Existing nonempty output is refused, not cleaned. Log/receipt files use
exclusive creation; failures preserve already-written evidence. Use a new output
directory for each repeat. This is no-clobber protection for cooperative local
use, not a sandbox against concurrent malicious filesystem mutation.

## Evidence and result interpretation

JSON is emitted on stdout. Execution also writes `receipt.json`, separate local
stdout/stderr logs, and compiler build artefacts under the selected output. Each
command record includes exact argument array, cwd, enforced environment, exit
code, byte lengths and SHA-256 log/artifact digests. API stdout itself is the
captured textual API artifact. Observed version strings are reduced to version
tokens in the receipt; original probe output remains in local hashed logs.
Review logs before sharing: compiler/tool output is not assumed secret-free.

- `dry_run`: planned only; no compiler/tool execution.
- `tool_unavailable`: missing/mismatched toolchain or tool; comparisons skipped.
- `initial_capture_only`: captures succeeded, but at least one candidate did not
  exist in the exact base. This is **not** complete compatibility evidence; each
  existing candidate still has its independent comparison result.
- `passed`: every candidate capture was nonempty/successful and every applicable
  comparison returned success, with no missing baseline candidates.
- `incompatible`: a nonzero comparison emitted the specifically recognized
  `semver requires new major/minor version` diagnostic. Other nonzero exits are
  conservatively `failed`, not automatically classified as incompatibility.
- `failed`: capture/tool/compiler/comparison failure; dependent comparisons are
  explicitly `skipped_capture_failed` where appropriate.
- `source_changed`: the final HEAD/tree/clean-state check differs from the
  inspected source; earlier command success cannot validate a moving checkout.
- `blocked`: Git/policy/path/write precondition failed. Existing output remains.

Dry-run, passed and initial-capture-only return exit zero; callers must inspect
the structured status rather than interpreting zero as compatibility. All other
states return nonzero. Exact-revision rechecks detect persistent source changes,
not hostile transient changes or mutable external tool/configuration inputs.

This local comparison does not establish hosted CI, consumer compatibility,
source rights, publication readiness, SemVer policy acceptance or rollback.
Track 03's downstream and owner gates remain separate. No actual Cargo API run
was used to validate the runner implementation: its tests inject deterministic
command results and use temporary output directories.

## Tests

```sh
python3 -B -m unittest discover -s tests -p test_local_api_evidence.py
```

The initial test-first attempt failed because the runner module did not yet
exist. The first implemented run exposed a macOS `/var` versus `/private/var`
root-alias issue in explicit output handling; lexical paths beneath the original
root now map to its resolved root before checking descendant symlinks.
