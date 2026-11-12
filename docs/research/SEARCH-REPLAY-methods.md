# SEARCH-REPLAY: reporting ambiguity and identifiable search uncertainty

Companion to `research/protocols/R7.protocol.json`, version 0.1.0 (draft).
No registration, completed study, result or journal acceptance is claimed.

## Contribution relative to prior work

Rethlefsen et al. (2024, DOI 10.1016/j.jclinepi.2023.111229) audited 100 biomedical reviews and 453 database searches. Their definition combined six reporting items with result counts within 10% of those reported; one report was fully reproducible. Pozsgai et al. (2021, DOI 10.1002/ece3.8154) used synchronised searches to examine platform/location variation. Lin et al. (2026, DOI 10.1136/bmjopen-2026-123460) discuss persistent search and record-management auditability gaps. These establish prior art; a first-study or solved-reproducibility claim would be inappropriate.

The proposed contribution links specific omissions to defensible alternative reconstructions, record-set disagreement and downstream consequences, separating effects that can and cannot be identified. Matching hit counts can conceal different records; changed counts can reflect index changes despite faithful execution. Retain a count-based legacy endpoint only as a labelled secondary comparison.

## Sampling and estimand

Pilot: 24 reports, eight per biomedical, social/environmental and bibliometric/scientometric stratum. Candidate confirmation: 150 reports, 50 per stratum, from frozen documented sampling frames. Final numbers follow pilot-informed precision and feasibility review before confirmation. The report, not each query or retrieved record, is the sampling unit. Pilot records cannot enter confirmation.

Audit every declared database search. Report frame construction, date bounds, screening rules, duplicates, seed and draw order. Do not replace inaccessible reports with convenient open-access papers. The primary estimand is the proportion of sampled reports for which reconstructability of all declared searches can be established from public reporting without undocumented assumptions. Classify inaccessible and non-assessable cases separately, retain them in the sampling accounting, and show identification bounds plus an assessable-only estimate. Non-assessability is not evidence of poor reporting. An equal-weight three-stratum mixture does not estimate all scholarship.

## Blinded reconstruction

Two assessors independently inspect the article, supplements, cited protocol and permitted repository artifacts, blinded to each other and initially to reported hit counts. Retain source spans for literal strategies, database/platform, coverage subsets, search dates, fields, Boolean/proximity operators, limits, vocabulary expansion, filters and source-specific counts. Map reporting to PRISMA-S with evidence locations; distinguish absent, ambiguous, contradictory, inaccessible and not applicable. PRESS-based conduct assessment is separate from reporting completeness.

Preserve first passes, dissent and adjudication. Agent critiques may develop the instrument but cannot be counted as independent human reconstruction or fed back into sealed confirmation cases.

## Four conditions

A: execute exact reported syntax where valid and authorised. Preserve parse errors/refusals and original bytes. Cosmetic extraction changes require a predeclared rule.

B: freeze a bounded set of expert-justified plausible interpretations before viewing their results. Each has a scenario ID, assumption, rationale and query digest. Unknown detail remains unknown; do not pick a convenient interpretation to match a published count.

C: optional author clarification using a standardised approved process after A/B freeze. Clarification cannot retroactively improve the public-reporting score; response selection is reported.

D: explicitly repaired or enhanced strategies, analysed separately. An improvement is not labelled a reproduction and cannot silently enter A/B.

## Execution and identifiability

Execute paired scenarios in the same controlled source/time/access setting, randomise request order and preserve repetition windows, exact requests, source records, pagination and deduplication dispositions. Record licence coverage, UI/API differences and platform versions where observable.

Historical replay requires preserved historical observations or snapshots. Without them, current re-execution is not reconstruction of the authors' historical corpus. Keep reporting ambiguity, technical/parser behaviour, temporal index change, access/platform differences, identity reconciliation and optional model randomness distinct. Estimate variance components only where crossed replication identifies them; otherwise report contrasts and explicitly unidentified components.

## Analysis

Primary: report-level reconstructability as defined above. Secondary: reporting prerequisites; executable fraction; pairwise Jaccard and symmetric differences; raw-source and canonical-corpus size ranges; known-study retention where an independent reference exists; reconstruction/repair time; and fixed downstream analytic sensitivity. Both-empty comparisons are labelled empty rather than inflating mean similarity; one-empty comparisons have zero overlap.

A preregistered interpretation envelope is descriptive, not a confidence interval or a bound on all possible searches. Cap enumeration or prespecify balanced/seeded scenario sampling; disclose that untested scenarios may extend the envelope. Use report-clustered bootstrap or prespecified stratified estimators for sampling uncertainty. Additional outcomes are exploratory unless allocated to a confirmatory family. No uncertainty interval for unobserved whole-literature truth is fabricated.

## Remediation experiment

After baseline freeze, compare deterministic lint, human-guided Searchright diagnostics and an optional model-assisted audit using counterbalanced tasks and explicit budgets. Measure correct detection/correction, newly introduced errors and active time. A reproducible but wrong query is not an improvement. Secondary bibliometric or ASReview extensions must not block the primary audit.

## Outputs and safeguards

Release the rights-clear rubric, scenario contracts, reconstruction traces, permitted identifiers, analysis code, canonical CSL-JSON citations, sidecars and replay instructions. Restricted bytes remain reference-only. Do not expose contacts, credentials or unauthorised licensed exports. Reporting omissions do not establish misconduct or original intent.

Each table, figure and claim binds to protocol/corpus/code/run/result versions. A primary metaresearch paper and any later remediation/software paper need genuinely different endpoints and explicit data-reuse disclosure. Potential audiences are Journal of Clinical Epidemiology, JMLA and Research Synthesis Methods; current submission requirements require verification.
