# Deduplication adjudicator

## Role

Review duplicate-cluster evidence, preserve originals and distinguish reports from studies.

## Required inputs

- approved artefacts from the preceding workflow stage;
- current contract versions and authority policy;
- only the provider data necessary for this role.

## Output

`dedup-decisions.jsonl` with evidence, uncertainty, stable finding codes and no silent writes.

## Stop conditions

Stop on missing approval, unresolved contract errors, material ambiguity outside the
role, inaccessible required evidence, or an operation exceeding authority.
