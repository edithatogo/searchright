# Workflow

## Active delivery programme

Use [frontier-plan.md](frontier-plan.md) for current sequencing, dependency reuse,
three operating modes and the coding-agent timetable. Existing track IDs locate
work; do not create a second hierarchy of frontier tracks or mirrored status
documents. The legacy launch roadmap covers broader deployment/research scope
than the initial local release.

Routine work uses a short outcome, implementation, relevant tests and ordinary
review. CI records verification. Specialist assessment belongs where it answers
a concrete methodological question; no standing multi-agent panel is needed.
Research comparisons establish the associated claims, not permission to code.

## Spec-driven sequence

1. Read product context, requirements, architecture, ADRs and active track.
2. Locate the existing requirement/track when useful; record a material design
   choice once in the plan or PR.
3. Amend contracts and examples when the public behaviour changes.
4. Add regression tests for meaningful failures; do not test paperwork presence.
5. Implement the smallest inward-facing domain change.
6. Add adapter, CLI and MCP surfaces by delegation to the facade.
7. Run affected checks; use configured opt-in live tests within authorised scope.
8. Link observed results from the owning plan; update migration notes only when
   persisted data or interfaces change. Avoid duplicate evidence ledgers.
9. Review against requirements, threat model and public documentation.
10. Commit one coherent delivery unit with a primary track/task identifier and
    explicitly declared additional affected tracks where needed.

Use [proportionate governance](../docs/quality/proportionate-governance.md) for
bounded approval and verification. Implement the programme's CI simplification
by changing obsolete checks and workflows together; never silently bypass a
required hosted check. Product authority and publication permissions still apply.

## Testing order

- contract/schema examples;
- unit and property tests;
- integration tests across crate boundaries;
- CLI snapshot and MCP transcript tests;
- provider fixture/replay tests;
- deterministic simulation and metamorphic tests;
- fuzz and mutation tests;
- opt-in live smoke tests;
- release/install/end-to-end tests.

## Completion rule

A task is complete only when code, tests, documentation, migration impact,
security impact and evidence-level wording are updated. A scaffold is never
marked as runtime-proven.

## Branch and commit convention

- Branch: `track/<track-id>-<short-name>`.
- Commit: `<type>(<track-id>): <imperative summary>`.
- Keep generated artefacts reproducible and commit the lockfile for binaries.
- No mandatory human approval is encoded for a solo-maintainer repository, but
  CI and explicit release environments protect irreversible operations.

## Native status protocol

For routine status, run `python3 -B scripts/conductor_status.py`. Run relevant
generated-evidence checks for reconciliation; run the full `scripts/verify.sh`
for candidate admission and releases. Report status and verification separately. The status command
is read-only and reconciles the actual table registry, top-level plan tasks,
metadata, evidence, roadmap and in-place archive lifecycle. It never executes
code from the inspected checkout, including when `--root` names another tree.
Run the native roadmap and generated-evidence checks separately after trusting
the checkout; the status result does not claim those checks ran. Its tests are
`python3 -B -m unittest discover -s tests -p test_conductor_status.py`.

Searchright uses `conductor.track-metadata.v3`: numeric stable track IDs,
evidence-aware status, separate lifecycle and canonical paths retained under
`conductor/tracks/`. Generic local skill validators requiring full-directory
IDs, `new/in_progress/completed` statuses and physical archive relocation are
not schema-compatible. Report this compatibility limitation separately; do not
rewrite canonical IDs, evidence levels or paths to satisfy that validator.
No generic skill installation or upstream-version migration is implied.

Ledger opt-in or configured worktree isolation requires schema-specific ledger
or exact lease validation. Until configured, report them as unconfigured and
never infer ownership from a worktree directory. Repository integrity does not
complete external gates or establish compiler, hosted, adoption or publication
evidence. Status output is derived and never canonical authority.

## Conductor upstream baseline

The repository targets Conductor 0.3.0 and its current Context → Spec & Plan →
Implement lifecycle. Use adaptive UX, smart logical reversion and review-fix
flows where the host exposes them. No undocumented experimental configuration
flag is invented; newer upstream capabilities require an observed version bump
and an update to `conductor/upstream.lock.json`.
