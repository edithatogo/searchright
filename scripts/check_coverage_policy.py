#!/usr/bin/env python3
"""Validate the governed coverage ratchet and optionally enforce it on LCOV."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "verification" / "coverage-policy.json"
WORKFLOW_PATH = ROOT / ".github" / "workflows" / "coverage.yml"
CODECOV_PATH = ROOT / "codecov.yml"


def read_lcov_totals(path: Path) -> tuple[int, int]:
    covered = 0
    instrumented = 0
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("LH:"):
            covered += int(line[3:])
        elif line.startswith("LF:"):
            instrumented += int(line[3:])
    if instrumented <= 0 or covered < 0 or covered > instrumented:
        raise ValueError("LCOV must contain valid positive LF/LH totals")
    return covered, instrumented


def validate_policy(policy: object, workflow: str, codecov: str) -> list[str]:
    errors: list[str] = []
    if not isinstance(policy, dict):
        return ["coverage policy must be an object"]
    if policy.get("schema_version") != "org.searchright.coverage-policy.v1":
        errors.append("unexpected coverage-policy schema_version")

    baseline = policy.get("baseline", {})
    admission = policy.get("admission", {})
    maturity = policy.get("maturity", {})
    decision = policy.get("decision", {})
    ratchet = policy.get("ratchet", {})
    try:
        covered = int(baseline["covered_lines"])
        instrumented = int(baseline["instrumented_lines"])
        observed = float(baseline["observed_line_percent"])
        observed_patch = float(baseline["observed_patch_percent"])
        floor = float(admission["minimum_line_percent"])
        patch_floor = float(admission["minimum_patch_percent"])
        target = float(maturity["target_line_percent"])
    except (KeyError, TypeError, ValueError) as exc:
        return [f"coverage policy has invalid numeric fields: {exc}"]

    calculated = round(covered * 100 / instrumented, 2) if instrumented else -1
    if calculated != observed:
        errors.append(f"baseline arithmetic mismatch: declared {observed}, calculated {calculated}")
    if floor > observed:
        errors.append("admission floor cannot exceed observed baseline")
    if patch_floor > observed_patch:
        errors.append("patch floor cannot exceed observed patch baseline")
    if floor < 0 or patch_floor < 0 or target > 100:
        errors.append("coverage percentages must be between zero and 100")
    if target <= 90:
        errors.append("Track 16 maturity target must remain greater than 90 percent")
    if floor >= target:
        errors.append("admission floor must remain distinct from the open maturity target")
    if admission.get("allowed_regression_percent") != 0:
        errors.append("admission policy must not allow regression")
    if ratchet.get("direction") != "increase_only":
        errors.append("coverage ratchet must be increase_only")
    if not decision.get("id") or not decision.get("rationale") or not decision.get("recorded_on"):
        errors.append("coverage change requires an identified, dated decision and rationale")
    commit = str(baseline.get("commit", ""))
    if re.fullmatch(r"[0-9a-f]{40}", commit) is None:
        errors.append("baseline commit must be a full Git object ID")

    if "python scripts/check_coverage_policy.py --lcov lcov.info" not in workflow:
        errors.append("coverage workflow must enforce the machine-readable policy")
    if "fail-under-lines" in workflow:
        errors.append("coverage workflow must not duplicate a hard-coded policy threshold")
    project_target = re.search(r"project:[\s\S]*?target:\s*([0-9.]+)%", codecov)
    patch_target = re.search(r"patch:[\s\S]*?target:\s*([0-9.]+)%", codecov)
    if project_target is None or float(project_target.group(1)) != floor:
        errors.append("Codecov project target must equal the governed admission floor")
    if patch_target is None or float(patch_target.group(1)) != patch_floor:
        errors.append("Codecov patch target must equal the governed patch floor")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lcov", type=Path)
    args = parser.parse_args()
    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    errors = validate_policy(
        policy,
        WORKFLOW_PATH.read_text(encoding="utf-8"),
        CODECOV_PATH.read_text(encoding="utf-8"),
    )
    observed: float | None = None
    if args.lcov:
        try:
            covered, instrumented = read_lcov_totals(args.lcov)
            observed = covered * 100 / instrumented
            floor = float(policy["admission"]["minimum_line_percent"])
            if observed < floor:
                errors.append(f"line coverage {observed:.2f}% is below admission floor {floor:.2f}%")
        except (OSError, ValueError, KeyError, TypeError) as exc:
            errors.append(f"cannot enforce LCOV: {exc}")
    receipt = {
        "schema_version": "org.searchright.coverage-policy-receipt.v1",
        "status": "failed" if errors else "passed",
        "admission_floor_percent": policy.get("admission", {}).get("minimum_line_percent"),
        "maturity_target_percent": policy.get("maturity", {}).get("target_line_percent"),
        "observed_line_percent": round(observed, 2) if observed is not None else None,
        "errors": errors,
        "claim_boundary": policy.get("claim_boundary"),
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
