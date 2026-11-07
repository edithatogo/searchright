# Benchmark and calibration protocol

## Purpose

Evaluate search compilation, duplicate clustering, supplementary discovery,
screening prioritisation and reporting without allowing benchmark labels to leak
into generation or tuning decisions.

## Evaluation strata

1. **Contract conformance:** schemas, identifiers, invariants and deterministic
   serialisation.
2. **Dialect fidelity:** exact and lossy translation against curated query pairs.
3. **Retrieval sensitivity:** recall of preregistered seed records and known
   relevant records, reported by source and topic.
4. **Deduplication:** pairwise and cluster precision/recall with ambiguity review.
5. **Prioritisation:** work-saved-over-sampling, recall at screening fractions,
   calibration error and subgroup performance; ranking cannot discard records.
6. **Human calibration:** information-specialist review of strategies, warnings,
   receipts, diagnostics and generated appendices.
7. **Operational robustness:** malformed responses, retries, replay, cache,
   cancellation, hostile metadata and resource budgets.

## Leakage controls

- Keep hidden evaluation reviews separate from development fixtures.
- Hash and version every split.
- Record all model, prompt, feature and threshold changes before evaluation.
- Report failed and null runs.
- Do not use test labels for prompt construction, query expansion or stopping.

## Baselines

Deterministic no-ranking order, identifier-only deduplication, human-authored
source strategy, and the previous tagged Searchright version are mandatory
comparators. External systems are named only when their licence permits a fair,
reproducible comparison.

## Complete-population sealed scoring

`searchright_bench::evaluate_sealed_benchmark` scores a complete binary
classification of the manifest's declared population. Every manifest case must
appear exactly once in the external labels and exactly once in the predictions.
Unknown, missing, blank or duplicate label identifiers are rejected, as are
unknown, missing or duplicate prediction identifiers. Repeated rows are invalid
even when they agree. A missing prediction is not silently treated as negative,
and this API does not support an uncounted abstention. An experiment with
abstentions or a partially judged corpus needs its own explicit scoring protocol.
Identifiers remain exact strings; they are not trimmed, case-folded or Unicode
normalised.

Use `searchright_bench::sealed_label_digest` to commit to the actual label
population and values. Its `sealed-labels v1` byte encoding is:

1. The ASCII bytes `org.searchright.sealed-labels.v1` followed by one NUL byte.
2. The number of labels as an unsigned 64-bit big-endian integer.
3. Entries sorted lexicographically by their identifier's UTF-8 bytes. Each entry
   contains the identifier's UTF-8 byte length as an unsigned 64-bit big-endian
   integer, the identifier bytes and one label byte: `0` for false or `1` for true.

SHA-256 of those bytes is encoded as 64 lowercase hexadecimal characters. Length
prefixes permit embedded separators without ambiguity; there is no trailing
newline. For the synthetic labels `r1=true`, `r2=false`, `r3=true`, the digest is
`040b66081df5355f6f01dd418b8dd96a0b24df4479bbec4d6f57bc7d184adea6`.
This commits to the labels, not the source file's incidental whitespace or row
ordering. Record an additional source-file hash in the dataset manifest when
exact acquisition bytes matter.

The evaluator recomputes this digest and compares it with its `label_digest`
argument and, when present, `SealedSplitManifest.label_digest`. The manifest and
label identifier sets must also match exactly. A syntax-valid digest alone is
insufficient. Keep the independently frozen commitment in the controlled
evaluation environment; checked-in sealed manifests continue to omit label
digests. A caller able to change both labels and their commitment can generate a
new matching pair, so these checks do not establish independent custody, label
quality or freedom from model leakage.

**Migration:** the evaluation function's arguments are unchanged, but callers
must replace placeholder/arbitrary digests with `sealed_label_digest` output and
provide complete unique labels and predictions. Previous permissive results are
not retroactively verified; rerun them against the declared population before
using them as evidence. Additional `BenchmarkError` variants distinguish identity
and commitment failures and may require updates to exhaustive matches. The
synthetic regression vectors test integrity only and are not external benchmark
results or evidence that an autonomous review is valid.

## Release gate

Internal benchmark success supports a fixture-proven claim only after the Rust
suite runs. It never substitutes for independent information-specialist review
or live database evidence.
