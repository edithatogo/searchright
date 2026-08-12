#!/usr/bin/env python3
"""Validate the executable, fail-closed launch-preparation roadmap."""
from __future__ import annotations

import json
import re
from pathlib import Path, PurePosixPath
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
ROADMAP = ROOT / "conductor" / "launch-preparation-roadmap.json"
COVERAGE = ROOT / "conductor" / "roadmap-coverage.json"
TASK_ID = re.compile(r"^LP-\d{3}$")


def validate(data: Any, track_ids: set[str]) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["launch roadmap must be an object"]
    if data.get("schema_version") != "org.searchright.launch-preparation-roadmap.v1":
        errors.append("unsupported launch roadmap schema_version")
    if data.get("status") != "not_ready":
        errors.append("launch roadmap must remain not_ready until a separate maturity decision")
    packages = data.get("work_packages")
    if not isinstance(packages, list) or not packages:
        return errors + ["work_packages must be a non-empty array"]
    rows = [row for row in packages if isinstance(row, dict)]
    if len(rows) != len(packages):
        errors.append("every work package must be an object")
    ids = [row.get("id") for row in rows]
    if len(ids) != len(set(ids)) or any(not isinstance(item, str) or not TASK_ID.fullmatch(item) for item in ids):
        errors.append("work-package IDs must be unique LP-NNN values")
    known = {str(item) for item in ids}
    graph: dict[str, list[str]] = {}
    for row in rows:
        key = str(row.get("id"))
        owner = row.get("owner_track")
        if owner not in track_ids:
            errors.append(f"{key} has unknown owner_track {owner!r}")
        dependencies = row.get("depends_on")
        if not isinstance(dependencies, list) or any(dep not in known for dep in dependencies):
            errors.append(f"{key} has invalid depends_on")
            dependencies = []
        if key in dependencies:
            errors.append(f"{key} depends on itself")
        graph[key] = list(dependencies)
        commands = row.get("commands")
        receipts = row.get("required_receipts")
        if not isinstance(commands, list) or not commands or any(
            not isinstance(item, str)
            or not item.strip()
            or not item.startswith(("cargo ", "python "))
            for item in commands
        ):
            errors.append(f"{key} requires at least one executable command")
        if not isinstance(receipts, list) or not receipts:
            errors.append(f"{key} requires receipt paths under verification/receipts")
        else:
            for receipt in receipts:
                if not isinstance(receipt, str):
                    errors.append(f"{key} requires receipt paths under verification/receipts")
                    continue
                path = PurePosixPath(receipt)
                if (
                    path.is_absolute()
                    or "\\" in receipt
                    or ".." in path.parts
                    or path.suffix != ".json"
                    or path.parts[:2] != ("verification", "receipts")
                ):
                    errors.append(f"{key} has unsafe receipt path {receipt!r}")
        if not isinstance(row.get("exit_criterion"), str) or len(row["exit_criterion"].strip()) < 30:
            errors.append(f"{key} requires a concrete exit_criterion")
        if type(row.get("external_gate")) is not bool:
            errors.append(f"{key} requires a boolean external_gate")
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(key: str) -> None:
        if key in visiting:
            errors.append(f"dependency cycle includes {key}")
            return
        if key in visited:
            return
        visiting.add(key)
        for dependency in graph.get(key, []):
            visit(dependency)
        visiting.remove(key)
        visited.add(key)

    for key in sorted(graph):
        visit(key)
    return errors


def main() -> int:
    data = json.loads(ROADMAP.read_text(encoding="utf-8"))
    coverage = json.loads(COVERAGE.read_text(encoding="utf-8"))
    track_ids = {str(row["track_id"]) for row in coverage.get("tracks", [])}
    errors = validate(data, track_ids)
    rows = data.get("work_packages", []) if isinstance(data, dict) else []
    receipt = {
        "schema_version": "org.searchright.launch-preparation-roadmap-receipt.v1",
        "status": "failed" if errors else "passed",
        "work_packages": len(rows),
        "external_gates": sum(1 for row in rows if isinstance(row, dict) and row.get("external_gate") is True),
        "errors": errors,
        "claim_boundary": "Static roadmap validation does not execute or complete any launch gate."
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
