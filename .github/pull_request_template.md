## Contract and track

- Conductor track: `NN`
- Multi-track exception: `none`
- Exception tracks (only when absolutely inseparable): `none`
- Why the work cannot be split (required for an exception): `none`
- Contracts changed:
- Evidence level reached:

One PR carries one Conductor track. Use `MULTI` for the Conductor track only
when splitting the change would make either PR uncompilable, unsafe or
unverifiable. A multi-track PR also requires the
`scope:multi-track-exception` label and a concrete inseparability explanation.

This PR is intended to be merged, not parked. Rebase auto-merge is enabled
after the scope policy passes; keep repairing the same PR until all required
checks are green.

## Verification

- [ ] `scripts/verify.sh`
- [ ] deterministic fixtures added/updated
- [ ] public claims remain within evidence
- [ ] security/privacy/licensing impact assessed
- [ ] migration notes added for public contract changes

## Methodological impact

Describe any change to retrieval sensitivity, query translation, deduplication,
screening authority, PRISMA counts or study/report linkage.
