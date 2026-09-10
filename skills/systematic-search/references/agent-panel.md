# Accountable-owner agent-panel protocol

## Purpose

Searchright is developed and governed by one accountable human owner supported by
specialist agents. Agent panels provide structured, adversarial and reproducible
review evidence without pretending to be additional human reviewers.

This protocol applies to internal methodological, implementation, testing,
security, safety and adversarial review gates. It does not convert an agent into
an information specialist, authorise final screening decisions, or satisfy a
claim of independent human PRESS peer review.

## Authority

The accountable owner is the only actor who may:

- approve the review question, eligibility criteria or protocol amendment;
- authorise live provider execution, durable writes or publication;
- make or delegate final eligibility decisions to another identified human;
- accept, reject, defer or require remediation of panel findings;
- promote an evidence or maturity claim.

Agents may draft, test, compare, critique, rank and recommend. Their findings are
advisory and cannot silently alter canonical state.

## Panel composition

Use at least five isolated roles for a consequential review:

1. **methodology** — question framing, source selection, recall, filters and
   review-method consistency;
2. **information retrieval / PRESS-aligned critique** — the six PRESS domains,
   native syntax and translation loss;
3. **implementation and testing** — contracts, code paths, fixtures, failure
   handling and regression coverage;
4. **security, privacy and rights** — secrets, hostile content, licensed sources,
   data minimisation and redistribution constraints;
5. **adversarial / replication** — counterexamples, unsupported claims,
   reproducibility and rollback.

Additional roles may be added. Removing a role requires an owner-recorded
rationale and may lower the evidence level.

## Sealed first passes

Before any synthesis:

- freeze the request, input artefact digests, role instructions, tool allowances,
  model/host identifiers and evaluation criteria;
- give each role an isolated context and do not disclose other first-pass
  findings;
- require findings to identify evidence, severity, confidence, uncertainty and a
  proposed disposition;
- retain failed, abstaining and dissenting responses;
- redact credentials, licensed full text, personal data and sensitive query
  material before persistence.

A synthesis may cluster duplicate findings, but it must not erase minority
findings or convert uncertainty into agreement.

## Owner adjudication

The owner adjudication record must state, for every material finding:

- accepted, rejected, deferred or remediation required;
- rationale and evidence references;
- resulting change or explicit no-change decision;
- residual risk and next gate;
- owner identity and timestamp.

A panel is complete only when the sealed inputs, individual responses, synthesis,
dissent register and owner adjudication are content-addressed and mutually
referenced. Panel completion is not the same as product approval.

## PRESS and reporting terminology

Searchright distinguishes four evidence classes:

1. **automated lint** — deterministic syntax or contract checks;
2. **PRESS-aligned agent critique** — isolated agents apply the six PRESS domains;
3. **owner-adjudicated agent panel** — the accountable owner disposes of the
   panel findings;
4. **independent human PRESS peer review** — an actual independent human
   information specialist performs and attests the review.

The project’s default internal gate is class 3. Class 4 is optional and may be
recorded only when it actually occurred. Neither agent consensus nor owner
adjudication may be labelled independent human peer review.

For PRISMA-S reporting, record the actual method and evidence references. A
completed internal panel may be reported as agent-assisted or owner-adjudicated
review, not as human PRESS peer review.

## Fail-closed conditions

Stop promotion when:

- the sealed request or an input digest cannot be verified;
- panel roles shared first-pass findings before freezing their responses;
- a material dissent or abstention is missing from the synthesis;
- the owner adjudication is absent or does not identify the owner;
- an agent attempted to make a final exclusion, protocol amendment, live-write
  or publication decision;
- the requested claim exceeds the recorded review method;
- credentials, licensed content or sensitive material entered a receipt.

## Minimum receipt

A panel receipt should contain:

```yaml
schema_version: org.searchright.agent-panel-review.v1
request_sha256: <sha256>
input_artifacts:
  - path: <path>
    sha256: <sha256>
panel_roles: [methodology, information_retrieval, implementation_testing, security_privacy_rights, adversarial_replication]
responses:
  - role: <role>
    response_sha256: <sha256>
    status: completed | abstained | failed
synthesis_sha256: <sha256>
dissent_register_sha256: <sha256>
owner_adjudication_sha256: <sha256>
owner_id: <stable identifier>
status: awaiting_panel | awaiting_owner | remediating | accepted | rejected
claim_boundary: <plain-language boundary>
```

The receipt must never contain provider credentials, raw licensed full text,
hidden benchmark labels or sensitive identifiers.
