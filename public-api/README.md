# Candidate public API evidence

All workspace crates are currently `publish = false`. Only the candidates in
`release/public-packages.json` receive API/SemVer surveillance. CI records their
current API surfaces and compares them with the pull-request base when the
candidate existed in that baseline.

Generated API files are workflow artefacts until a package is promoted to a
stable public contract. Promotion requires reviewed snapshots, SemVer policy,
compiler evidence and an explicit release decision.
