# GitHub issue and Project hierarchy

Conductor remains the repository source of truth. This directory contains a
deterministic projection into GitHub Issues and Projects:

- one roadmap epic issue;
- one nested issue for every Conductor track;
- four nested phase subissues for every track;
- one nested task subissue for every top-level Conductor checklist item;
- one linked Project v2 containing every generated issue and manifest-owned
  custom fields/views.


The current node and relationship counts are derived from Conductor and reported
by `python scripts/check_github_issue_hierarchy.py`. The Project manifest owns its
custom fields and views. Do not use a historical count as a completion gate.

`render_github_issues.py` owns the Markdown bodies, labels and hierarchy.
`sync_github_issues.py` and `sync_github_project.py` are dry-run by default.
Remote mutation requires explicit CLI and environment opt-ins, a clean Git tree
and authenticated GitHub CLI access. Project writes require a token with
Projects permission.

## Issue state reconciliation

Task states follow the canonical checklist. A phase closes when every task in
that phase is complete. A track closes only when it is archived, its source
implementation and closeout are complete, every phase is complete, and its
explicit blocker list is empty. Active or blocked tracks remain open. The epic
stays open; completing local work does not itself establish product maturity.
Keep unfinished external obligations in the canonical plan before projecting
states. A simulated review cannot satisfy an independent or live-evidence task.

`--sync-task-state` retains its task-only behaviour. The optional
`--sync-issue-state` includes parent closure and reopening from the generated
`explicit_issue_states` policy. It uses the existing apply/state opt-ins; there
is no additional approval record:

```sh
# Preview only: no GitHub credentials or mutations.
python scripts/sync_github_issues.py --sync-issue-state --verbose-plan

# Apply the reviewed projection from a clean checkout.
SEARCHRIGHT_GITHUB_APPLY=1 SEARCHRIGHT_GITHUB_TASK_STATE_APPLY=1 \
  python scripts/sync_github_issues.py --apply --sync-issue-state

# Read-only convergence check, including parent states.
python scripts/audit_github_control_plane.py --check-issue-state
```

Without `--apply` and the existing environment opt-ins, state mutations remain
disabled. All-issue state reconciliation preserves identities, relationships and
history; it does not delete or archive issues. Repeating a converged run performs
no state writes. The audit remains task-only by default for compatibility with
existing task-only apply workflows.

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
