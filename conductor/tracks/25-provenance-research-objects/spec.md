# 25: Provenance and research-object interoperability

## Objective

Make review lineage, living updates and durable research-object deposit
first-class outputs through deterministic RO-Crate and W3C PROV representations.

## Scope

- Immutable identifiers and predecessor relationships.
- Review, strategy, run, receipt, amendment and decision entities.
- RO-Crate 1.3-compatible JSON-LD export boundary.
- W3C PROV entity/activity/agent export.
- OSF, Zenodo and repository handoff packets without automatic deposit.
- Round-trip and golden-fixture contracts.

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `25`.

## Acceptance contract

Source exports, schemas and documentation are statically consistent. Compiler,
round-trip, repository validation and live deposit remain separately evidenced.

## Out of scope

A provenance graph does not establish the truth, completeness or quality of a
review and does not imply that any repository accepted a deposit.
