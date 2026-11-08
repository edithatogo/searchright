# Research programme

Status: proposed studies; no results or journal acceptance claimed. Primary
sources checked on 7 November 2026. Conductor remains the execution plan.

The programme should produce useful software, reproducible evidence and a small
number of distinct publications. Start with existing labelled data and bounded
experiments. Add human participation when the research question requires human
judgement, observed working time or external validation. Neither an independent
panel nor journal submission is a prerequisite for ordinary development.

## Questions and four studies

### R1. Can explicit translation-loss reporting prevent silent search errors?

**Tool deliverable:** a supported query subset, executable dialect conformance
corpus, stable loss diagnostics and replayable search receipts in the shared
kernel. Preserve the original query and every unsupported construct.

**Design:** paired evaluation of frozen strategies from CLEF TAR and the seed
collection. Separate development topics from evaluation topics. Compare native
queries, Searchright translations and a pinned accessible comparator such as
[Polyglot](https://polyglot.sr-accelerator.com/). Evaluate Boolean semantics on a
controlled corpus; deliberately vary nesting, negation, fields, truncation and
unsupported operators. These generated cases establish correctness against the
specified model, not equivalence of live databases. An optional information
specialist annotates real translated strategies; a second specialist is needed
only for claims about independently adjudicated expert agreement.

**Main outcomes:** silent semantic failures, detection of injected errors,
retained known-included records and unsupported-query coverage. Report refusal
and warning rates alongside fidelity, so abstention cannot inflate success.
Live result counts alone cannot establish translation equivalence: the original
[Polyglot trial](https://jmla.mlanet.org/ojs/jmla/article/view/834) also found
substantial variability in retrieval counts.

**Publication:** JMLA Original Investigation for externally assessed translation
performance; Research Synthesis Methods (RSM) Software Focus for a substantive
computational contribution with reproducible conformance evidence. Select the
article according to the evidence actually obtained.

**Dependencies and time:** import/query adapters and a frozen evaluation runner.
Estimate 3–7 agent working days for a narrow executable experiment, then 1–3
days for analysis and a manuscript draft. Optional specialist assessment adds
approximately 8–20 person-hours and 1–3 elapsed weeks, subject to availability.

### R2. How much relevant evidence does conservative deduplication preserve?

**Tool deliverable:** reproducible duplicate clusters, reversible decisions,
retained source records and a diagnostic report explaining each proposed merge.

**Design:** evaluate the corrected ASySD benchmark datasets without tuning on
their final labels. Compare exact-identifier matching, Searchright and a pinned
ASySD version. Add separately reported perturbations for absent identifiers,
Unicode, conflicting metadata and input order. A limited development subset
must be identified before any changes informed by benchmark errors; later runs
on exposed labels become development results.

**Main outcomes:** distinct publications incorrectly removed, duplicate recall,
cluster errors, runtime and memory. Report counts and rates per dataset rather
than relying on specificity across the enormous number of nonduplicate pairs.
Inspect difficult cases against the existing reference labels; unreviewed
disagreements remain disagreements. These data evaluate duplicate citations,
not whether different reports belong to one underlying study.

**Publication:** Systematic Reviews methods research or JMLA Original
Investigation. RSM becomes a stronger target if the work introduces a
generalisable method, rather than only comparing another implementation.

**Dependencies and time:** reliable imports, duplicate-cluster export and an
ASySD adapter. Estimate 2–5 agent days plus 1–2 days for analysis/drafting. No new
human labelling is required for comparison with published labels; optional
error adjudication adds 4–12 person-hours. Confirm downloadable file licences
before redistributing the benchmark.

### R3. Can ranking reduce screening workload without concealing missed studies?

**Tool deliverable:** benchmarkable ranking and calibration, declared stopping
rules, review-level diagnostics and complete records of model/version/cost.

**Design:** retrospective simulations on SYNERGY and nonoverlapping CLEF TAR
topics. Compare random order, a reproducible lexical baseline, an accessible
active-learning baseline and the proposed method. Split by review, check
shared publications and topic families, freeze tuning before testing, and use
repeated random seeds. Existing labels are an oracle for simulation; they are
not new human observations. Add CSMeD-FT only as a separately analysed full-text
extension after the extraction boundary is executable.

**Main outcomes:** missed known-included records at predeclared budgets and
operational stopping points; reviews failing the recall target; screened-record
counts; runtime and cost. Report workload at oracle 95%/99% recall as a
descriptive metric, separately from achieved recall when a real stopping rule
has no access to remaining labels. Use review-level uncertainty and show
worst-performing reviews. Record counts do not establish human time saved.

**Publication:** RSM Research Article; Systematic Reviews as an alternative for
an applied biomedical methods study.

**Dependencies and time:** sealed-label runner and stable ranking API. Estimate
4–8 agent days plus 1–3 days for evaluation/analysis, with compute duration
measured separately. No new reviewers are necessary for this retrospective
study. A human-reviewed extension requires its own workload estimate and labels.

### R4. Do agents, solo reviewers and teams produce reproducible review workflows?

**Tool deliverable:** an end-to-end review package covering search receipts,
deduplication, screening provenance, amendments, living updates and generated
reporting, with replay and discrepancy analysis.

**Design:** first replay bounded historical tasks using the preceding datasets,
comparing a scripted baseline with autonomous agent execution under the same
data, budgets and policies. Use TREC-COVID's documented snapshots to test temporal
updates; LitSearch adds a separately reported nonmedical retrieval stress test.
Then optionally run a prospective counterbalanced comparison of autonomous,
single-human-assisted and team workflows. Keep tasks, training, order effects
and the reference standard explicit. Size this pilot from feasibility data;
do not promise a powered noninferiority study before estimating variance and
the number of independent review tasks.

**Main outcomes:** reproducible result sets, unexplained record loss, traceable
decisions, reporting discrepancies, update-detection delay and total cost. The
prospective extension measures active human time, corrections and eligibility
agreement. Machine replay cannot demonstrate usability or human time savings.

**Publication:** JAMIA Open Original Research for biomedical workflow evaluation;
JAMIA Research and Applications is a more ambitious option if the informatics
contribution and external evaluation warrant it. JMLA is suitable when the main
question concerns information-specialist work.

**Dependencies and time:** working R1–R3 components and reporting integration.
Estimate 5–10 agent days for replay instrumentation and 2–4 for analysis/drafting.
An optional prospective pilot adds approximately 4–8 elapsed weeks and 20–60
participant-hours; these are planning ranges, not a sample-size justification.

## Existing datasets: fitness, rights and limits

| Dataset and primary source | Use and access/licence | Principal limitation |
|---|---|---|
| [SYNERGY](https://github.com/asreview/synergy-dataset) | Documented release: 26 reviews, 169,288 records, 2,834 inclusions; labelled screening/ranking; declared CC0 data. Pin the actual downloaded release. | Candidate-pool labels do not establish retrieval of all relevant literature. Historical review decisions can contain error. |
| [CLEF TAR 2017–2019](https://github.com/CLEF-TAR/tar) | Queries, PubMed identifiers and relevance labels; repository declares MIT. Useful for retrieval and screening. | Check overlap between years and other collections. Repository licensing does not transfer rights to every PubMed abstract or original review. Freeze metadata rather than silently refreshing it. |
| [Systematic-review seed collection](https://github.com/ielab/sysrev-seed-collection) | Queries, dates, seed studies, included studies, retrieval and snowballing outputs; repository declares MIT. Useful for seed discovery and query tests. | Seeds must not count as independently discovered successes. It is not a reference set of expert-approved cross-dialect translations. |
| [ASySD evaluation datasets](https://pmc.ncbi.nlm.nih.gov/articles/PMC10483700/) | Five labelled search datasets, 1,845–79,880 citations, including corrected duplicate IDs; linked [OSF deposit](https://osf.io/c9evs/). ASySD code is GPL ≥3. | Deposit/file-level redistribution licence was not verified here; article/code licences are not substitutes. Use corrected versions. Duplicate-publication labels do not validate study linkage. |
| [TREC-COVID Complete and rounds](https://ir.nist.gov/covidSubmit/data.html) | NIST topics/judgements and fixed CORD-19 releases; Complete uses 50 topics and the 16 July 2020 corpus. Corpus access has separate [data terms](https://ir.nist.gov/covidSubmit/round3.html). | Pooled judgements are incomplete. NIST explicitly disallows training on earlier-round labels then evaluating the full topic set against later chronological labels; use residual evaluation or disjoint topics. |
| [LitSearch](https://huggingface.co/datasets/princeton-nlp/LitSearch) | 597 expert-checked queries; 64,183 ML/NLP documents. [Repository licence](https://github.com/princeton-nlp/LitSearch/blob/main/LICENSE) is MIT. | Narrow domain and known-target retrieval, not systematic-review recall. Check paper-text rights separately; a repository licence is not blanket permission to redistribute its source literature. |
| [CSMeD-FT](https://github.com/WojciechKusa/systematic-review-datasets) | Full-text eligibility pairs; published train/dev/test sizes 2,053/644/636. [Original datasheet](https://papers.nips.cc/paper_files/paper/2023/file/4962a23916103301b27bde29a27642e8-Supplemental-Datasets_and_Benchmarks.pdf) specifies CC BY 4.0. | Full-text retrieval and extraction are separate failure points. CSMeD aggregates earlier collections; do not count reused reviews as fresh external validation. |

[BioASQ](https://participants-area.bioasq.org/datasets/) can extend biomedical
document/snippet retrieval, subject to its access terms, but its question-answer
labels do not establish systematic-search completeness.
[PubMedQA](https://pubmedqa.github.io/) tests yes/no/maybe answers from abstracts;
[SciFact](https://github.com/allenai/scifact) tests scientific claims and evidence.
They are optional component tests if those capabilities are added, not core
validation of Searchright's current search workflow.

No shortlisted dataset supplies universal truth for live search completeness,
all database dialects, or multi-report study linkage. Label each outcome by its
actual task and reference standard.

## Staffing options and scientific claims

| Mode | Work that can finish | Evidence and limitations |
|---|---|---|
| Autonomous research execution | Agents implement adapters, run frozen benchmarks, generate machine-labelled outputs, perform analyses and prepare evidence packages/manuscript drafts. A separate evaluator can execute sealed-label scoring. | Real route to R1's formal experiment, R2, R3 and R4's replay study. Report machine provenance and dependence on existing labels. Separate agents are not independent human reviewers; model diversity does not prove independence. |
| One human with agents | The author specifies the question, reviews methods and outputs, resolves selected errors and performs a measured solo workflow. | Supports accountable author review and human-observed outcomes. Repeating a judgement later does not turn one reviewer into two independent reviewers. |
| Multiple humans with agents | Information specialists/reviewers assess blinded samples, independently judge selected cases, adjudicate disagreement and participate in a workflow comparison. | Supports the corresponding expert-agreement and external-validation claims. Recruit only the roles required by the question; team availability is not a coding prerequisite. |

Autonomous experimental classifications must remain distinguishable from
human-confirmed eligibility decisions. This programme does not silently change
current production exclusion authority. A future autonomous production profile
requires its own explicit policy and implementation, rather than a research
benchmark being treated as permission. Human authors remain responsible for
publication and the journal's required disclosures.

The day estimates above assume one coordinating agent with bounded parallel
workers, an available build environment, existing component interfaces and
accessible data. They exclude provider quotas, unavailable licences, recruitment,
ethics processing where applicable, full-text access and editorial review.
Record agent hours, compute time, human person-hours and elapsed days separately.

## First affordable experiments

1. Within 1–2 agent days, freeze protocols, dataset versions, metrics and rights
   records; implement one existing-labelled-data adapter and a deterministic
   baseline. Run a small development subset to estimate runtime and failure
   modes. This is an engineering pilot, not the final test.
2. Within the next 2–5 agent days, run R2 if the data/rights are accessible, or R3
   on SYNERGY otherwise. Produce complete per-review results, failures and cost
   tables. Do not wait for every connector, user interface or companion project.
3. Run R1 on a deliberately narrow supported dialect subset. Expand only after
   the first experiment reveals which additional constructs matter.
4. Register the confirmatory design before exposing its held-out labels. An
   earlier exploratory result stays useful when clearly labelled. A second
   institution or specialist is added when external validation is the claim,
   not as an automatic approval stage for all experiments.

## Publication routes and reusable evidence

These are editorial-fit judgements, not promises of acceptance:

- [JMLA](https://jmla.pitt.edu/ojs/jmla/about/submissions) explicitly includes
  search-filter development/testing in Original Investigations: up to 5,000
  main-text words, a 250-word structured abstract and six figures/tables.
- [RSM](https://www.cambridge.org/core/journals/research-synthesis-methods/information/author-instructions/preparing-your-materials)
  currently offers Research Articles and Software Focus. It requires data
  availability, CRediT and highlights; general methodological value matters.
- [Systematic Reviews](https://link.springer.com/journal/13643/aims-and-scope)
  includes methods research concerning systematic reviews.
- [JAMIA Open](https://academic.oup.com/jamiaopen/pages/general_instructions)
  covers biomedical informatics and implementation, including application and
  database notes; accepted papers require a patient/community-facing abstract.
  [JAMIA](https://academic.oup.com/jamia/pages/General_Instructions) Research and
  Applications allows up to 4,000 words and a 250-word structured abstract.
- [JOSS](https://joss.readthedocs.io/en/latest/submitting.html) is a conditional
  companion software paper, not a rapid publication dependency: current rules
  require more than six months of active public development, demonstrated
  research use, usable software and open-source practices. AI assistance is
  permitted with disclosure and human validation/core design; AI-generated
  author interactions with editors/reviewers are prohibited except translation.
- [Scientific Data](https://www.nature.com/sdata/submission-guidelines) becomes
  appropriate if the programme creates a reusable, technically validated dataset
  with a meaningful contribution beyond mirroring existing benchmarks. A Data
  Descriptor documents data and technical validation, rather than serving as
  the comparative effectiveness paper. Reviewers need downloadable data and,
  from the second review round, a compliant public repository.

Recheck author instructions and charges when choosing a submission; no APC
amount or waiver is assumed. Distinct questions, endpoints and contributions
justify separate papers; routine development tasks do not each require one.

Every study should produce the exact-code/data manifest, frozen protocol,
licence record, raw predictions, evaluator outputs, analysis code and a candid
limitations section described in [evidence packages](evidence-packages.md).
That document also specifies the future Hugging Face dataset pipeline; publish
only eligible material, retaining restricted sources as identifiers and
reconstruction instructions. [Portfolio reuse](portfolio-reuse.md) describes
how reusable adapters and evidence can benefit companion tools without copying
their implementation or relabelling shared benchmarks as independent evidence.
