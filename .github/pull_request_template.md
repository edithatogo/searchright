## Contract and track

- Conductor track: `NN`
- Affected tracks: `none`
- Shared outcome rationale: `none`
- Multi-track exception: `none`
- Exception tracks (only when absolutely inseparable): `none`
- Why the work cannot be split (required for an exception): `none`
- Contracts changed:
- Evidence level reached:

One PR carries one coherent outcome with one primary Conductor track. If other
tracks are affected, list all IDs (including the primary) under Affected tracks
and explain the shared outcome. Split unrelated work. No exception label is
needed for this declaration. Legacy `MULTI` declarations still require the
exception tracks, rationale and `scope:multi-track-exception` label.

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
