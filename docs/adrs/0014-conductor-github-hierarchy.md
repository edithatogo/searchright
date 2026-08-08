# ADR 0014: Conductor-to-GitHub hierarchy

## Status

Accepted.

## Decision

Conductor is canonical. Generate one roadmap epic, one issue per track and one
native GitHub subissue per numbered plan phase. Stable issue keys and local
Markdown bodies provide idempotency and a client-neutral fallback.

Remote sync is dry-run by default. Apply requires a clean tree, `--apply`, the
`SEARCHRIGHT_GITHUB_APPLY=1` environment opt-in and GitHub issue-write authority.
The synchroniser never creates a repository, closes an issue or promotes an
evidence level.

## Consequences

The local hierarchy can be reviewed and versioned before external mutation.
Remote issue numbers and relationships remain external evidence. The hierarchy
adds 156 prepared issue artefacts for the current 31-track roadmap.
