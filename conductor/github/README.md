# GitHub issue and Project hierarchy

Conductor remains the repository source of truth. This directory contains a
deterministic projection into GitHub Issues and Projects:

- one roadmap epic issue;
- one nested issue for every Conductor track;
- four nested phase subissues for every track;
- one nested task subissue for every top-level Conductor checklist item;
- one linked Project v2 containing every generated issue and manifest-owned
  custom fields/views.


The canonical projection currently contains **568 issues/Project items**: one
roadmap epic, 38 track issues, 152 phase subissues and 377 task subissues, joined
by 567 native parent-child relationships. The Project manifest owns 13 custom
fields and six views. These counts are generated and validated; changing them
requires changing Conductor first.

`render_github_issues.py` owns the Markdown bodies, labels and hierarchy.
`sync_github_issues.py` and `sync_github_project.py` are dry-run by default.
Remote mutation requires explicit CLI and environment opt-ins, a clean Git tree
and authenticated GitHub CLI access. Project writes require a token with
Projects permission.

`bootstrap_github.py` is the end-to-end controller used after cloning the final
repository: it creates or verifies the remote, pushes `main`, applies repository
settings and branch rules, creates protected environments, synchronises issues,
and creates/populates the Project.

Remote issue numbers, relationships, Project IDs and timestamps are external
evidence and are never invented or automatically committed. Synchronisers are
convergent, bounded and resumable; atomic checkpoints and receipts live in the
ignored `.searchright/receipts/` directory. No synchroniser deletes or archives
remote work. Closing an issue never promotes the evidence level recorded by
Conductor. A read-only post-apply audit must verify settings, issues, native
subissues, Project fields, views and items before the control plane is described
as converged.
