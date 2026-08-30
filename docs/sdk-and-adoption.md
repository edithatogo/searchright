# SDK and adoption strategy

The Rust facade is canonical. Python and TypeScript clients are to be generated
from OpenAPI/JSON Schema contracts and remain thin transport/validation layers;
they must not fork query, screening or evidence semantics.

Every tutorial is fixture-backed, network-off by default, and paired with CLI,
MCP and Rust-facade equivalents where the operation exists. Adoption material
must state the current evidence ceiling, data-handling defaults, unsupported
sources and the distinction between agent recommendations and human decisions.

Publishing an SDK requires generated-source reproducibility, consumer contract
tests, install smokes, semver review, signed packages and explicit approval.

## Contract-only binding validation

Before running the network-free harness, install the locked development tools:

```sh
npm ci --prefix requirements/bindings --ignore-scripts --no-audit --no-fund
python3 scripts/check_contract_bindings.py
```

The gate runs pinned TypeScript and Pyright checks over the generated packages
and assignment fixtures, along with generation freshness and import checks.
These types do not implement JSON Schema validation: numeric bounds, patterns,
array cardinality and other runtime constraints still require the canonical
schema validator. Rust schema drift evidence hashes both compared schema
documents and retains dialect/base-URI keywords; it is not an equivalence proof.
