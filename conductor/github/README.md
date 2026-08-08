# GitHub issue hierarchy

Conductor remains the repository source of truth. This directory contains a
deterministic projection into GitHub Issues:

- one roadmap epic issue;
- one child issue for every Conductor track;
- four child subissues for every numbered track phase.

`render_github_issues.py` owns the Markdown bodies and
`issue-hierarchy.json`. `sync_github_issues.py` is dry-run by default and needs
both `--apply` and `SEARCHRIGHT_GITHUB_APPLY=1` before it can mutate GitHub.
Remote issue numbers, subissue relationships and timestamps are external
evidence and are never invented locally.

Native GitHub subissues are the primary remote hierarchy. Stable markers and
this manifest are retained as a portable fallback for clients that do not yet
render nested issues.
