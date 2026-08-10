#!/usr/bin/env python3
"""Ensure CI Rust tool pins agree with the canonical developer-tool manifest."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "requirements/rust-tools.json"
PATTERN = re.compile(r"cargo install\s+([A-Za-z0-9_-]+)\s+--version\s+(?:=)?([0-9][^\s]*)\s+--locked")


def main() -> int:
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    declared = {tool["crate"]: tool["version"] for tool in data["tools"]}
    errors: list[str] = []
    observed: dict[str, set[str]] = {}
    for workflow in sorted((ROOT / ".github/workflows").glob("*.yml")):
        text = workflow.read_text(encoding="utf-8")
        for crate, version in PATTERN.findall(text):
            observed.setdefault(crate, set()).add(version)
            expected = declared.get(crate)
            if expected is None:
                errors.append(f"{workflow.name}: {crate} is pinned in CI but absent from rust-tools.json")
            elif expected != version:
                errors.append(
                    f"{workflow.name}: {crate} uses {version}, canonical manifest uses {expected}"
                )
    unused = sorted(set(declared) - set(observed))
    if unused:
        errors.append(f"canonical tools are not exercised by a workflow: {unused}")
    receipt = {
        "schema_version": "org.searchright.rust-tools-validation.v1",
        "status": "failed" if errors else "passed",
        "declared_tools": len(declared),
        "workflow_tools": {name: sorted(versions) for name, versions in sorted(observed.items())},
        "errors": errors,
        "limitations": ["Static pin parity only; installation and execution require compiler-backed evidence."],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
