# Codex execution contract — Searchright

Unpack `searchright-complete-git.zip` into a clean directory and preserve the supplied `.git` history. Read `CODEX_HANDOFF.md`, `CONTEXT.md`, and the Conductor requirements/design/tracks before changing source. The packaged head is `6782c041eead0629b55abee3681d826ee8a995fc` and the packaged working-tree-clean receipt is `True`.

This is a single accountable human/developer project. Never require a second human or an “independent human review”. Every review gate is performed by the specified isolated panel of agents: preserve sealed first passes, preserve dissent, run methodology/safety/adversarial roles, and return findings to the accountable owner. Agents are advisory; only the owner adjudicates and authorises promotion.

For every companion repository, search configured local roots for the exact clone. Refuse dirty or divergent clones. Fetch and fast-forward clean clones; clone missing repositories; create an isolated branch; import the generated portable Conductor tracks idempotently; retain source provenance, baseline ancestry and rollback receipts; run the target-native verification plan; and commit only when authored/interim gates pass. A missing Rust or other native toolchain is not permission to omit implementation or tests: keep all source and tests, use the supplied static/schema/lexical/differential/Python oracles, and mark native commands `pending_native_confirmation`. Pending native confirmation blocks release promotion; a failed gate blocks commit.

Run all GitHub and companion writes in dry-run mode first. Then create/wire `edithatogo/searchright`, push logical commits, synchronise the roadmap epic, tracks, phase subissues, task subissues and GitHub Project, and run the read-only convergence audit. Do not delete issues, tracks, receipts, target history, or planned behaviour to make checks pass.

Every pull request is limited to one declared Conductor track and is expected
to merge after its required checks pass. A multi-track exception is allowed
only for a technically inseparable change and requires the repository's
exception label plus a written split-failure rationale. Do not open omnibus or
parking PRs; enable rebase auto-merge and continue remediation on the same PR
until it merges, unless the change is explicitly superseded, unsafe or
unauthorized.

Initial native state: `pending_native_confirmation`. Report exact commits, target branches, agent-panel artefacts, test receipts, paused/failed gates and unresolved promotion blockers.

## Required sequence

1. Verify archive integrity, Git history and the expected clean head.
2. Run the complete network-free static harness and all Python reference tests.
3. Install the pinned native toolchains, generate the lockfile, execute every pending verification plan and repair source/tests without reducing scope.
4. Run companion discovery/synchronisation in dry-run mode; review the exact ancestry and branch plan; then apply one repository at a time.
5. Run each imported companion track’s target-native tests. Record confirmed, pending and failed partitions separately.
6. Create and wire the Searchright remote and Project only after dry-run convergence.
7. Re-run CI, security, compatibility, package and release-train gates.
8. Stop at the first failed gate. Paused gates permit a source commit but not promotion.

## Agent-panel protocol

Use isolated first-pass agents for methodology, implementation, testing, security/safety and adversarial review. Bind each response to the sealed request and preserve dissent. A synthesis agent may organise findings but may not silently discard them or confer authority. The accountable owner makes the final decision.

## Companion-repository safety

Exact repository identity, clean state, fast-forward ancestry and baseline provenance are mandatory. Never force-push, rewrite target history, overwrite local work or import into an ambiguous clone. Missing clones are cloned from the declared remote. Imports are idempotent and occur on isolated branches with rollback instructions.
