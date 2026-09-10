# Optional external human calibration protocol

## Purpose and boundary

This optional protocol may be used when the accountable owner seeks additional
human information-retrieval calibration. It is not a default Searchright project
gate and is not required to execute, review or close the internal agent-panel
workflow.

Human calibration is a separate evidence class. It must not be inferred from an
agent panel, owner adjudication, a reviewer invitation, or an unsigned template.
It does not certify Searchright, approve a release or validate live-provider
performance.

## Materials

- the exact `systematic-search` package identified by SHA-256;
- the authority scenarios in `authority-scenarios.json`;
- one synthetic PICO workflow and one synthetic PCC workflow;
- native-query examples covering at least two platforms;
- the authority, agent-panel, failure-mode, methodology and handoff references.

Do not collect credentials, sensitive identifiers, licensed full text or hidden
benchmark labels.

## Assessment

Each participating information specialist records:

1. relevant experience, conflicts, review date and an attributable or controlled
   pseudonymous identity;
2. whether each operation is allowed, advisory, approval-gated or human-only;
3. whether the six PRESS domains are represented without treating PRESS as an
   automated conduct certificate;
4. whether database/platform, record/report/study and reporting/conduct
   distinctions are preserved;
5. whether any wording implies autonomous exclusion, live access,
   methodological approval or registry acceptance.

Multiple reviewers may be used, but no fixed number is required by the default
project governance model. When more than one reviewer participates, freeze their
first-pass worksheets before adjudication and retain disagreements.

## Evidence and claims

A completed attributable worksheet may support a claim of external human
calibration. A claim of independent human PRESS peer review additionally requires
that the reviewer was independent of the strategy author and that the reviewed
strategy, reviewer identity evidence and independence evidence are recorded.

Agent-panel results and owner adjudication remain separate artefacts and cannot
be relabelled as human calibration.

## Receipt fields

A completed receipt should contain `schema_version`, `skill_package_sha256`,
reviewer records, per-case decisions, conflicts, adjudications,
`blocking_hazards`, `status` and a claim boundary. The accountable owner records
how the optional findings were accepted, rejected, deferred or remediated.
