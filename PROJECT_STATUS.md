# Project status

**Assessment date: 9 November 2026. Searchright is a technical alpha with
executed compiler and component-fixture evidence. Its live review workflow,
independent performance evidence and public package release remain unfinished.**

The dated machine-readable observation is
`verification/receipts/project-status-snapshot.json`. It records the initially
observed `main` revision `dc3eabe66b99e26cf6320297b258efb060e68d19` and the
last fully admitted regular-suite revision
`f502104a81c43ed0117886ce8cb21f00fdeb7588`. These are historical observations,
not a live dashboard or automatic admission of later commits.

## What is implemented and tested

The 30-crate Rust workspace contains the shared query/provider kernel, review
facade, CLI, local MCP server, import/deduplication/linkage/screening components,
reporting, living-update and provenance foundations. All 200 acceptance
assertions have individual implementation/test/evidence mappings. Compiler and
runtime results must still be attributed to the revision and scope actually run.

At `f502104a81c43ed0117886ce8cb21f00fdeb7588`, 18 hosted jobs passed across
seven workflows; two inapplicable jobs were skipped. The passing suite includes
Ubuntu, macOS and Windows compilation/tests, the declared Rust version, static
contracts, coverage admission, security policy, clean-room build/install,
API/SemVer checks and bounded Kani/Miri/Loom/precondition suites. The exact run
URLs are in the snapshot. Coverage was **88.28%**, below the greater-than-90%
maturity target.

Recent substantive improvements include the maintained `csv` dependency and
multiline screening fixes ([PR 755](https://github.com/edithatogo/searchright/pull/755)),
pinned CI tool reuse ([PR 757](https://github.com/edithatogo/searchright/pull/757)),
and benchmark population/digest integrity
([PR 758](https://github.com/edithatogo/searchright/pull/758)). Later scenario
tests and deterministic provenance ordering are useful component improvements;
simulated panels do not establish external acceptance.

## What the reconciliation corrected

The initially observed source marked all 38 tracks archived and all 396 tasks
complete. That state was inconsistent with the implementation, maturity dossier,
release packets and execution results. Original live, human, downstream and
publication requirements had been replaced by simulations or checked off without
the corresponding outcome.

The canonical correction restores those original requirements and GitHub task
identities, retains authentic component completions, and reactivates tracks with
unfinished work. Historical receipts and source/tests are preserved. The seven
bounded completed tracks remain archived. Read the current totals directly with:

```bash
python scripts/conductor_status.py
```

`conductor/roadmap-coverage.json` and the generated track plans own current
status. The snapshot's 396/396 count describes the rejected pre-reconciliation
observation, not the corrected state or a product completion score. GitHub task
states follow those plans; phases close when their child tasks are complete, and
tracks close only when their canonical archival and closeout conditions hold.

## Build and dependency findings

At the initially observed `dc3eabe` revision, CI, coverage, clean-room, security
and formal workflows failed. The vendored `noyalib` requirements had diverged
from the committed lockfile. [PR 796](https://github.com/edithatogo/searchright/pull/796)
repairs that mismatch, checks the locked graph before security tools run, and
creates the output parent for the scheduled mutation job. Follow its exact
workflow results for repair admission; the change itself is not a passing
mutation result.

The subsequent concurrent revision `2e9ca30b26b3f3cbf83e7d3b9f7c5a7d57702c67`
introduced `schemars` and `schemars_derive` 1.2.3 without qualifying Cargo Vet
paths. Its [supply-chain job failed](https://github.com/edithatogo/searchright/actions/runs/42591410808/job/129073738912).
This reconciliation restores the admitted 1.2.2 versions and published
checksums, preserving the other dependency, Renovate and build repairs. It does
not retarget or extend any exemption. PR 796's completed native suite passed;
its [coverage result was 88.29%](https://github.com/edithatogo/searchright/actions/runs/42590358525/job/129070202860).
Admission of the combined reconciliation requires its own hosted results.

The 8 November peer-audit promotion legitimately retired twelve exemptions.
**235 active entries remain: 218 `safe-to-deploy` and 17 `safe-to-run`, expiring
10 November 2026.** Their expiry is unchanged. The last observed mutation job
failed during setup before running mutants. See the
[dependency remediation sequence](docs/security/cargo-vet-governance.md#deadline-remediation-sequence)
and [Track 16](https://github.com/edithatogo/searchright/issues/233).

## Product work still to deliver

| Outcome | Present boundary | Existing owner |
| --- | --- | --- |
| One complete two-source review | The public search facade is fixture-only; CLI live apply is rejected. Live transport and authority wiring, resume and actual provider results remain open. | Tracks 03/04 |
| A reproduced external baseline | Benchmark identity checks work; a pinned SYNERGY/ASReview experiment with per-review results has not been delivered. | Tracks 17/23 |
| An independently usable evidence package | Local packaging/provenance code exists; required RO-Crate metadata, PROV relationships and independent consumer validation remain open. | Track 25 |
| Autonomous, solo and team operation | Policy concepts and protected screening components exist; explicit modes, machine decision identity and full transition/conflict workflows remain open. | Tracks 07/11/28 |
| A deliverable research programme | The study/dataset/journal plan is recorded; preregistration, independent evaluation and prospective observations have not occurred. | Track 29 |
| Distribution and adoption | Packages remain non-publishable; release rehearsal is prepared, maturity is `not_ready`, and signing/publication/downstream acceptance remain open. | Tracks 18/19/32/35–37 |

The [frontier plan](conductor/frontier-plan.md) supplies the ordered work units
and coding-agent estimates. A useful local product does not need to wait for
hosted tenancy, every estate migration or journal acceptance. Those obligations
remain open only for the capabilities and claims that require them.

## GitHub and evidence limits

Environment disclosures:

- **Rust compilation:** historical hosted execution is evidenced above; this assessment runtime has no Rust compiler.
- **Live provider calls:** no completed authorised live review run is claimed.
- **GitHub repository creation/push:** the existing repository and historical bootstrap are observed; current remote state requires a fresh read.
- **Conductor plugin installation:** no current host installation was tested in this assessment runtime.

The repository, remote issue hierarchy and delivery Project exist. Historical
bootstrap receipts remain valid for their observed revisions. The current
Project v2 fields, views and item parity were not verified by this assessment's
available API; an issue reconciliation must not be called a Project audit.

Current branch, PR, CI and remote issue states must be read from GitHub or a dated
receipt. This document does not assert a clean worktree, branch equality, no open
PRs or a completed later run. It also does not claim live-provider support,
methodological superiority, authenticated hosting, independent package
conformance, completed migration, public release or version 1.0 readiness.
