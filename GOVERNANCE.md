# Governance

Searchright is initially a maintainer-led open-source project. Architectural,
contract and methodological changes are recorded as ADRs and Conductor tracks.

## Decision classes

- **Routine:** backwards-compatible implementation and documentation changes.
- **Contract:** schema, CLI, MCP or public Rust API changes; require compatibility
  evidence and migration notes.
- **Methodological:** changes that alter search sensitivity, deduplication,
  screening authority or reporting; require a methods rationale and fixtures.
- **Security/privacy:** egress, credentials, plugin permissions, telemetry or
  full-text handling; require threat-model review.
- **External claim:** registry, benchmark or standards-compliance claims; require
  public evidence and must not be inferred from repository metadata alone.

One maintainer may merge changes after automated gates pass. The repository does
not require performative human approvals; high-risk decisions instead require a
machine-readable decision record and explicit evidence receipt.
