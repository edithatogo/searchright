# Contributing

Start with `conductor/product.md`, `conductor/requirements.md`,
`conductor/design.md` and `AGENTS.md`. Every change must identify its contract,
track and evidence level.

## Local gates

```bash
./scripts/bootstrap.sh
./scripts/verify.sh
```

The intended Rust gates are formatting, strict Clippy, unit/integration/doc tests,
contract snapshots, property tests, coverage, dependency/licence policy, API
compatibility and security workflow linting. Live provider tests are opt-in and
must never be part of default CI.

## Change discipline

1. Add or update a Conductor track/spec before material implementation.
2. Change canonical contracts before adapters.
3. Add deterministic fixtures before live tests.
4. Preserve audit semantics and migration paths.
5. State limitations; a scaffold is not a supported provider or accepted registry
   listing.
