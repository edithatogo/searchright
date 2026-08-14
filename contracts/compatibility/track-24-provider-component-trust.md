# Provider-component trust contracts compatibility note

The Track 24 component-trust slice adds two alpha contract families:
`provider-component-release-signature` and `provider-component-trust-policy`.
It does not modify or supersede `provider-component.v1`, so existing component
manifest readers and writers remain byte-compatible.

The new contracts are opt-in admission inputs. A caller using only the original
manifest verifier retains its previous behaviour; the stronger
`verify_signed_component_release` entry point additionally requires exact
manifest/component binding, an authorised Ed25519 key, bounded validity and a
reviewed revocation policy. No automatic migration is required because no
persisted v1 document changes shape.

This additive alpha surface does not establish a public distribution registry,
publisher identity, key custody, transparency log, or automatic trust-root
updates. Such authority requires separately reviewed operational evidence.
