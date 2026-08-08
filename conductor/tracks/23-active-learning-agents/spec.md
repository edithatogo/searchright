# 23: Active learning and calibrated agent teams

## Objective

Add prioritisation and adversarial agents without compromising recall or authority.

## Scope

- Define outcome-gradable screening benchmark and thresholds
- Compare deterministic, classical IR and model rankings
- Calibrate uncertainty/FNR/FPR with human gold decisions
- Add independent strategy generator and PRESS adversary roles
- Prevent shared-context leakage and sycophantic concession
- Allow stopping rules only under explicit protocol and external validation

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `23`.

## Acceptance contract

- All named tasks have implementation, deterministic tests, documentation and a
  machine-readable verification receipt or an explicit external blocker.
- Public claims remain at the achieved evidence level.
- Security, privacy, migration and rollback impact are reviewed.
- CLI/MCP/facade parity is preserved where the track changes a public operation.

## Out of scope

Work owned by later tracks is documented but not promoted as implemented.
