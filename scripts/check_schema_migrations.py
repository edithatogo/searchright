#!/usr/bin/env python3
"""Validate schema-family history and explicit migration plans."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts" / "schema-catalog.json"
REGISTRY = ROOT / "contracts" / "migrations" / "registry.json"
VERSION_RE = re.compile(r"^(?P<family>.+)\.v(?P<version>\d+)\.schema\.json$")


def main() -> int:
    errors: list[str] = []
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    catalog_families: dict[str, dict[int, str]] = {}
    for entry in catalog.get("entries", []):
        name = Path(entry["schema"]).name
        match = VERSION_RE.fullmatch(name)
        if not match:
            errors.append(f"catalog schema name is not versioned: {name}")
            continue
        family = match.group("family")
        version = int(match.group("version"))
        catalog_families.setdefault(family, {})[version] = entry["schema_id"]
    registered = {row.get("family"): row for row in registry.get("families", []) if isinstance(row, dict)}
    multi_version = {family: versions for family, versions in catalog_families.items() if len(versions) > 1}
    if set(multi_version) != set(registered):
        errors.append(
            f"migration family mismatch missing={sorted(set(multi_version)-set(registered))} "
            f"extra={sorted(set(registered)-set(multi_version))}"
        )
    plans = 0
    for family, versions in multi_version.items():
        row = registered.get(family, {})
        declared_versions = {
            item.get("version"): item.get("schema_id")
            for item in row.get("versions", [])
            if isinstance(item, dict)
        }
        if declared_versions != versions:
            errors.append(f"registered versions disagree with catalog for {family}")
        current = row.get("current_write_version")
        if current != max(versions):
            errors.append(f"{family} current_write_version must be highest catalog version")
        if row.get("minimum_read_version") != min(versions):
            errors.append(f"{family} minimum_read_version must preserve the oldest catalog version")
        migrations = row.get("migrations")
        if not isinstance(migrations, list) or len(migrations) < len(versions) - 1:
            errors.append(f"{family} has insufficient migration plans")
            continue
        for relative in migrations:
            path = ROOT / relative
            if not path.is_file():
                errors.append(f"missing migration plan {relative}")
                continue
            plan = json.loads(path.read_text(encoding="utf-8"))
            plans += 1
            if plan.get("family") != family:
                errors.append(f"migration {relative} has wrong family")
            if plan.get("automatic_apply") is not False:
                errors.append(f"migration {relative} must not auto-apply")
            if plan.get("destructive") is not False:
                errors.append(f"migration {relative} must not be destructive")
            if plan.get("backup_required") is not True:
                errors.append(f"migration {relative} must require backup")
            rollback = plan.get("rollback")
            if not isinstance(rollback, dict) or rollback.get("supported") is not True:
                errors.append(f"migration {relative} requires explicit rollback support")
            if plan.get("from_version") not in versions or plan.get("to_version") not in versions:
                errors.append(f"migration {relative} references unknown versions")
            if plan.get("from_version", 0) >= plan.get("to_version", 0):
                errors.append(f"migration {relative} must move forward")
    policy = registry.get("default_policy", {})
    for key, expected in {
        "unknown_version": "reject",
        "destructive_migration": "deny",
        "implicit_write_upgrade": "deny",
    }.items():
        if policy.get(key) != expected:
            errors.append(f"default migration policy {key} must be {expected}")
    if policy.get("backup_required") is not True or policy.get("receipt_required") is not True:
        errors.append("migration policy must require backup and receipts")
    receipt = {
        "schema_version": "org.searchright.schema-migration-receipt.v1",
        "status": "failed" if errors else "passed",
        "catalog_families": len(catalog_families),
        "multi_version_families": len(multi_version),
        "migration_plans": plans,
        "errors": errors,
        "limitations": [
            "Declarative history validation only; compiled readers/writers and representative persisted-data migrations remain higher evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
