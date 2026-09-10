# ADR 0019: Single-owner, sealed agent-panel governance

- **Status:** accepted
- **Date:** 2026-09-10
- **Owner:** Dylan Mordaunt
- **Conductor track:** 11

## Context

Searchright is maintained as a single accountable human/developer project. Earlier
skill and calibration text assumed that routine track closure required two
additional human information specialists and described the PRESS stage as an
independent human review. That was inconsistent with the project's operating
model and risked conflating three different things:

1. automated validation;
2. PRESS-aligned review by specialist agents;
3. independent human PRESS peer review.

The repository already requires human authority for live execution, durable
writes, protocol amendments and final screening decisions. It also contains an
isolated agent-panel framework and a top-level single-owner governance decision.
The skill package must apply the same model without overstating the evidence.

## Decision

Searchright's default internal review gate is a sealed, owner-adjudicated panel of
at least five specialist agents:

- methodology;
- information retrieval / PRESS-aligned critique;
- implementation and testing;
- security, privacy and rights;
- adversarial / replication.

Each role produces an isolated first pass. Responses, abstentions and dissent are
frozen before synthesis. The accountable owner accepts, rejects, defers or
requires remediation for every material finding.

Agent output remains advisory. Only the accountable owner may approve the
protocol, live execution, durable writes, final eligibility decisions,
publication or evidence promotion. Another human may be explicitly delegated a
screening role by the owner, but no default second-human requirement is imposed.

An independent human information specialist review is optional and remains a
separate evidence class. Searchright must record the actual method and must never
represent an agent panel or owner adjudication as independent human peer review.

## Consequences

- The systematic-search skill, workflow, authority table and calibration package
  use the same single-owner model.
- Live execution requires a completed panel adjudication as well as owner strategy
  and live-execution approval.
- PRISMA-S reporting records whether review evidence was automated,
  agent-assisted, owner-adjudicated or independently human-reviewed.
- Existing independent-human-review contracts remain readable and may be used
  when an external human review actually occurs.
- External human calibration is no longer a default Track 11 blocker.
- Agent-panel execution, owner adjudication, host/model evaluation, downstream
  adoption and registry acceptance remain separately evidenced gates.

## Rejected alternatives

### Require two additional human reviewers for every release

Rejected because it does not match the single-developer operating model and would
turn a useful optional evidence class into an unfulfillable default gate.

### Treat agent consensus as peer review

Rejected because models are not independent human information specialists and
consensus does not confer authority or methodological validity.

### Let the owner silently approve agent findings

Rejected because it would lose dissent, weaken reproducibility and obscure the
basis of consequential decisions.

## Verification

The skill checker and dedicated regression tests must fail when:

- the workflow no longer identifies the single accountable owner;
- any required agent-panel role is removed without an explicit evidence change;
- first-pass isolation, dissent preservation or owner adjudication disappears;
- live execution omits the panel-adjudication gate;
- agent output is labelled independent human peer review;
- final screening authority is assigned to an agent;
- the optional human-calibration template implies an unobserved review.
