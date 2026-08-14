# ADR 0004: WASI component provider plugins

Status: accepted for the component trust boundary; runtime sandbox remains proposed

Mature third-party connectors will use WIT contracts and capability-scoped WASI
components. Native in-process plugins are not accepted by default because they
expand the memory-safety, egress and supply-chain boundary.

## Component release trust

Before a provider component can be admitted, Searchright verifies all of the
following as one fail-closed operation:

- the reviewed manifest is valid and names the exact WIT ABI;
- the BLAKE3 digest of the component bytes matches the manifest;
- an Ed25519 detached signature covers a domain-separated, length-prefixed
  release message containing the manifest digest, component digest, component
  identity/version, ABI, key identifier, trust-policy identifier and validity
  window;
- the public key is present in the reviewed trust policy, is authorised for the
  exact component identifier, and was valid when the release was signed; and
- neither the release nor an effective key revocation has expired its authority.

Trust policies and revocations are versioned contracts. Key identifiers are
unique, revocations must refer to a trusted key, and no wildcard component
authority is accepted. The repository fixture key is test-only and is not a
distribution trust root.

This decision proves local cryptographic admission of exact bytes only. It does
not establish a component registry, publisher identity, key custody, revocation
distribution, transparency log, Wasmtime execution, network isolation, or live
provider support. Those remain explicit Track 24 gates.
