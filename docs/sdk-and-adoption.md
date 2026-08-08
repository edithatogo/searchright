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
