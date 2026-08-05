# Validation dependencies

`validation.in` declares compatible ranges. `validation.txt` is the exact-pinned
bootstrap lock used by CI. A release candidate must replace it with a
`pip-compile --generate-hashes` lock containing hashes for every supported
platform; this generation environment could not obtain distribution hashes.

The repository validator checks that the exact lock exists and contains no
unbounded requirement.
