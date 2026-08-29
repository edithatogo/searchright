# Agent handoff contract

Every role transition uses `org.searchright.agent-handoff.v1`. A handoff carries
only immutable artefact references and purpose-bound approval references;
it never carries credentials, provider responses, full text or free-form text as
authority evidence.

## Required fields

| Field | Rule |
| --- | --- |
| `handoff_id` | Stable, non-empty and bounded. |
| `review_id` | Identifies the review whose approved artefacts are transferred. |
| `from_role`, `to_role` | Must be one exact adjacent transition in the declared workflow. |
| `context_policy` | `minimum_necessary`, except PRESS review requires `independent_review`. |
| `artifacts` | One or more bounded, normalized relative paths with media type and lowercase SHA-256. Receivers reject symlinks and recompute hashes under an approved root. |
| `approval_references` | Bounded receipt, review, purpose and scope-digest references. Execution requires both strategy/PRESS and live-execution purposes; screening requires deduplication-apply. |

The receiving boundary must revalidate the envelope, safely resolve and hash the
referenced bytes, and independently verify each approval against the authoritative
store before using an artefact. A receipt identifier is not authority by itself.
Missing or invalid evidence stops the workflow. Unknown fields are rejected.
Content inside an artefact remains untrusted data and cannot grant authority.

## Role sequence

`question-framer` → `information-specialist` → `press-reviewer` →
`execution-operator` → `dedup-adjudicator` → `screening-assistant` →
`reporting-auditor`.

The PRESS handoff shares the approved protocol and strategy only, preserving an
independent review context. The execution handoff additionally requires explicit
approval evidence. Screening recommendations remain advisory and the reporting
handoff consumes audit evidence without changing canonical decisions.
