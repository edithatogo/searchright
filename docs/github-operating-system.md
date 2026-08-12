# GitHub operating system

Searchright uses a declarative, dry-run-first GitHub control plane rather than
manually maintained issues or Project fields. Conductor is canonical; GitHub is
a coordination projection and observed remote evidence surface.

## Canonical artefacts

- `conductor/roadmap-coverage.json`: track ownership and evidence state.
- `conductor/tracks/*/plan.md`: phases and top-level tasks.
- `conductor/github/issue-hierarchy.json`: epic, track, phase and task issues.
- `conductor/github/project.json`: Project v2 fields, views and sync policy.
- `conductor/github/repository-settings.json`: repository, security and ruleset intent.
- `CODEX_HANDOFF.md`: compiler, remote creation and Project setup contract.

The current projection is one roadmap epic, 38 tracks, 152 phases and 376
canonical top-level tasks: 567 issues/Project items and 566 native parent-child
relationships.

## Mutation controls

All scripts default to dry-run. Remote writes require an explicit `--apply`, a
matching environment opt-in, exact owner/repository matching, a clean Git tree,
and authenticated GitHub CLI access. Project mutation additionally requires a
token with GitHub Projects permission. The bootstrap verifies the authenticated
user before creating a user-owned Project.

No synchroniser deletes issues, removes or archives Project items, recreates an
incompatible field, rewrites an unrelated `origin`, or promotes evidence from
remote state.

## Convergence, checkpoints and rate limits

Issue content and labels are compared before update. Existing native subissues
are cached per parent. Project fields are compared when exposed by GitHub CLI's
dynamic JSON output. Known-equal values are skipped.

Remote calls use a bounded inter-call interval and exponential retry for
secondary rate limits and transient server failures. The controls are bounded by:

- `SEARCHRIGHT_GITHUB_MIN_INTERVAL_MS` (0–2000; default 75);
- `SEARCHRIGHT_GITHUB_MAX_RETRIES` (0–10; default 6);
- `SEARCHRIGHT_GITHUB_RETRY_CAP_SECS` (1–120; default 60).

Atomic checkpoints and receipts are written under ignored
`.searchright/receipts/`. A failed run can be safely repeated in full. Bounded
continuation is available through:

```text
sync_github_issues.py  --resume-after KEY --max-nodes N
sync_github_project.py --resume-after KEY --max-items N
```

The canonical ordering is never changed by a partial run.

## Bootstrap order

1. Verify authentication, Project access, branch and clean Git state.
2. Create or verify the remote without rewriting an unrelated `origin`.
3. Push `main` and configure repository settings, protected environments and
   the default-branch ruleset.
4. Render and validate issue bodies.
5. Create or update issues and attach native nested subissues.
6. Create/link the Project, fields and views.
7. Add every issue to the Project and synchronise manifest-owned custom fields.
8. Run `scripts/audit_github_control_plane.py` to compare observed remote state
   with all three control-plane manifests.
9. Preserve mutation and audit receipts as protected workflow artefacts; do not
   commit remote IDs into source contracts automatically.

The turnkey local command is documented in `CODEX_HANDOFF.md`.

## Workflows

`.github/workflows/github-control-plane.yml` validates the projection on changes
and on a weekly schedule. Its write job is manual, main-branch-only, protected by
the `github-project-write` environment and supplied through the
`SEARCHRIGHT_PROJECT_TOKEN` secret. Checkpoints are uploaded even when an apply
run fails.

`.github/workflows/issue-sync.yml` is a narrower issues-only path using
`GITHUB_TOKEN` with `issues:write`, protected by `github-issue-write`.

## Truth boundary

Remote closure and Project status coordinate work. They cannot upgrade a track
from source-verified to compiler-, fixture-, live-, external- or
publication-level evidence. A passing remote audit establishes GitHub parity,
not product maturity.
