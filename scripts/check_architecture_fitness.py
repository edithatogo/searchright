#!/usr/bin/env python3
"""Enforce repository architecture and capability-placement invariants."""
from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "verification" / "architecture-policy.json"
URL_RE = re.compile(r'https://(?:eutils\.ncbi\.nlm\.nih\.gov|www\.ebi\.ac\.uk|api\.crossref\.org|api\.openalex\.org|clinicaltrials\.gov)[^"\s]*')
FINAL_RE = re.compile(r"DecisionValue::Exclude|final_eligibility_decision")


def package_manifests() -> dict[str, tuple[Path, dict[str, Any]]]:
    result: dict[str, tuple[Path, dict[str, Any]]] = {}
    for path in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        value = tomllib.loads(path.read_text(encoding="utf-8"))
        package = value.get("package", {})
        name = package.get("name")
        if isinstance(name, str):
            result[name] = (path, value)
    return result


def dependency_names(manifest: dict[str, Any]) -> set[str]:
    names: set[str] = set()
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        value = manifest.get(section, {})
        if isinstance(value, dict):
            names.update(str(name) for name in value)
    return names


def under(path: Path, roots: list[str]) -> bool:
    relative = path.relative_to(ROOT).as_posix()
    return any(relative == root or relative.startswith(root.rstrip("/") + "/") for root in roots)


def main() -> int:
    errors: list[str] = []
    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    manifests = package_manifests()

    if policy.get("default_policy") != "deny":
        errors.append("architecture policy must be default-deny")

    neutral = set(policy.get("neutral_crates", []))
    forbidden_prefixes = tuple(policy.get("forbidden_dependency_prefixes_for_neutral_crates", []))
    for name in neutral:
        if name not in manifests:
            errors.append(f"neutral crate {name} is missing")
            continue
        path, manifest = manifests[name]
        for dependency in dependency_names(manifest):
            if dependency.startswith(forbidden_prefixes):
                errors.append(f"neutral crate {name} depends on product crate {dependency} in {path.relative_to(ROOT)}")

    for edge in policy.get("forbidden_internal_edges", []):
        source = edge.get("from")
        target = edge.get("to")
        if source not in manifests:
            errors.append(f"forbidden edge source package is missing: {source}")
        elif target in dependency_names(manifests[source][1]):
            errors.append(f"forbidden internal dependency edge {source} -> {target}")

    for dependency, allowed in policy.get("network_dependencies", {}).items():
        allowed_set = set(allowed)
        for name, (path, manifest) in manifests.items():
            if dependency in dependency_names(manifest) and name not in allowed_set:
                errors.append(f"network dependency {dependency} is not allowed in {name} ({path.relative_to(ROOT)})")

    endpoint_roots = list(policy.get("provider_endpoint_source_roots", []))
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if URL_RE.search(text) and not under(path, endpoint_roots):
            errors.append(f"provider endpoint literal is outside connector boundary: {path.relative_to(ROOT)}")

    authority_roots = list(policy.get("final_eligibility_authority_source_roots", []))
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if FINAL_RE.search(text) and not under(path, authority_roots):
            errors.append(f"final eligibility authority marker is outside approved boundary: {path.relative_to(ROOT)}")

    declared_writers: set[str] = set()
    for entry in policy.get("external_write_scripts", []):
        if not isinstance(entry, dict):
            errors.append("external_write_scripts entries must be objects")
            continue
        relative = entry.get("path")
        if not isinstance(relative, str):
            errors.append("external write script path is missing")
            continue
        declared_writers.add(relative)
        path = ROOT / relative
        if not path.is_file():
            errors.append(f"external write script is missing: {relative}")
            continue
        text = path.read_text(encoding="utf-8")
        apply_flag = str(entry.get("apply_flag", ""))
        environment_gate = str(entry.get("environment_gate", ""))
        if apply_flag != "explicit-environment-only" and apply_flag not in text:
            errors.append(f"{relative} does not contain its declared apply flag {apply_flag}")
        env_name = environment_gate.split("=", 1)[0]
        if not env_name or env_name not in text:
            errors.append(f"{relative} does not contain its declared environment gate {environment_gate}")

    for name, (path, manifest) in manifests.items():
        package = manifest.get("package", {})
        if package.get("publish") is not False:
            errors.append(f"workspace package {name} is not explicitly publish=false ({path.relative_to(ROOT)})")

    receipt = {
        "schema_version": "org.searchright.architecture-fitness-receipt.v1",
        "status": "failed" if errors else "passed",
        "packages": len(manifests),
        "neutral_crates": len(neutral),
        "declared_external_write_scripts": len(declared_writers),
        "errors": errors,
        "limitations": [
            "Static source and manifest analysis only; compiler resolution, runtime egress controls and operating-system sandboxing remain separate evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
