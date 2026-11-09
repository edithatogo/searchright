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
trust is prohibited. Historical approval entries link to issue #241 and expire
on 2026-11-10. Those original approval records are preserved; the current
dependency-review task is [#242](https://github.com/edithatogo/searchright/issues/242),
while #241 owns coverage. The dependency-review backlog remains open until
qualifying audits replace the exemptions.

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

## Peer audit promotion and exemption retirement (2026-11-08)

Twelve exact versions covered by temporary backlog exemptions were proved fully
audited by authoritative upstream peer maintainers in the allowlisted Bytecode
Alliance, Mozilla, and Zcash registries:

- `futures 0.3.34` (Bytecode Alliance: Pat Hickey)
- `futures-channel 0.3.34` (Bytecode Alliance: Pat Hickey)
- `futures-core 0.3.34` (Bytecode Alliance: Pat Hickey; Mozilla: Alex Franchuk)
- `futures-executor 0.3.34` (Bytecode Alliance: Pat Hickey)
- `futures-io 0.3.34` (Bytecode Alliance: Pat Hickey)
- `futures-macro 0.3.34` (Bytecode Alliance: Pat Hickey)
- `shlex 2.0.1` (Bytecode Alliance: Alex Crichton; Mozilla: Emilio Cobos Álvarez)
- `signature 2.2.0` (Zcash: Jack Grigg)
- `simdutf8 0.1.5` (Mozilla: Henri Sivonen)
- `sync_wrapper 1.0.2` (Mozilla: Alex Franchuk)
- `tracing-core 0.1.36` (Bytecode Alliance: Alex Crichton; Mozilla: Alex Franchuk)
- `untrusted 0.9.0` (Mozilla: Alex Franchuk)

Refreshing `supply-chain/imports.lock` from the four allowlisted registries
and verifying with `cargo vet --store-path supply-chain --locked --frozen --no-minimize-exemptions`
confirmed these twelve crates satisfy `safe-to-deploy` via qualifying peer
audit chains. Their temporary exemptions were removed from `supply-chain/config.toml`,
and their ledger proposals were updated to `status: retired` with linked
receipt `verification/receipts/track-16-peer-audit-promotion-2026-11-08.json`.

This increases fully audited dependencies from 48 to 61 (+13) and reduces
effective exemptions from 247 to 235 (-12). Note that `openssl-probe 0.2.1` was
inspected but retains its unreviewed exemption as it lacks a complete peer
`safe-to-deploy` chain, and `untrusted 0.7.1` retains its unreviewed exemption.
The remaining 235 exemptions retain the 2026-11-10 deadline.

## Locked dependency reconciliation (2026-11-09)

Automated updates raised the vendored Noyalib minimum versions for `libm` and
`rustix` without updating the workspace lockfile. Restore their original upstream
requirements (`0.2` and `1.1.4`, also recorded in `Cargo.toml.orig`) so the
committed `libm 0.2.6` and `rustix 1.1.4` graph remains usable with `--locked`.
The existing Mozilla audit covers `libm 0.2.6`; `rustix 1.1.4` retains its existing
temporary exemption. This changes neither locked versions nor accepted reviews.
Reintroduce higher minimum versions together with a coherent lockfile, qualifying
review evidence and native checks; the observed `libm 0.2.16` audit gap remains
unresolved for that future upgrade.

## Deadline remediation sequence

As of 9 November 2026, the active set is 218 `safe-to-deploy` and 17
`safe-to-run` exemptions. The existing checker accepts their 10 November expiry
through that runner calendar date and rejects them from 11 November. This plan
does not renew an exemption or certify a dependency. Keep work in Track 16 and
issue [242](https://github.com/edithatogo/searchright/issues/242), using the
existing Cargo Vet store and proposal history.

Initial triage and the twelve peer-audit retirements above are complete. Estimate
remaining source-review work from the next measured batch; there is no evidence
supporting completion of all 235 remaining entries before the next day's expiry.
The passing Cargo Vet report may count 230 exempted locked packages because
ledger entries also include qualifying delta-audit roots; do not equate these
different populations or delete roots solely because their versions are unlocked.

1. Reproduce the locked graph and Cargo Vet result, then use a temporary copy of
   the store for suggestions, one refresh of the four approved registries and
   pruning analysis. Preserve command failures separately from audit gaps.
2. Compare the refreshed suggestions with the committed snapshot. Inspect any
   imported change before promoting it and preserve peer attribution. The
   6 November refresh found no additional complete paths; the 8 November refresh
   closed twelve entries. Neither observation proves a later refresh will resolve
   the remainder.
3. Assign at most two review batches in parallel, prioritising security exposure
   and then the actual full/delta review size. Retire an exemption only after the
   resolver proves it unnecessary or qualifying audit evidence replaces it.
4. Use the observed 9 November residual set and revise the estimate after each
   real review batch in the existing issue/PR and CI artifacts. Where gaps cannot be resolved by the
   deadline, present the exact versions, exposure, proposed duration and next
   remediation to the owner for a concrete decision. Existing expiry enforcement
   continues until such a decision actually exists.

| Review batch | Starting scope |
| --- | --- |
| Network, authentication and cryptography | TLS/runtime audit roots below; `aws-lc-rs`, `aws-lc-sys`, `ring`, `jsonwebtoken`, `reqwest`, `rmcp` and certificate verification/discovery |
| Build execution and generated code | `cc`, `cmake`, `jobserver`, `pkg-config`, `proc-macro2`, `quote`, `syn`, `serde_derive` and `rmcp-macros` |
| Short residual reviews | Select from actual Cargo Vet suggestions; small utility crates are candidates, not automatically safe |
| Remaining data/platform and development dependencies | Preserve all 17 development-only gaps; build/test dependencies execute code too |

Do not equate absence from `Cargo.lock` with an obsolete exemption. Three current
exempted versions support delta-audit chains to locked versions:

| Exempted root | Existing audit path to the locked version |
| --- | --- |
| `rustls 0.23.43` | Local delta to `0.23.46` |
| `rustls-webpki 0.103.13` | Bytecode Alliance deltas through `0.103.15` to `0.103.16` |
| `tokio 1.53.1` | Local delta to `1.54.1` |

A qualifying full review of the root, another complete qualifying audit path,
or a full review of the locked version is needed before removing those roots.

### Use existing Cargo Vet commands

Run from the repository root with the pinned native tools. The temporary store
keeps exploratory refresh/pruning away from accepted records:

```bash
vet_review_dir="$(mktemp -d)"
mkdir "$vet_review_dir/supply-chain"
cp supply-chain/config.toml supply-chain/audits.toml supply-chain/imports.lock "$vet_review_dir/supply-chain/"
cargo fetch --locked
cargo metadata --locked --offline --all-features --format-version 1 > "$vet_review_dir/metadata.json"
cargo vet --store-path "$vet_review_dir/supply-chain" --locked --frozen --no-minimize-exemptions
cargo vet --store-path "$vet_review_dir/supply-chain" --locked --no-registry-suggestions --output-format=json suggest > "$vet_review_dir/suggestions-before.json"
cargo vet --store-path "$vet_review_dir/supply-chain" --no-minimize-exemptions --no-registry-suggestions
cargo vet --store-path "$vet_review_dir/supply-chain" --locked --no-registry-suggestions --output-format=json suggest > "$vet_review_dir/suggestions-after.json"
cargo vet --store-path "$vet_review_dir/supply-chain" --locked --frozen prune --no-imports --no-audits
```

Inspect each command's result before proceeding; the unlocked check performs the
single configured-registry refresh. Review selected packages with `cargo vet
inspect --mode local` or `cargo vet diff --mode local`, preserving exact versions
and published source identity. See the official [command reference](https://mozilla.github.io/cargo-vet/commands.html).
These are execution instructions, not a claim that the new triage was run.

Each completed batch must bind reviewed source to its published crate/checksum,
attribute the actual reviewer, identify any qualifying delta base, and retire
only resolved proposal entries without changing historical decisions. Run the
existing governance regressions/checker and `cargo vet --locked --frozen
--no-minimize-exemptions`; dependency/feature changes also need native behaviour
checks. Report remaining review gaps separately from obsolete exemptions removed.
Do not use publisher trust, automatic renewal or regenerated exemptions to turn
an unresolved review into a passing claim.
