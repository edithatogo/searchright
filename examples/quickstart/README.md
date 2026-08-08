# Searchright fixture-only quickstart

This quickstart is intentionally network-off. After the compiler-backed bootstrap
has generated `Cargo.lock`, run the checked-in fixture workflow through each
interface:

```bash
cargo run -p searchright-cli -- --help
cargo test --workspace --locked
cargo run -p searchright-mcp
```

Use the review-plan, strategy, provider-page, screening and PRISMA examples under
`contracts/examples/` as canonical inputs. Live provider execution requires an
explicit provider capability, host allowlist and opt-in environment. No example
authorises final agent exclusions or external writes.
