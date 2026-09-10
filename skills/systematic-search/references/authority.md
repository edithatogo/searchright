# Authority policy

Searchright has one accountable human owner. Agents provide isolated, advisory
review and execution assistance; they do not acquire decision rights through
consensus, confidence or tool access.

| Operation | Default agent authority | Accountable-owner gate |
| --- | --- | --- |
| Draft question/framework | Draft | Approve scope |
| Draft eligibility criteria | Draft | Approve and version |
| Recommend sources | Advisory | Confirm access and coverage |
| Build/translate query | Draft | Review warnings and approve |
| Automated PRESS lint | Automatic, read-only | Review exceptions |
| PRESS-aligned agent-panel critique | Advisory, sealed first passes | Adjudicate findings and approve strategy |
| Claim independent human PRESS peer review | Denied | Record only attributable human review and independence evidence |
| Fixture/replay execution | Automatic within declared budget | None beyond approved policy |
| Live execution | Denied | Explicit owner approval |
| Dedup exact candidates | Preview | Apply or reject clusters |
| Dedup fuzzy candidates | Advisory | Owner review |
| Title/abstract recommendation | Advisory | Owner or designated human records final decision |
| Any final exclusion | Denied | Owner or explicitly designated human |
| Full-text final decision | Denied | Owner or explicitly designated human under the approved protocol |
| PRISMA generation | Automatic from audit | Review gaps, counts and actual review-method label |
| Protocol amendment | Denied | Owner approval and versioned amendment |
| Durable canonical write | Denied by default | Explicit owner approval |
| Registry/publication write | Denied | Explicit owner approval |
| Evidence or maturity promotion | Denied | Owner adjudication against recorded gates |

## Panel boundary

A consequential internal review uses at least five isolated roles: methodology,
information retrieval / PRESS-aligned critique, implementation/testing,
security/privacy/rights and adversarial/replication. First-pass responses are
frozen before synthesis. Material dissent, abstentions and failed responses remain
visible.

The owner adjudicates every material finding as accepted, rejected, deferred or
remediation required. Owner adjudication completes the internal review gate but
does not transform agent output into independent human peer review.

## Screening boundary

Searchright does not require multiple human reviewers by default. A protocol may
designate additional humans, but the single-owner project can record final
eligibility decisions through the accountable owner. Agents remain advisory and
cannot silently resolve conflicts or make irreversible exclusions.

## Evidence boundary

Record the actual review method:

- automated lint;
- PRESS-aligned agent critique;
- owner-adjudicated agent panel;
- independent human PRESS peer review.

Only the fourth label requires and permits a claim of independent human review.
See `agent-panel.md` for isolation, dissent and receipt requirements.
