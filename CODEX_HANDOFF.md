# Codex handoff: finish and publish Searchright

Use this file as the execution contract after unpacking
`searchright-complete-git.zip`. Work through it without replacing the
contract-first architecture, weakening gates, or marking external evidence as
complete.

## Required outcome

1. Preserve the supplied Git history and make all further changes as commits.
2. Generate and commit `Cargo.lock`.
3. Compile, test and repair the entire Rust workspace without removing planned
   capabilities merely to make the build pass.
4. Create `edithatogo/searchright`, wire `origin`, push `main`, apply repository
   settings and protections, create the complete nested issue hierarchy, and
   create/populate the GitHub Project v2.
5. Verify the remote control plane and GitHub Actions, then report exact URLs,
   commits, receipts, blockers and next external gates.

## 1. Verify the delivered repository

```bash
git status --short
git fsck --full
git log --oneline --decorate --graph -20
git remote -v
python scripts/run_static_harness.py
```

The working tree must be clean. The delivery is intended to have no configured
remote. Inspect any unexpected remote; never overwrite an unrelated repository.

Read, in this order:

```text
AGENTS.md
CONTEXT.md
context/manifest.json
conductor/requirements.md
conductor/design.md
conductor/tracks.md
conductor/roadmap-coverage.json
conductor/github/issue-hierarchy.json
conductor/github/project.json
conductor/github/repository-settings.json
PROJECT_STATUS.md
```

## 2. Establish the compiler-backed baseline

Install the exact toolchain from `rust-toolchain.toml`, then the pinned core
developer tools:

```bash
rustup toolchain install 1.97.1 --profile minimal \
  --component clippy,rustfmt,rust-src,rust-docs,llvm-tools-preview
rustup target add \
  x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
  wasm32-wasip2 aarch64-apple-darwin x86_64-pc-windows-msvc \
  --toolchain 1.97.1
python -m pip install --disable-pip-version-check -r requirements/validation.txt
python scripts/install_dev_tools.py --profile core
```

Then run:

```bash
cargo generate-lockfile
./scripts/bootstrap.sh
./scripts/verify.sh
```

Repair all failures at their source. Keep dependency and feature choices
bleeding-edge unless an incompatibility is demonstrated and recorded in an ADR.
Do not delete contracts, crates, tests, Conductor tracks, providers or assurance
surfaces to obtain a green build. Add focused unit, integration, property and
contract tests for each repair.

Install the extended pinned tools and run the applicable deeper gates:

```bash
python scripts/install_dev_tools.py --profile all
cargo llvm-cov nextest --workspace --all-features --fail-under-lines 91
cargo mutants --workspace
```

Run configured fuzz, Miri, Kani, Loom and `cargo-careful` workflows where the
host supports them. Record unsupported targets as explicit evidence gaps rather
than silently skipping them.

Commit the lockfile and every compiler-backed repair in logical commits. Re-run
`./scripts/verify.sh` and require a clean working tree before any remote write.

## 3. Authenticate GitHub safely

Use GitHub CLI authenticated as `edithatogo`. The token must be able to create
and administer `edithatogo/searchright`, create issues and native subissues,
manage Actions variables/secrets/environments/rulesets, and manage the
user-owned Project v2.

```bash
gh auth status
gh auth refresh -h github.com -s repo,workflow,project
```

Do not print tokens. Use a dedicated Project-capable token where available. To
wire future Project synchronisation, pass it directly through the environment
without committing it:

```bash
export SEARCHRIGHT_PROJECT_TOKEN_VALUE="$(gh auth token)"
export SEARCHRIGHT_GITHUB_BOOTSTRAP_APPLY=1
```

## 4. Create and synchronise the complete GitHub control plane

First confirm the mutation-free plan:

```bash
python scripts/bootstrap_github.py --create-project --sync-task-state
```

Then apply it:

```bash
python scripts/bootstrap_github.py \
  --apply \
  --create-project \
  --sync-task-state \
  --receipt-path .searchright/receipts/github-bootstrap.json
```

The synchronisers are convergent and checkpointed. A transient failure is
recovered by rerunning the same command. For bounded manual recovery use
`--resume-after` and `--max-nodes` with `sync_github_issues.py`, or
`--resume-after` and `--max-items` with `sync_github_project.py`. Checkpoints and
observed remote IDs remain under ignored `.searchright/receipts/`; do not commit
them into canonical source contracts.

Expected canonical projection:

- 1 roadmap epic;
- 38 track issues;
- 152 phase subissues;
- 373 task subissues;
- 564 Project items in total;
- 563 native parent-child relationships;
- 12 custom Project fields;
- 5 Project views.

Run the read-only remote audit:

```bash
python scripts/audit_github_control_plane.py \
  --receipt-path .searchright/receipts/github-control-plane-audit.json
```

The audit must pass. Resolve drift through the manifests and synchronisers,
never by weakening the expected counts or deleting canonical work.

## 5. Verify CI, Project and repository wiring

Confirm the remote, ruleset, environments, Actions secret, issue hierarchy and
Project are visible. Wait for the initial Actions runs and require all blocking
checks to pass:

```bash
gh repo view edithatogo/searchright
gh run list --repo edithatogo/searchright --limit 30
gh project list --owner edithatogo --limit 100
gh issue list --repo edithatogo/searchright --state all --limit 1000
```

Run the GitHub control-plane workflow once in dry-run mode, then use its
protected apply path only when a deliberate resynchronisation is needed. Do not
automatically publish crates, release binaries, submit registry listings, start
pilots, or declare version 1.0 maturity. Those remain separately approval- and
evidence-gated tracks.

## 6. Final report

Report:

- final local and remote commit IDs;
- repository URL and Project URL;
- issue, subissue, relationship, field and item counts;
- status of every required Action;
- compiler/test/coverage/mutation/formal evidence obtained;
- exact receipts created under `.searchright/receipts/`;
- any unresolved live-provider, licensed-provider, downstream, human,
  registry, pilot or maturity gates;
- recommended next track according to `conductor/roadmap-coverage.json`.

Do not call a source-complete task mature, production-ready, externally
validated, registered or published without the corresponding observed receipt.
