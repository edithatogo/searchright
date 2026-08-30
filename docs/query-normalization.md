# Bounded native query normalization

Track 02 implements source-preserving normalization for the explicitly declared
project-authored dialect subsets in `contracts/query-corpus/loss-matrix.json`.
These fixtures are not external vendor conformance tests or evidence that two
databases retrieve equivalent records.

Exact native text and byte spans remain available when normalization fails.
Unsupported fields, foreign dialect syntax, undefined set references and
unmodeled limits must not silently become a complete semantic strategy.
Callers must inspect normalization state and review-required diagnostics before
using the derived AST. Compilation warnings remain independently applicable:
a successful round trip is not proof of lossless cross-database translation.

## Resource limits and incomplete metadata

Semantic parsing accepts at most 262,144 raw UTF-8 bytes, 4,096 lines and 4,096
tokens per expression. A conservative combined nesting/reference/operator depth
of 64 and an expanded-node/cumulative set-storage budget of 16,384 bound
recursive parsing and set cloning. Exceeding a budget rejects semantic parsing;
it never returns a truncated AST as complete.

If the raw-byte or line limit is exceeded, the source-preserving wrapper retains
the exact raw text but does not expand per-line metadata. It returns a zero-width
`Unknown` sentinel named `unexpanded-source` and a review-required
`native.resource_limit` diagnostic. That sentinel is not a parsed source line.
Callers must not interpret it as full span coverage. The input string itself is
already caller-owned; these limits bound additional parsing and expansion work.

Only the explicitly modeled English restriction on the currently selected set,
or the project-form `limits: english`, is normalized. Other restrictions, a
different target set, combined limits or expressions following a limit require
review and remain raw-only rather than silently changing the search.

## Review and authority

Methodology, safety and adversarial agents perform isolated first passes over
the exact corpus and loss-matrix digests. Preserve their findings and dissent;
submit the synthesis to the accountable owner for decision. No second person is
required and no agent can confer owner approval.

The checked-in named-filter pack is synthetic and structural only. Real packs
need exact expressions, source/version/checksum, applicable platform, currentness
evidence and a rights decision, followed by panel findings and owner disposition.
Contract validation checks metadata shape; it does not authenticate evidence.
Topic-specific PRESS adequacy, recall and retrieval equivalence additionally need
a protocol, eligibility criteria and empirical results. The syntax corpus cannot
supply those materials.

## Integration and migration

Historical August 29 receipts remain evidence of the revisions they name, not
validation of later fixes. Current panel records supersede their second-person
review wording without converting pending decisions into approvals.

The existing wire schema is unchanged. Stricter normalization can return
raw-only/review-required results for inputs previously labeled complete.
Persisted source text is never rewritten automatically; re-normalize explicitly
and review any changed result. Track 02 remains open while its required evidence
and owner decisions are missing.
