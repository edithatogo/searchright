#!/usr/bin/env python3
"""Verify that all consequential capabilities remain denied by default."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load(path: str) -> object:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main() -> int:
    errors: list[str] = []
    checked = 0

    capabilities = load("context/capability-matrix.json")
    if not isinstance(capabilities, dict) or capabilities.get("default_policy") != "deny":
        errors.append("context capability matrix must use a deny default")
    else:
        for component in capabilities.get("components", []):
            checked += 1
            for field in ("network", "external_writes", "telemetry", "final_exclusion"):
                if component.get(field) is not False:
                    errors.append(
                        f"{component.get('component', '<unknown>')} must default {field}=false"
                    )

    locks = load("integration/locks.json")
    if not isinstance(locks, dict) or locks.get("automatic_updates") is not False:
        errors.append("integration pins must never update automatically")

    passport_index = load("integration/passports/index.json")
    if isinstance(passport_index, dict):
        for item in passport_index.get("passports", []):
            path = item.get("path")
            if not isinstance(path, str):
                errors.append("integration passport index contains an invalid path")
                continue
            passport = load(path)
            if not isinstance(passport, dict):
                errors.append(f"integration passport is not an object: {path}")
                continue
            checked += 1
            for field in (
                "default_network",
                "default_external_writes",
                "default_telemetry",
                "automatic_revision_updates",
            ):
                if passport.get(field) is not False:
                    errors.append(f"{path} must set {field}=false")

    hierarchy = load("conductor/github/issue-hierarchy.json")
    if not isinstance(hierarchy, dict) or hierarchy.get("apply_permitted") is not False:
        errors.append("rendered GitHub issue hierarchy must not authorise remote writes")

    sync_source = (ROOT / "scripts/sync_github_issues.py").read_text(encoding="utf-8")
    required_sync_guards = (
        "SEARCHRIGHT_GITHUB_APPLY",
        "--apply",
        "apply requires a clean Git working tree",
        "apply repository must match the generated hierarchy",
    )
    for guard in required_sync_guards:
        if guard not in sync_source:
            errors.append(f"GitHub issue sync is missing guard: {guard}")

    provider_source = (ROOT / "crates/evidence-search-core/src/provider.rs").read_text(
        encoding="utf-8"
    )
    if "mode == ProviderMode::Live && !request.policy.live_enabled" not in provider_source:
        errors.append("live provider execution is not visibly gated by live_enabled")
    if "mode == ProviderMode::Replay && !request.policy.replay_enabled" not in provider_source:
        errors.append("replay provider execution is not visibly gated by replay_enabled")

    tool_catalog = load("contracts/mcp/tool-catalog.json")
    if isinstance(tool_catalog, dict):
        for tool in tool_catalog.get("tools", []):
            if tool.get("implementation_status") != "implemented":
                continue
            checked += 1
            effect = tool.get("effect")
            authority = tool.get("authority")
            if effect in {"network_and_local_write", "external_write"} and authority in {
                "automatic",
                "read_only",
            }:
                errors.append(
                    f"implemented MCP tool {tool.get('name')} has consequential effect without explicit authority"
                )

    receipt = {
        "schema_version": "org.searchright.default-deny-receipt.v1",
        "status": "failed" if errors else "passed",
        "components_checked": checked,
        "errors": errors,
        "limitations": [
            "Static source-policy validation only; runtime sandboxing and host enforcement require compiler and execution evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
