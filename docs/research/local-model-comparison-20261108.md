# Local models for search translation, screening and evidence extraction

**Status: planned comparison within the existing [frontier plan](../../conductor/frontier-plan.md) and Track 17/23/25 owners, not a new track.** Searchright's deterministic search/query AST, provider semantics, source lineage, PRESS and study-level reporting are authoritative. The common device/model runtime evidence is owned by [models_lang](https://github.com/edithatogo/models_lang).

## Minimal experiment

- Sample versioned difficult Boolean/field/proximity queries, publication-type restrictions, translated syntax and PRESS problems; split by source topic/author, not near-duplicate strings.
- Establish no-model AST/rules baseline, then small Liquid LFM3 and d1 for bounded screening/choice, and strong local generative model for proposed translations or study-screening justifications. Never let a generated string bypass parser or provider dialect validation.
- Compare native query translation validity and *retrieval equivalence* using actual source runs, not string similarity. For screening record recall at specified workload, false exclusions, inter-reviewer variability, calibration/abstention and time. For extraction record source spans and unsupported statements.
- Optionally compare long-context reading with chunked retrieval on the same evidence bundle. Enforce equal token/retrieval budgets and no systematic leakage from the test set.
- Record model checkpoint revision, runtime, hardware profile and prompt template in immutable run receipts; attach a compatible `models_lang` qualification record. Never copy restricted article full text or reviewer-private decisions into public HF artefacts.
- Evaluate autonomous/solo/team modes using existing workflow identity and human checkpoint contracts. All automatic exclusions must have explicit accountable review or a prespecified, validated low-risk scope.

## Acceptance and delivery

A first coding-agent increment adds an optional model comparison fixture and scoring adapter without making model availability a dependency of basic search. Reuse ASReview and existing statistical/benchmark libraries. Run a blinded, independently assessed sample before a performance claim. Results can be negative; preserve them and do not create a second planning hierarchy or shared orchestration subsystem.
