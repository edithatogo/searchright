# Searchright: agent-led frontier development

Updated 7 November 2026. This is the forward development plan, coordinated through
existing Track 30. It adds no new track hierarchy. Implementation status remains
in the relevant PRs and observed test results; a planned capability is not a
working or validated capability.

## Goal and priority

Deliver a usable research tool that can find, assess, update and release evidence
with less work and fewer consequential errors. Establish a measured advantage
first in search translation, then in living updates and total reviewer effort.
Use mature dependencies for commodity functions and established methods.

The initial product is a local CLI/MCP workflow. Hosted tenancy, portfolio-wide
migration, registry acceptance and publication do not belong on its critical path.
Research comparison is required for the corresponding superiority claim, not for
every development release. The wider legacy launch roadmap remains historical
and integration planning context; it is not a mandatory serial sequence for this
local programme.

Supporting plans:

- [Research studies, datasets and journals](../docs/research/programme.md).
- [Versioned evidence packages and repository outputs](../docs/research/evidence-packages.md).
- [Existing GitHub and Hugging Face reuse](../docs/research/portfolio-reuse.md).
- [Optional local-model comparisons](../docs/research/local-model-comparison-20261108.md) — small, task-scoped tests within existing owners; no extra track hierarchy.

## Four milestones

| Milestone | Deliverable and useful test | Existing owners |
| --- | --- | --- |
| 1. Make it work | Two live sources through CLI/MCP; interruption/resume; conservative deduplication; recorded decisions; truthful export and replay | 03, 04, 06, 07, 09, 10 |
| 2. Find an advantage | Small development comparisons for difficult translations and screening; retain the strongest simple baseline | 02, 17, 23 |
| 3. Demonstrate it | Frozen scoped comparison; usable autonomous/solo/team modes; independently validated evidence package | 11, 17, 25, 28, 29 |
| 4. Expand on results | Selective living updates, broader validation, prospective use and citable releases | 22, 25, 29, 35, 36 |

Archived tracks remain useful ownership references. Follow-up work does not
reopen their historical acceptance or require a new phase/subissue tree. Track 38
is already used in [draft PR 679](https://github.com/edithatogo/searchright/pull/679);
the institutional monitor can become an operational pilot, but is not a recall
validation study.

## Immediate execution queue

This is the next work sequence within the existing tracks, not another release
checklist or issue hierarchy. Link the implementation PR and its observed results
to the relevant item; keep the supporting research and package specifications in
their existing documents. Do not restart completed work.

- [x] Remove artificial gate counts and mandatory readiness wording: merged
  [PR 754](https://github.com/edithatogo/searchright/pull/754).
- [x] Replace handwritten bibliographic/screening CSV with `csv` 1.4.0: merged
  [PR 755](https://github.com/edithatogo/searchright/pull/755), including multiline
  screening fixes, sixteen regressions and exact-version source reviews. See the
  [dependency decision](../docs/csv-dependency-decision.md).
- [ ] Reuse pinned security-tool installations in CI (Track 16, in progress).
  Preserve every substantive security command and required check. Cache only
  dedicated tool binaries, with an exact environment/tool/workflow key; rebuild
  on a miss and verify versions on both paths. Only successful main runs populate
  the cache. Check cold installation and warm reuse in hosted logs; measure any
  timing improvement from those runs rather than predict a percentage.
- [ ] Repair benchmark population and label-digest binding (Track 17, in
  progress). Reject duplicate, extra or missing label/prediction identities;
  define complete predictions or explicit abstention accounting; verify the
  digest against a documented canonical representation. A different population
  or label set must not retain the original evaluation identity. This precedes
  any new scored comparison.

| Next work unit and existing owners | Prerequisites | Observable completion | Coding-agent effort |
| --- | --- | --- | --- |
| Resolve the dependency-review deadline — 16 | Existing lockfile, approved audit sources and exact-version acceptance records | Follow the [bounded remediation sequence](../docs/security/cargo-vet-governance.md#deadline-remediation-sequence). Retire only genuinely unused audit roots, replace exemptions with qualifying audits, and show the exact residual set before 10 November. No automatic extension or certification from a passing build. | Initial triage 0.5–1 day; source-review effort depends on the remaining code and cannot honestly be fixed at 247 versions in three days |
| Deliver the first bounded two-source review — 03/04, with 05/06/07/09/10 | Shared transport controls; explicit local principal, policy, budget and provider access | CLI and MCP execute the same protocol through the shared live runtime; persist observations, records and decisions; interrupt/resume without duplicate effects or lost decisions; conserve counts through deduplication/export. Cover pagination, retry/429, cancellation, partial failure and budget exhaustion. Record separate authorised redacted live results for the selected pair. | 3–7 days; re-estimate after the first complete run |
| Reproduce one existing-data baseline — 17, with 23/29 | Benchmark identity repair and a rights-clear pinned input | Use the existing SYNERGY upstream/fork route first; group splits by review/topic family and check shared publications. Reproduce random, lexical and pinned ASReview baselines through a small optional worker. Save per-review predictions, failures, seeds, costs and analysis. The worker receives observed training labels only. | 2–4 days for the first narrow experiment; informs [R3](../docs/research/programme.md#r3-can-ranking-reduce-screening-workload-without-concealing-missed-studies) |
| Export one independently consumable evidence package — 25 | A small complete review output, which may initially use fixtures, and declared package metadata | Fix RO-Crate date/licence metadata, byte-size versus record-count semantics and PROV entity/activity relations. An independent consumer resolves identifiers, inspects decisions and detects altered/missing assets. Demonstrate deterministic replay where supported and disclose inspection-only/restricted inputs. | 2–4 days, parallel with the baseline; follows the [package specification](../docs/research/evidence-packages.md) |
| Implement autonomous, solo and team policies — 07/11/28, with 10 | Bounded execution and durable decisions | Preserve actor, mode and policy version; keep machine decisions identifiable; link human confirmation to original decisions; preserve real team independence/conflicts/adjudication. Test mode transitions, interrupted work and contested writes. No fictional human reviewers. | 4–7 days; supplies the workflow for [R4](../docs/research/programme.md#r4-do-agents-solo-reviewers-and-teams-produce-reproducible-review-workflows) |
| Run the first frontier experiment — 02/17/22/29 | Working slice, valid evaluator and reproduced baseline | Select the R1 translation or R3 ranking endpoint from development results, freeze its comparison before confirmation, and retain failures and no-gain results. Prototype selective updates in parallel and compare deterministic incremental results with complete replay. | 1–3 weeks after the prerequisites, plus the explicitly budgeted specialist time |

The current search facade is fixture-only, the CLI rejects live apply, and the
default MCP verifier denies consequential effects. The two-source task therefore
includes live-runtime and authority wiring; removing a denial or recording a
smoke test alone is insufficient. Select the first pair from providers whose
transport and access conditions can actually be met; PubMed and Europe PMC are
candidates, not new live-support claims.

The baseline and local package can proceed in parallel. Neither waits for a
hosted service, all companion migrations, a manuscript, a new public repository
or all historical Track 30 maturity gates. After local export works, add the
optional GitHub release/repository destination and pinned Hugging Face dataset
references through the [existing estate](../docs/research/portfolio-reuse.md).
Publishing remains within the configured destination/content authority.

## Lean delivery rules

One useful change, one PR, relevant tests and ordinary review. CI records results.
An experiment records its inputs, versions and outputs automatically. Routine
work does not need duplicated receipts, five role panels or updates to several
status documents. Existing user authority covers ordinary implementation and
continuation; ask again only when an action materially exceeds that authority.

Apply process reduction in small coherent changes, alongside product work:

1. Remove gate-count requirements and mandatory readiness prose. Check real
   conditions and supported capabilities rather than a predetermined status.
2. Replace duplicated maturity/gap/status maintenance with one active backlog and
   generated capability/results view. Preserve useful historical results.
3. Stop creating phase/task issue mirrors for each new plan item. Conductor owns
   the design and milestone plan; GitHub issues/PRs hold actionable work.
4. Make CI depend on changed behaviour. Run parser, authority, persistence and
   interface tests where relevant; expensive fuzz/formal/mutation suites on
   affected changes or schedules. Update workflows and required checks together
   through authorised repository operations; never bypass a failing check.
5. Replace universal maturity prerequisites with release profiles: local package,
   enabled integration, or hosted service. A paper or registry listing is not a
   prerequisite for distributing a working local package.
6. Shorten agent instructions to architecture, task context, relevant commands
   and actual authority boundaries. Use one coordinating agent; add specialists
   only where their contribution changes the outcome.

Keep original source records, review amendments, machine/human decision identity,
credentials protection and meaningful recovery tests. These are product
requirements. Assess the cleanup using existing Git/CI history: useful changes
delivered, process-only edits, waiting time, repeated prompts and escaped defects.
Do not create a new reporting subsystem to measure the cleanup.

The next process changes each have a concrete completion condition:

| Follow-on change | Completion condition |
| --- | --- |
| Change-sensitive CI | Stable required jobs report applicability explicitly; unclassified changes take the broader path; parser, authority, persistence, dependency and workflow changes still run their relevant checks. Scheduled expensive checks continue. Validate changed-path cases before altering branch requirements. |
| Generated capability/status view | Select existing Conductor inputs once, generate the public view, and remove duplicate manual update obligations. Preserve historical evidence and show unavailable evidence honestly. |
| Coarser future issue projection | Forward-plan work creates actionable track/issues and PRs without automatic phase/task mirrors. Existing remote identities, history and additive/idempotent synchronisation remain intact. |
| Release profiles | A local package, an enabled integration and a hosted deployment each run checks for the capabilities they actually expose. Unsupported capabilities remain unavailable; papers and registry listings do not become local-package prerequisites. |

Start with tool caching before changing CI applicability. Use one implementation
PR per coherent change, and retire a process only when its useful responsibility
has an identified owner. These rows are planned work, not completed migration.

## Coding-agent delivery estimate

Assumptions: one coordinating coding agent, up to two parallel implementation or
test agents in isolated worktrees, working CI, and the owner available for short
decisions. These estimates concern a scoped local product, not all historical
roadmap features. Remaining estimates start with the next implementation
increment; already merged work above is not included again. They are not deadlines.

| Window, with overlapping work | Agent deliverables | Human/external elapsed time |
| --- | --- | --- |
| Days 1–3 | Resolve the immediate dependency-review disposition; reuse CI tools; repair benchmark identity checks; begin bounded live-runtime wiring | Decide any concrete residual dependency risk; confirm unavailable provider access and initial review topics |
| Days 3–10 | Two-source run/resume/export; reproduced existing-data/ASReview baseline; difficult native-query corpus; independently inspected small package | A specialist checks the first query pairs; allow approximately 4–8 hours initially |
| Weeks 2–3 | Autonomous/solo/team policy and decision provenance; package/repository exporter; regression and adversarial tasks | Owner tests the full workflow; specialist resolves material semantic disagreements |
| Weeks 3–6 | Candidate translation improvements, selective-update prototype, one scoped frozen comparison and installable release | Approximately 10–30 specialist hours for feasibility/annotation is a planning allowance; revise from the pilot |
| Weeks 6–10 | Follow-up fixes, independent rerun, research analysis and a first methods manuscript if results support it | Recruitment, blinded assessment, authorship and journal preparation determine the calendar |
| Weeks 8–16+ | Prospective update cycles and broader mode/effort evaluation; additional method only when it wins | Real updates and prospective observations cannot be accelerated merely by adding agents |

The engineering target is a useful increment within the first week and a scoped
local candidate in roughly 3–6 weeks. Re-estimate after the first complete live
run. More agents do not linearly shorten integration, CI, adjudication or data
access. Journal review is outside the delivery estimate. An unsuccessful
experiment should narrow the next increment, not trigger more framework building.

## Dependency choices and replacement order

Keep Rust ownership of state and contracts. Reuse specialised implementations
through small adapters; do not rewrite the product around a research dependency.

| Custom code or scaffold | Reuse decision | Replacement test and boundary |
| --- | --- | --- |
| Handwritten CSV parsing/writing | Delivered with `csv` 1.4.0 in [PR 755](https://github.com/edithatogo/searchright/pull/755) | Multiline fields, quotes, CRLF/CR, blank lines, variable rows and large fields covered; intended output bytes preserved. The new error variant and upstream permissive quoting are documented in the [dependency decision](../docs/csv-dependency-decision.md) |
| Token-overlap-only prioritisation | Add an optional local ASReview worker; retain the simple ranker as baseline | Topic-held-out effort/recall comparison, deterministic seeds where supported, model/configuration provenance; worker cannot impersonate human decisions |
| Repeated screening simulations | Reuse ASReview Makita where compatible | Pin the ASReview/Makita pair; verify baseline reproduction before adding parallel simulations |
| Handcrafted MCP/schema machinery | Retain official `rmcp` already used; evaluate a maintained JSON Schema implementation for remaining manual validator code | Same canonical schemas, local resolution only, hostile schemas/resource limits, output/error parity; remove custom code after comparison |
| Provider transport and XML | Retain existing `reqwest`/rustls, `url` and `quick-xml` | No replacement merely for novelty; remove duplicate wrappers only when budgets, retries, redaction and egress behaviour survive |
| RO-Crate/PROV conformance | Validate with independent RO-Crate tooling; use Python `rocrate`/PROV tooling in packaging or tests where suitable | Fix missing required metadata and invalid entity/activity relationships; a self-written JSON shape check is insufficient |
| Dataset acquisition and versioning | Reuse `huggingface_hub`, Parquet readers and the user's existing archive/publishing pipelines | Pin dataset commits and files; preserve source rights and actual-capture states; do not create another downloader estate |
| Large-corpus inspection | Optional DuckDB over Arrow/Parquet before a new database service | Measure memory/query behaviour on actual corpora; keep the small local path simple |
| Statistical stopping | Compare published methods and BuscarPy; use DESTINY evaluation code under its own licence | Separate method assumptions, remaining-pool audit and realised recall from ranking; avoid importing AGPL implementation into permissively distributed code without resolving obligations |

For each new dependency record the exact chosen version, upstream, licence,
relevant maintenance evidence and the custom code it removes in the PR. Use the
lockfile and existing automated dependency checks. No extra certification ledger
is needed. A dependency is adopted when it improves tested behaviour or removes
meaningful maintenance, not because it has a recent release number.

Primary references: [Rust CSV](https://github.com/BurntSushi/rust-csv),
[ASReview](https://github.com/asreview/asreview),
[Makita](https://github.com/asreview/asreview-makita),
[official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk),
[RO-Crate Python](https://github.com/ResearchObject/ro-crate-py),
[Hugging Face Hub](https://huggingface.co/docs/huggingface_hub),
[DuckDB](https://duckdb.org/docs/stable/),
[BuscarPy](https://github.com/mcallaghan/buscarpy),
[DESTINY stopping evaluation](https://github.com/destiny-evidence/stopping-methods).
Recheck exact versions at implementation; a candidate listed here is not installed
or demonstrated merely because this plan names it.

## Operating modes

Mode controls workflow and authority, not the quality label of every output.
Preserve a mode/version on each run and an actor/evidence status on each decision.

| Mode | What can run | Human contribution | Required output distinction |
| --- | --- | --- | --- |
| Autonomous | Question decomposition, search/translation variants within scope, retrieval, deduplication, machine screening, stopping advice, extraction, synthesis draft, package creation and scheduled updates | Initial policy/budget/source/deposit authorisation; no routine per-record approval | Machine-assessed decisions and draft conclusions; never described as independently human-reviewed |
| Solo | Same automation, with configurable sampling, uncertainty/conflict queues and selected decision checkpoints | One reviewer can accept, amend, reject or take over; no fictitious second reviewer | Human-confirmed, machine-only and unresolved decisions remain distinguishable |
| Team | Assignments, independent/blinded decisions when requested, conflicts, adjudication, role-specific access and parallel work | Two or more actual people according to the chosen protocol; one-person tasks still supported | Real reviewer identities, independence/blinding status and adjudication history |

All three are supported design targets. The current human-protected runtime does
not yet implement unrestricted autonomous final decisions. Implement autonomous
machine decisions in an explicit namespace/policy and test them before enabling
that mode; do not simply remove existing authority checks. A multi-agent ensemble
does not count as two independent human reviewers.

Allow transitions between modes without rewriting history. A team may review an
autonomous run, or a solo reviewer may delegate an update. Link later confirmations
to the original machine decisions. Apply a lease or optimistic-concurrency check
to contested writes; make conflicts visible and reversible. Default permissions
are narrow enough for the selected workspace, with no repeated prompt for
ordinary authorised continuation. External publication can be automatic only
when the destination, scope and content policy were explicitly preauthorised.

## Experiments that can create a lead

### Search meaning and differential testing

Keep native syntax alongside a portable representation. Record whether each
rewrite preserves, broadens, narrows or leaves meaning unknown, with a source
span and rule version. Start with real hard queries and controlled documents for
NOT, proximity, field scope, phrases and filters. Compare against independently
prepared strategies and Polyglot where supported. Use a small independent
interpreter only for a demonstrated coverage gap. Never use cross-database result
equality as proof of equivalence.

### Retrieval and review effort

Union supported Boolean, semantic, citation and register routes, retain route
provenance and disclose any cutoff. Rank candidates without quietly discarding
the tail. Compare total human minutes, including calibration, correction and
audit, at declared achieved recall. Evaluate stopping separately from ranking.
Observed human labels are valid active-learning inputs; unobserved holdout answers
must remain outside candidate prompts, caches and model-worker inputs.

### Selective living updates

Separate observation digests from material content and field-level deltas. Link
source observations, report/study identity, criterion evidence, decisions and
outputs. Reconsider only affected decisions, with an inspectable reason. Compare
incremental deterministic outputs with a complete replay. Test new reports,
corrections, late indexing, missing records, protocol amendments and changes that
only affect provenance. Paid change feeds and periodic rescans have different
coverage; retain explicit reconciliation limits.

### Beyond the first candidate

Prioritise these only after an end-to-end workflow and baseline exist:

- **Provider drift diagnosis:** sentinel queries and replayable responses reveal
  changes in API behaviour, indexing or translation; distinguish these causes
  before changing a strategy.
- **Cost-aware retrieval:** measure marginal relevant yield and reviewer burden
  by source, then experiment with budget allocation. Preserve the complete
  protocol route unless the user chooses an explicitly exploratory policy.
- **Review forks and comparisons:** branch eligibility or methods, reuse unchanged
  evidence and explain exactly why conclusions or counts differ.
- **Evidence-to-claim links:** attach extracted values and synthesis claims to
  source spans and decision versions through CiteWeft/Sourceright; missing support
  remains visible. This is an extension after reliable search and screening.
- **Model challengers:** evaluate new models offline against fixed development
  tasks, cost and failure cases; promote only measured improvements. No unattended
  self-modification of production methodology.
- **Reproducibility service:** consume a released review package in a clean
  environment and produce a discrepancy report. Validate this with an independent
  consumer, not only Searchright reading its own output.

## Evaluation and stopping investment

Start with approximately six development topics and a targeted set of difficult
query/import/authority cases. Choose the promising endpoint before confirmation.
Provisional ambitions are 25% less translation correction time, 20% less total
screening effort at protected recall, and 50% less update-review effort in defined
scenarios. These are hypotheses, not routine release gates or achieved results.

Use independent topics as the unit of generalisation. Repeated seeds do not create
new independent reviews. Public screening labels do not establish live-search
completeness or freedom from model pretraining contamination. For a broad claim,
the initial 24-topic/three-pilot allowance must be replaced by a design supported
by pilot variability and the desired precision. A scoped result can use a smaller
appropriate comparison and a narrower statement.

Keep failed runs, abstentions, unavailable sources and no-stop runs in the results.
Retain simple methods when challengers offer no useful gain. Reuse old holdouts for
regression tests; use fresh confirmation data or a declared repeated-test design
for new superiority claims after tuning. Release useful software while these
research questions remain open.

## Completion of this planning change

This plan records intended work and sources. It does not claim live support,
autonomous review validity, external evaluation, published datasets, package
conformance, journal acceptance or adoption of every proposed dependency.
Subsequent PRs should link the useful outcome and observed results here rather
than create a second status hierarchy.
