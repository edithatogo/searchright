# Evidence debt

`verification/evidence-debt.json` is a deterministic register of work that is
not yet proven at the level required for a stronger claim. It is not a score and
must not be converted into a percentage-complete badge.

The register derives from:

- all Conductor assertion ledgers;
- the executable gate catalogue;
- the maturity dossier;
- public-package policy;
- provider-policy review state.

It reports implementation states, mapping confidence, assertions without
symbol-level mappings, open evidence gates, critical maturity blockers and a
priority-ordered closure queue. It deliberately keeps compiler, live-provider,
methodological, migration and external-review debt visible even when the source
scaffolding is extensive.

Regenerate it with:

```bash
python scripts/generate_evidence_debt.py --write
```

A release may reduce the register only by adding the named evidence. Editing the
counts or claim boundary directly is not an accepted closure mechanism.
