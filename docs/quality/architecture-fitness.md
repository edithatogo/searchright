# Architecture fitness

`verification/architecture-policy.json` and
`scripts/check_architecture_fitness.py` make key architectural boundaries
executable.

The current policy requires:

- product-neutral contract and runtime crates not to depend on Searchright
  product crates;
- `reqwest` to remain confined to `searchright-connectors`;
- provider endpoint literals to remain in the connector boundary;
- final eligibility authority markers to remain in the access, agent,
  screening-contract and screening-runtime boundaries;
- every external-write script to expose its declared explicit apply flag and
  environment gate;
- every workspace package to remain `publish = false` until separately
  promoted.

These are source-placement invariants. They complement, but do not replace,
compiler dependency resolution, operating-system sandboxing, network egress
controls or runtime authorisation tests.
