# PR #679 branch-rewrite note

The existing PR branch contains only a temporary payload-applicator workflow. The final repair intentionally rebuilds the branch from the latest `main` and adds the reviewed demonstration directly. The update uses `--force-with-lease` against the observed old PR head so concurrent changes cannot be overwritten silently.

The demonstration is owned by existing Conductor Track 36. It does not expand the repository's Track ID range or weaken the one-track PR policy.
