# Cargo-vet trust governance

Searchright requires `safe-to-deploy` for normal dependencies and
`safe-to-run` for development-only dependencies. An advisory, licence scan or
successful build does not satisfy this source-review requirement.

## Peer audit imports

The repository imports audit evidence from four established Rust projects:

- Bytecode Alliance's Wasmtime audit registry;
- Embark Studios' Rust ecosystem audit registry;
- Mozilla's supply-chain audit registry; and
- Zcash's Rust ecosystem audit registry.

The exact URLs are allowlisted by `scripts/check_cargo_vet_governance.py` and
the fetched evidence is pinned in `supply-chain/imports.lock`. These are peer
audit claims with preserved attribution. They are not Searchright-authored
audits and do not establish that every locked dependency has been reviewed.

At the locked dependency graph on 2026-08-12, these imports reduced the
unreviewed set from 273 dependency versions to 242: 225 require
`safe-to-deploy`, 17 require `safe-to-run`, across 229 unique crate names.
Before accountable backlog acceptance, `cargo vet --locked` therefore failed
closed.

## Residual review path

The preferred resolution is a dependency-review campaign:

1. remove unused or unnecessarily duplicated dependencies;
2. upgrade or constrain versions to audited paths when that is compatible with
   the MSRV and public contract;
3. import additional authoritative peer audits only after reviewing their
   ownership, criteria mapping and URL stability; and
4. perform and record Searchright audits for the exact residual versions,
   prioritising cryptography, TLS, native build scripts, proc macros and network
   runtimes.

An exemption is a temporary risk acceptance, not audit evidence. The accountable
owner authorised the current set only as exact crate-version-and-criterion entries with a
linked GitHub issue, rationale, risk summary, replacement plan, decision
evidence and an expiry no more than 90 days after proposal. Wildcard publisher
trust is prohibited. Every current exemption is linked to issue #241 and expires
on 2026-11-10; the dependency-review backlog remains open until audits replace
the exemptions.

## Obsolete exemption retirement (2026-11-06)

The all-features locked dependency graph no longer contains eleven exact versions
covered by historical exemptions. Those exemptions were removed, and their
original proposal IDs, approvals and expiry dates remain in the ledger with
`status: retired` and a linked retirement receipt. A retired proposal cannot
authorise a reintroduced exemption.

This reduces effective exemptions from 258 to 247; it does **not** reduce the
247 active source-review gaps. The remaining deadline is still 2026-11-10.
Refreshing the four approved peer registries in a temporary store did not close
additional exact-version gaps. No refreshed imports, publisher trust, new audit
certifications or expiry extensions are part of this retirement.

The governance checker rejects effective cargo-vet exemptions without matching,
unexpired approvals. Even an approved exemption must be added separately to
`config.toml`; a proposal alone has no effect on CI.
