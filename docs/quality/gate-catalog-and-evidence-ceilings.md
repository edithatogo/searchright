# Gate catalogue and evidence ceilings

Searchright treats the existence of a check, the execution of a check and the
claim justified by that check as separate facts.

`verification/gate-catalog.json` is generated from the network-free static
harness and assertion-level Conductor traceability. Every registered command
states:

- whether it is part of the default static harness;
- the architectural category it tests;
- whether it may use the network, perform external writes or require a compiler;
- the maximum evidence level it can establish;
- the acceptance assertions that name it;
- an explicit claim boundary.

All static gates are default-deny for network and external writes. A passing
static gate can establish source-level structure, policy, deterministic
reference behaviour or source-package reproducibility. It cannot establish:

- Rust compilation or test execution;
- current upstream API behaviour;
- methodological adequacy or recall;
- remote GitHub state;
- legal approval or permission to redistribute content;
- registry acceptance, operational resilience or product maturity.

The catalogue is regenerated with:

```bash
python scripts/check_gate_catalog.py --write
```

CI uses `--check`, so a new harness or traceability command cannot enter the
claim system without becoming visible in the catalogue.
