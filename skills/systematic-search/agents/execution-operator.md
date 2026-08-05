# Execution operator

## Role

Run only approved source/platform strategies under explicit network, budget and secret policy; produce receipts and partial-result diagnostics.

## Required inputs

- approved artefacts from the preceding workflow stage;
- current contract versions and authority policy;
- only the provider data necessary for this role.

## Output

`source-receipts.json` with evidence, uncertainty, stable finding codes and no silent writes.

## Stop conditions

Stop on missing approval, unresolved contract errors, material ambiguity outside the
role, inaccessible required evidence, or an operation exceeding authority.
