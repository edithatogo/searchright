# Hugging Face methodology-fixture distribution

## Purpose

Searchright prepares a small, deterministic Hugging Face dataset containing only
rights-clear synthetic methodology fixtures. The dataset supports reproducible
examples, SDK tests and downstream benchmark harnesses without distributing
sealed labels, licensed database material or external corpora.

Target dataset: `edithatogo/searchright-methodology-fixtures`.

## Architectural boundary

The GitHub repository remains the canonical source and owns:

- fixture authoring and review;
- source paths and licences;
- benchmark and leakage policy;
- deterministic package generation;
- publication authorisation;
- the relationship between source revisions and Hub revisions.

The Hugging Face repository is a release projection. It must not become the
canonical editable source, contain hidden labels, accept unreviewed files or
silently diverge from the GitHub manifest.

## Package contract

The package is built from
`registry/huggingface/searchright-methodology-fixtures/source-allowlist.json`.
Every source and destination path is explicit. The generated
`dataset-manifest.json` records SHA-256 digests, byte lengths, media types,
rights statements and a dataset root digest.

The builder rejects:

- paths outside the allowlist;
- symbolic links and traversal paths;
- sealed or hidden-label paths;
- malformed JSON or non-UTF-8 text;
- private keys, recognised provider tokens, bearer credentials and email
  addresses;
- source files larger than the declared bound;
- screening fixtures whose authority boundary is not advisory;
- methodology manifests that expose sealed labels to the repository or agents.

Repeated builds from the same source tree must be byte-identical.

## Publication control

The workflow `Publish Hugging Face methodology fixtures` is manually dispatched.
Its plan job is read-only with respect to Hugging Face and always builds,
verifies and preserves the exact package.

The publish job runs only when `apply=true` and is protected by the
`huggingface-dataset-release` environment. It requires `HF_TOKEN` and a second
runtime opt-in, verifies a clean Git worktree, permits only the exact configured
dataset identifier, and uploads the exact manifest-backed file set in one Hub
commit. Tokens are never written to receipts.

Existing remote files outside the package and Hub-managed `.gitattributes`
block an update rather than being deleted automatically. A remote revision is
claimed only after the Hub API returns and a post-commit file-set check passes.

## Evidence levels

- **prepared_not_published** — source, builder, card and protected workflow exist;
- **dry_run_passed** — exact package built and verified for a Git revision;
- **published_commit_observed** — a Hub commit and file set were observed;
- **externally_validated** — a separately governed benchmark or consumer run
  completed against an immutable Hub revision;
- **publicly_accepted** — a relevant external registry or publication accepted
  the artefact.

Publication does not establish methodological performance, search sensitivity,
model quality, state-of-the-art status or safe autonomous exclusions.

## Dependency policy

The protected publisher pins `huggingface_hub==1.30.0`, the current stable release
selected for this tranche. Renovate may propose updates, but every update must
rerun package, dry-run, API-surface and receipt-redaction tests before promotion.

## Hugging Face Jobs

A Hugging Face Job may later provide an independent clean-room build runner, but
it is not part of the required release path. Job execution requires available
account credits and an immutable source checkout. A failed or unavailable Job is
reported as unavailable, never converted into package or compiler evidence.
