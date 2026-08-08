# MSRV and package-surface policy

Searchright separates rapid toolchain adoption from downstream compatibility.

- The repository development toolchain remains Rust **1.97.1**.
- `evidence-search-contracts` and `evidence-search-core` declare Rust **1.88** until a lower, tested MSRV is demonstrated.
- Application and experimental crates may advance faster than the neutral contracts and core.
- MSRV changes require a compatibility receipt, downstream Sourceright/CiteWeft checks and a documented rationale.

## Public package surface

The workspace uses crates as internal architecture boundaries, not as an implicit promise to publish every crate. The only intended public package candidates during alpha are:

1. `evidence-search-contracts` after neutral contract governance is proven;
2. `evidence-search-core` after Sourceright dual-run parity;
3. `searchright-plugin-sdk` after component conformance testing;
4. the Searchright application/CLI only after the internal dependency graph is consolidated.

All other crates are non-publishable implementation details. A source path, Cargo package or generated API is not a SemVer commitment unless it is listed in `release/public-packages.json` with `publish_ready: true`.
