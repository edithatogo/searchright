# 32: Cross-repository contract release train and downstream canaries

## Objective

Coordinate CiteWeft, Searchright/shared core and Sourceright compatibility without coupling repositories or automatically promoting revisions.

## Scope

- Pinned producer-consumer promotion order
- Cross-repository consumer fixtures and canaries
- Explicit promotion, rollback and receipt authority

## Requirements owned

`SR-065, SR-070, SR-076, SR-077`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
