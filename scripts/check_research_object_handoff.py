#!/usr/bin/env python3
"""Check the Track 05 plan-only research-object handoff boundary."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / "contracts" / "examples" / "research-object-handoff-plan.json"


def main() -> int:
    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    errors: list[str] = []
    expected = {
        "execution_mode": "dry_run",
        "deposit_authorized": False,
        "ro_crate_conformance_claimed": False,
        "osf_acceptance_claimed": False,
        "delegated_export_track": "25",
    }
    for key, value in expected.items():
        if plan.get(key) != value:
            errors.append(f"{key} must be {value!r}")
    destination_kinds = {
        row.get("kind") for row in plan.get("proposed_destinations", []) if isinstance(row, dict)
    }
    if destination_kinds != {"ro_crate", "osf"}:
        errors.append("the plan must describe both RO-Crate and OSF handoffs")
    if any(
        row.get("external_write") is not True
        for row in plan.get("proposed_destinations", [])
        if isinstance(row, dict)
    ):
        errors.append("every proposed destination must be marked as an external write")
    canonical = json.dumps(plan, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    receipt = {
        "schema_version": "org.searchright.research-object-handoff-check.v1",
        "status": "failed" if errors else "passed",
        "plan_sha256": hashlib.sha256(canonical).hexdigest(),
        "destination_kinds": sorted(destination_kinds),
        "errors": errors,
        "limitations": [
            "Plan-only validation; no RO-Crate was generated, no OSF write was attempted, and no conformance or acceptance was established."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
