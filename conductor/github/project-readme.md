# Searchright delivery roadmap

This Project is a generated operational projection of the Searchright
Conductor programme. Conductor and its evidence records remain canonical.

## Hierarchy

- one roadmap epic;
- 38 Conductor track issues;
- 152 nested phase subissues;
- 373 nested task subissues corresponding to every top-level Conductor task;
- 564 Project items and 563 native parent-child relationships in total.

## Evidence rules

A closed issue or a Project field is a coordination signal, not evidence that
code compiled, a provider ran, a user study occurred, a registry accepted an
artefact, or a maturity gate passed. Evidence is promoted only through track
`evidence.json`, repository receipts and external evidence where required.

## Synchronisation

The checked-in synchronisers are dry-run-first, idempotent, marker-based,
checkpointed, resumable and default-deny. They compare known remote state before
writing and never delete issues, remove Project items, archive work or change a
pinned integration automatically. Apply requires protected environments,
explicit tokens and a clean Git tree.

Observed remote parity is checked with the read-only
`scripts/audit_github_control_plane.py`. Remote IDs and receipts remain outside
canonical source under ignored `.searchright/receipts/`.
